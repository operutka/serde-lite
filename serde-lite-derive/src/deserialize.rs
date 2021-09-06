use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Variant,
};

use crate::attributes;

/// Expand the derive Deserialize.
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => expand_for_struct(input.ident, input.generics, data, &input.attrs),
        Data::Enum(data) => expand_for_enum(input.ident, input.generics, data, &input.attrs),
        Data::Union(_) => panic!("derive Deserialize is not supported for union types"),
    };

    proc_macro::TokenStream::from(expanded)
}

/// Expand Deserialize for a given struct.
fn expand_for_struct(
    name: Ident,
    generics: Generics,
    data: DataStruct,
    _: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let deserialize = match data.fields {
        Fields::Named(fields) => expand_struct_named_fields(fields),
        Fields::Unnamed(fields) => expand_struct_unnamed_fields(fields),
        Fields::Unit => quote! {
            Ok(Self)
        },
    };

    let expanded = quote! {
        #[allow(unused_variables)]
        impl #impl_generics serde_lite::Deserialize for #name #ty_generics #where_clause {
            fn deserialize(__val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error> {
                #deserialize
            }
        }
    };

    expanded
}

/// Expand Deserialize for named struct fields.
fn expand_struct_named_fields(fields: FieldsNamed) -> TokenStream {
    let (deserialize, constructor) = deserialize_named_fields(&fields);

    quote! {
        #deserialize

        Ok(Self {
            #constructor
        })
    }
}

/// Expand Deserialize for unnamed struct fields.
fn expand_struct_unnamed_fields(fields: FieldsUnnamed) -> TokenStream {
    let (deserialize, constructor) = deserialize_unnamed_fields(&fields);

    quote! {
        #deserialize

        Ok(Self(#constructor))
    }
}

/// Expand Deserialize for a given enum.
fn expand_for_enum(
    name: Ident,
    generics: Generics,
    data: DataEnum,
    attrs: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if data.variants.is_empty() {
        panic!("enum with no variants cannot be deserialized")
    }

    let deserialize = if let Some(tag) = attributes::get_enum_tag(attrs) {
        let content = attributes::get_enum_content(attrs);

        expand_internally_tagged_enum(&tag, content.as_deref(), data)
    } else {
        expand_externally_tagged_enum(data)
    };

    quote! {
        #[allow(unused_variables)]
        impl #impl_generics serde_lite::Deserialize for #name #ty_generics #where_clause {
            fn deserialize(__val: &serde_lite::Intermediate) -> Result<Self, serde_lite::Error> {
                #deserialize
            }
        }
    }
}

/// Expand Deserialize for an internally tagged enum or an adjacently tagged
/// enum.
fn expand_internally_tagged_enum(
    tag_field: &str,
    content_field: Option<&str>,
    data: DataEnum,
) -> TokenStream {
    let mut deserialize = TokenStream::new();

    for variant in data.variants.into_iter() {
        let sname = attributes::get_variant_name(&variant);
        let lname = Literal::string(&sname);
        let constructor = if content_field.is_some() {
            // This is a bit counter-intuitive. It means that the enum content
            // is in a sub-field and we don't know yet if the field exists.
            // Therefore, we have to use the construct_enum_variant function
            // here which will check if the field exists.
            construct_enum_variant(&variant, content_field)
        } else {
            // Here the enum content is a part of the currently deserialized
            // object, so we don't need to check anything.
            construct_enum_variant_with_content(&variant)
        };

        deserialize.extend(quote! {
            #lname => { #constructor }
        });
    }

    let content = if let Some(content) = content_field {
        let lcontent = Literal::string(content);

        quote! {
            let __content = __obj.get(#lcontent);
        }
    } else {
        quote! {
            let __content = __val;
        }
    };

    let ltag = Literal::string(tag_field);

    quote! {
        let __obj = __val.as_map().ok_or_else(|| serde_lite::Error::invalid_value("object"))?;

        let __variant = __obj
            .get(#ltag)
            .map(|v| v.as_str())
            .ok_or_else(|| serde_lite::Error::MissingField)
            .and_then(|v| v.ok_or_else(|| serde_lite::Error::invalid_value("enum variant name")))
            .map_err(|err| serde_lite::Error::NamedFieldErrors(vec![
                (String::from(#ltag), err),
            ]))?;

        #content

        match __variant {
            #deserialize
            _ => Err(serde_lite::Error::UnknownEnumVariant),
        }
    }
}

/// Expand Deserialize for an externally tagged enum.
fn expand_externally_tagged_enum(data: DataEnum) -> TokenStream {
    let mut plain = TokenStream::new();
    let mut with_content = TokenStream::new();

    for (index, variant) in data.variants.into_iter().enumerate() {
        let sname = attributes::get_variant_name(&variant);
        let lname = Literal::string(&sname);
        let constructor_with_content = construct_enum_variant_with_content(&variant);
        let constructor_without_content = construct_enum_variant_without_content(&variant, None);

        plain.extend(quote! {
            #lname => { #constructor_without_content }
        });

        if index == 0 {
            with_content.extend(quote! {
                if let Some(__content) = __obj.get(#lname) {
                    #constructor_with_content
                }
            })
        } else {
            with_content.extend(quote! {
                else if let Some(__content) = __obj.get(#lname) {
                    #constructor_with_content
                }
            })
        }
    }

    quote! {
        if let Some(__obj) = __val.as_map() {
            #with_content
            else {
                Err(serde_lite::Error::UnknownEnumVariant)
            }
        } else if let Some(__variant) = __val.as_str() {
            match __variant {
                #plain
                _ => Err(serde_lite::Error::UnknownEnumVariant),
            }
        } else {
            Err(serde_lite::Error::invalid_value("enum variant"))
        }
    }
}

/// Generate code for constructing a given enum variant.
fn construct_enum_variant(variant: &Variant, content_field: Option<&str>) -> TokenStream {
    let with_content = construct_enum_variant_with_content(variant);
    let without_content = construct_enum_variant_without_content(variant, content_field);

    quote! {
        if let Some(__content) = __content {
            #with_content
        } else {
            #without_content
        }
    }
}

/// Generate code for constructing a given enum variant and use the available
/// variant content.
fn construct_enum_variant_with_content(variant: &Variant) -> TokenStream {
    match &variant.fields {
        Fields::Named(fields) => construct_struct_enum_variant(variant, fields),
        Fields::Unnamed(fields) => construct_tuple_enum_variant(variant, fields),
        Fields::Unit => construct_unit_enum_variant(variant),
    }
}

/// Generate code for constructing a given enum variant without any content.
fn construct_enum_variant_without_content(
    variant: &Variant,
    content_field: Option<&str>,
) -> TokenStream {
    match &variant.fields {
        Fields::Named(fields) if fields.named.is_empty() => {
            return construct_struct_enum_variant(variant, fields);
        }
        Fields::Unnamed(fields) if fields.unnamed.is_empty() => {
            return construct_tuple_enum_variant(variant, fields);
        }
        Fields::Unit => return construct_unit_enum_variant(variant),
        _ => (),
    }

    if let Some(content) = content_field {
        let lcontent = Literal::string(content);

        quote! {
            Err(serde_lite::Error::NamedFieldErrors(vec![
                (String::from(#lcontent), serde_lite::Error::MissingField)
            ]))
        }
    } else {
        quote! {
            Err(serde_lite::Error::MissingEnumVariantContent)
        }
    }
}

/// Generate code for constructing a given struct-like enum variant.
fn construct_struct_enum_variant(variant: &Variant, fields: &FieldsNamed) -> TokenStream {
    let mut init = TokenStream::new();

    if !fields.named.is_empty() {
        init.extend(quote! {
            let __val = __content;
        });
    }

    let (deserialize, constructor) = deserialize_named_fields(fields);

    let ident = &variant.ident;

    quote! {
        #init
        #deserialize

        Ok(Self::#ident {
            #constructor
        })
    }
}

/// Generate code for constructing a given tuple-like enum variant.
fn construct_tuple_enum_variant(variant: &Variant, fields: &FieldsUnnamed) -> TokenStream {
    let mut init = TokenStream::new();

    if !fields.unnamed.is_empty() {
        init.extend(quote! {
            let __val = __content;
        });
    }

    let (deserialize, constructor) = deserialize_unnamed_fields(fields);

    let ident = &variant.ident;

    quote! {
        #init
        #deserialize

        Ok(Self::#ident(#constructor))
    }
}

/// Generate code for constructing a given enum variant.
fn construct_unit_enum_variant(variant: &Variant) -> TokenStream {
    let ident = &variant.ident;

    quote! {
        Ok(Self::#ident)
    }
}

/// Generate code for deserializing given named fields.
fn deserialize_named_fields(fields: &FieldsNamed) -> (TokenStream, TokenStream) {
    let mut deserialize = TokenStream::new();
    let mut constructor = TokenStream::new();

    if !fields.named.is_empty() {
        deserialize.extend(quote! {
            let __obj = __val
                .as_map()
                .ok_or_else(|| serde_lite::Error::invalid_value("object"))?;

            let mut __field_errors = Vec::new();
        });
    }

    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let sname = attributes::get_field_name(field);
        let lname = Literal::string(&sname);
        let skip = attributes::has_flag(&field.attrs, "skip")
            || attributes::has_flag(&field.attrs, "skip_deserializing");

        if skip {
            deserialize.extend(quote! {
                let #name: #ty = Default::default();
            });
        } else if attributes::has_flag(&field.attrs, "flatten") {
            deserialize.extend(quote! {
                let #name = match <#ty as serde_lite::Deserialize>::deserialize(__val) {
                    Ok(v) => Some(v),
                    Err(serde_lite::Error::NamedFieldErrors(errors)) => {
                        __field_errors.extend(errors);
                        None
                    }
                    Err(err) => return Err(err),
                };
            });
        } else if attributes::has_flag(&field.attrs, "default") {
            deserialize.extend(quote! {
                let #name = __obj
                    .get(#lname)
                    .map(<#ty as serde_lite::Deserialize>::deserialize)
                    .unwrap_or_else(|| Ok(Default::default()))
                    .map_err(|err| __field_errors.push((String::from(#lname), err)))
                    .ok();
            });
        } else {
            deserialize.extend(quote! {
                let #name = __obj
                    .get(#lname)
                    .map(<#ty as serde_lite::Deserialize>::deserialize)
                    .unwrap_or_else(|| Err(serde_lite::Error::MissingField))
                    .map_err(|err| __field_errors.push((String::from(#lname), err)))
                    .ok();
            });
        }

        if skip {
            constructor.extend(quote! {
                #name,
            });
        } else {
            constructor.extend(quote! {
                #name: #name.unwrap(),
            });
        }
    }

    if !fields.named.is_empty() {
        deserialize.extend(quote! {
            if !__field_errors.is_empty() {
                return Err(serde_lite::Error::NamedFieldErrors(__field_errors));
            }
        });
    }

    (deserialize, constructor)
}

/// Generate code for deserializing given unnamed fields.
fn deserialize_unnamed_fields(fields: &FieldsUnnamed) -> (TokenStream, TokenStream) {
    let mut deserialize = TokenStream::new();
    let mut constructor = TokenStream::new();

    if !fields.unnamed.is_empty() {
        let len = Literal::usize_unsuffixed(fields.unnamed.len());

        match fields.unnamed.len() {
            0 => (),
            1 => deserialize.extend(quote! {
                let __arr = std::slice::from_ref(__val);
            }),
            _ => deserialize.extend(quote! {
                let __arr = __val
                    .as_array()
                    .ok_or_else(|| serde_lite::Error::invalid_value("array"))?;

                if __arr.len() < #len {
                    return Err(serde_lite::Error::invalid_value(concat!("array of length ", #len)));
                }
            }),
        }

        deserialize.extend(quote! {
            let mut __field_errors = Vec::new();
        });
    }

    for (index, field) in fields.unnamed.iter().enumerate() {
        let ty = &field.ty;
        let sname = format!("f{}", index);
        let name = Ident::new(&sname, Span::call_site());
        let lindex = Literal::usize_unsuffixed(index);

        deserialize.extend(quote! {
            let #name = <#ty as serde_lite::Deserialize>::deserialize(&__arr[#lindex])
                .map_err(|err| __field_errors.push((#lindex, err)))
                .ok();
        });

        if index > 0 {
            constructor.extend(quote! {
                , #name.unwrap()
            });
        } else {
            constructor.extend(quote! {
                #name.unwrap()
            });
        }
    }

    match fields.unnamed.len() {
        0 => (),
        1 => deserialize.extend(quote! {
            if let Some((_, err)) = __field_errors.pop() {
                return Err(err);
            }
        }),
        _ => deserialize.extend(quote! {
            if !__field_errors.is_empty() {
                return Err(serde_lite::Error::UnnamedFieldErrors(__field_errors));
            }
        }),
    }

    (deserialize, constructor)
}
