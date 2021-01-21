use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Variant,
};

use crate::attributes;

/// Expand derive Update.
pub fn derive_update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => expand_for_struct(input.ident, input.generics, data, &input.attrs),
        Data::Enum(data) => expand_for_enum(input.ident, input.generics, data, &input.attrs),
        Data::Union(_) => panic!("derive Update is not supported for union types"),
    };

    proc_macro::TokenStream::from(expanded)
}

/// Expand Update for a given struct.
fn expand_for_struct(
    name: Ident,
    generics: Generics,
    data: DataStruct,
    _: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let update = match data.fields {
        Fields::Named(fields) => expand_named_fields(fields),
        Fields::Unnamed(fields) => expand_unnamed_fields(fields),
        Fields::Unit => quote! {
            Ok(())
        },
    };

    let expanded = quote! {
        impl #impl_generics serde_lite::Update for #name #ty_generics #where_clause {
            fn update(&mut self, __val: &serde_lite::Intermediate) -> Result<(), serde_lite::Error> {
                #update
            }
        }
    };

    expanded
}

/// Expand Update for given named struct fields.
fn expand_named_fields(fields: FieldsNamed) -> TokenStream {
    let (deconstructor, update) = update_named_fields(&fields);

    let mut init = TokenStream::new();

    if !fields.named.is_empty() {
        init.extend(quote! {
            let Self { #deconstructor } = self;
        });
    }

    quote! {
        #init
        #update
        Ok(())
    }
}

/// Expand Update for given unnamed struct fields.
fn expand_unnamed_fields(fields: FieldsUnnamed) -> TokenStream {
    let (deconstructor, update) = update_unnamed_fields(&fields);

    let mut init = TokenStream::new();

    if !fields.unnamed.is_empty() {
        init.extend(quote! {
            let Self(#deconstructor) = self;
        });
    }

    quote! {
        #init
        #update
        Ok(())
    }
}

/// Expand Update for a given enum.
fn expand_for_enum(
    name: Ident,
    generics: Generics,
    data: DataEnum,
    attrs: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let update = if let Some(tag) = attributes::get_enum_tag(attrs) {
        let content = attributes::get_enum_content(attrs);

        expand_internally_tagged_enum(&tag, content.as_deref(), data)
    } else {
        expand_externally_tagged_enum(data)
    };

    quote! {
        impl #impl_generics serde_lite::Update for #name #ty_generics #where_clause {
            fn update(&mut self, __val: &serde_lite::Intermediate) -> Result<(), serde_lite::Error> {
                #update
                Ok(())
            }
        }
    }
}

/// Expand Update for a given internally tagged enum or a given adjacently
/// tagged enum.
fn expand_internally_tagged_enum(
    tag_field: &str,
    content_field: Option<&str>,
    data: DataEnum,
) -> TokenStream {
    let mut update = TokenStream::new();

    for variant in &data.variants {
        let sname = attributes::get_variant_name(variant);
        let lname = Literal::string(&sname);
        let update_varaint = if content_field.is_some() {
            // This is a bit counter-intuitive. It means that the enum content
            // is in a sub-field and we don't know yet if the field exists.
            // Therefore, we have to use the construct_enum_variant function
            // here which will check if the field exists.
            update_enum_variant(variant, content_field)
        } else {
            // Here the enum content is a part of the currently deserialized
            // object, so we don't need to check anything.
            update_enum_variant_with_content(variant)
        };

        update.extend(quote! {
            #lname => { #update_varaint }
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

    let current_variant = get_current_enum_variant(&data);

    let ltag = Literal::string(tag_field);

    quote! {
        let __obj = __val.as_map().ok_or_else(|| serde_lite::Error::invalid_value("object"))?;

        #current_variant

        let __variant = __obj
            .get(#ltag)
            .map(|v| v.as_str())
            .unwrap_or_else(|| Some(__current_variant))
            .ok_or_else(|| serde_lite::Error::NamedFieldErrors(vec![
                (String::from(#ltag), serde_lite::Error::invalid_value("enum variant name")),
            ]))?;

        #content

        match __variant {
            #update
            _ => return Err(serde_lite::Error::UnknownEnumVariant),
        }
    }
}

/// Expand Update for a given externally tagged enum.
fn expand_externally_tagged_enum(data: DataEnum) -> TokenStream {
    let mut plain = TokenStream::new();
    let mut with_content = TokenStream::new();

    for (index, variant) in data.variants.into_iter().enumerate() {
        let sname = attributes::get_variant_name(&variant);
        let lname = Literal::string(&sname);
        let update_variant_with_content = update_enum_variant_with_content(&variant);
        let update_variant_without_content = update_enum_variant_without_content(&variant, None);

        plain.extend(quote! {
            #lname => { #update_variant_without_content }
        });

        if index == 0 {
            with_content.extend(quote! {
                if let Some(__content) = __obj.get(#lname) {
                    #update_variant_with_content
                }
            })
        } else {
            with_content.extend(quote! {
                else if let Some(__content) = __obj.get(#lname) {
                    #update_variant_with_content
                }
            })
        }
    }

    quote! {
        if let Some(__obj) = __val.as_map() {
            #with_content
            else {
                return Err(serde_lite::Error::UnknownEnumVariant);
            }
        } else if let Some(__variant) = __val.as_str() {
            match __variant {
                #plain
                _ => return Err(serde_lite::Error::UnknownEnumVariant),
            }
        } else {
            return Err(serde_lite::Error::invalid_value("enum variant"));
        }
    }
}

/// Generate code to get the current enum variant name.
fn get_current_enum_variant(data: &DataEnum) -> TokenStream {
    let mut match_arms = TokenStream::new();

    for variant in &data.variants {
        let name = &variant.ident;
        let sname = attributes::get_variant_name(variant);
        let lname = Literal::string(&sname);

        match &variant.fields {
            Fields::Named(_) => match_arms.extend(quote! {
                Self::#name { .. } => #lname,
            }),
            Fields::Unnamed(_) => match_arms.extend(quote! {
                Self::#name(..) => #lname,
            }),
            Fields::Unit => match_arms.extend(quote! {
                Self::#name => #lname,
            }),
        }
    }

    quote! {
        let __current_variant = match self {
            #match_arms
        };
    }
}

/// Generate code for updating a given enum variant.
fn update_enum_variant(variant: &Variant, content_field: Option<&str>) -> TokenStream {
    let with_content = update_enum_variant_with_content(variant);
    let without_content = update_enum_variant_without_content(variant, content_field);

    quote! {
        if let Some(__content) = __content {
            #with_content
        } else {
            #without_content
        }
    }
}

/// Generate code for updating a given enum variant and use the available
/// variant content.
fn update_enum_variant_with_content(variant: &Variant) -> TokenStream {
    match &variant.fields {
        Fields::Named(fields) => update_struct_enum_variant(variant, fields),
        Fields::Unnamed(fields) => update_tuple_enum_variant(variant, fields),
        Fields::Unit => update_unit_enum_variant(variant),
    }
}

/// Generate code for updating a given enum variant without variant content.
fn update_enum_variant_without_content(
    variant: &Variant,
    content_field: Option<&str>,
) -> TokenStream {
    match &variant.fields {
        Fields::Named(fields) if fields.named.is_empty() => {
            return update_struct_enum_variant(variant, fields);
        }
        Fields::Unnamed(fields) if fields.unnamed.is_empty() => {
            return update_tuple_enum_variant(variant, fields);
        }
        Fields::Unit => return update_unit_enum_variant(variant),
        _ => (),
    }

    if let Some(content) = content_field {
        let lcontent = Literal::string(content);

        quote! {
            return Err(serde_lite::Error::NamedFieldErrors(vec![
                (String::from(#lcontent), serde_lite::Error::MissingField),
            ]));
        }
    } else {
        quote! {
            return Err(serde_lite::Error::MissingEnumVariantContent);
        }
    }
}

/// Generate code for updating a given struct-like enum variant.
fn update_struct_enum_variant(variant: &Variant, fields: &FieldsNamed) -> TokenStream {
    let mut init = TokenStream::new();

    if !fields.named.is_empty() {
        init.extend(quote! {
            let __val = __content;
        });
    }

    let (deconstructor, update) = update_named_fields(fields);

    let ident = &variant.ident;

    quote! {
        if let Self::#ident { #deconstructor } = self {
            #init
            #update
        } else {
            *self = Self::deserialize(__val)?;
        }
    }
}

/// Generate code for updating a given tuple-like enum variant.
fn update_tuple_enum_variant(variant: &Variant, fields: &FieldsUnnamed) -> TokenStream {
    let mut init = TokenStream::new();

    if !fields.unnamed.is_empty() {
        init.extend(quote! {
            let __val = __content;
        });
    }

    let (deconstructor, update) = update_unnamed_fields(fields);

    let ident = &variant.ident;

    quote! {
        if let Self::#ident(#deconstructor) = self {
            #init
            #update
        } else {
            *self = Self::deserialize(__val)?;
        }
    }
}

/// Generate code for updating a given enum variant.
fn update_unit_enum_variant(variant: &Variant) -> TokenStream {
    let ident = &variant.ident;

    quote! {
        *self = Self::#ident;
    }
}

/// Generate code for updating given named field.
fn update_named_fields(fields: &FieldsNamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();
    let mut update = TokenStream::new();

    if !fields.named.is_empty() {
        update.extend(quote! {
            let __obj = __val
                .as_map()
                .ok_or_else(|| serde_lite::Error::invalid_value("object"))?;

            let mut __field_errors = Vec::new();
        });
    }

    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();
        let sname = attributes::get_field_name(&field);
        let lname = Literal::string(&sname);

        deconstructor.extend(quote! {
            #name,
        });

        if attributes::has_flag(&field.attrs, "skip")
            || attributes::has_flag(&field.attrs, "skip_deserializing")
        {
            continue;
        }

        if attributes::has_flag(&field.attrs, "flatten") {
            update.extend(quote! {
                if let Err(err) = serde_lite::Update::update(#name, __val) {
                    if let serde_lite::Error::NamedFieldErrors(errors) = err {
                        __field_errors.extend(errors);
                    } else {
                        return Err(err);
                    }
                }
            });
        } else {
            update.extend(quote! {
                if let Some(__v) = __obj.get(#lname) {
                    if let Err(err) = serde_lite::Update::update(#name, __v) {
                        __field_errors.push((String::from(#lname), err));
                    }
                }
            });
        }
    }

    if !fields.named.is_empty() {
        update.extend(quote! {
            if !__field_errors.is_empty() {
                return Err(serde_lite::Error::NamedFieldErrors(__field_errors));
            }
        });
    }

    (deconstructor, update)
}

/// Generate code for updating given unnamed fields.
fn update_unnamed_fields(fields: &FieldsUnnamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();
    let mut update = TokenStream::new();

    if !fields.unnamed.is_empty() {
        match fields.unnamed.len() {
            0 => (),
            1 => update.extend(quote! {
                let __arr = __val.as_array().unwrap_or_else(|| std::slice::from_ref(__val));
            }),
            _ => update.extend(quote! {
                let __arr = __val
                    .as_array()
                    .ok_or_else(|| serde_lite::Error::invalid_value("array"))?;
            }),
        }

        let len = Literal::usize_unsuffixed(fields.unnamed.len());

        update.extend(quote! {
            if __arr.len() < #len {
                return Err(serde_lite::Error::invalid_value(concat!("array of length ", #len)));
            }

            let mut __field_errors = Vec::new();
        });
    }

    for (index, _) in fields.unnamed.iter().enumerate() {
        let sname = format!("f{}", index);
        let name = Ident::new(&sname, Span::call_site());
        let lindex = Literal::usize_unsuffixed(index);

        if index == 0 {
            deconstructor.extend(quote! {
                #name
            });
        } else {
            deconstructor.extend(quote! {
                , #name
            });
        }

        update.extend(quote! {
            if let Err(err) = serde_lite::Update::update(#name, &__arr[#lindex]) {
                __field_errors.push((#lindex, err));
            }
        });
    }

    match fields.unnamed.len() {
        0 => (),
        1 => update.extend(quote! {
            if __val.as_array().is_some() {
                if !__field_errors.is_empty() {
                    return Err(serde_lite::Error::UnnamedFieldErrors(__field_errors));
                }
            } else if let Some((_, err)) = __field_errors.pop() {
                return Err(err);
            }
        }),
        _ => update.extend(quote! {
            if !__field_errors.is_empty() {
                return Err(serde_lite::Error::UnnamedFieldErrors(__field_errors));
            }
        }),
    }

    (deconstructor, update)
}
