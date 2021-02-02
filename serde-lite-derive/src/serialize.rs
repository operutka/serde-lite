use std::str::FromStr;

use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Variant,
};

use crate::attributes;

/// Expand derive Serialize.
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => expand_for_struct(input.ident, input.generics, data, &input.attrs),
        Data::Enum(data) => expand_for_enum(input.ident, input.generics, data, &input.attrs),
        Data::Union(_) => panic!("derive Serialize is not supported for union types"),
    };

    proc_macro::TokenStream::from(expanded)
}

/// Expand Serialize for a given struct.
fn expand_for_struct(
    name: Ident,
    generics: Generics,
    data: DataStruct,
    _: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let serialize = match data.fields {
        Fields::Named(fields) => expand_struct_named_fields(&fields),
        Fields::Unnamed(fields) => expand_struct_unnamed_fields(&fields),
        Fields::Unit => quote! {
            Ok(serde_lite::Intermediate::None)
        },
    };

    quote! {
        #[allow(unused_variables)]
        impl #impl_generics serde_lite::Serialize for #name #ty_generics #where_clause {
            fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
                #serialize
            }
        }
    }
}

/// Expand Serialize for given named struct fields.
fn expand_struct_named_fields(fields: &FieldsNamed) -> TokenStream {
    let (deconstructor, serialize) = serialize_named_fields(fields);

    quote! {
        let Self { #deconstructor } = self;

        #serialize

        Ok(serde_lite::Intermediate::Map(__map))
    }
}

/// Expand Serialize for given unnamed struct fields.
fn expand_struct_unnamed_fields(fields: &FieldsUnnamed) -> TokenStream {
    let (deconstructor, serialize) = serialize_unnamed_fields(fields);

    quote! {
        let Self(#deconstructor) = self;

        #serialize

        if __arr.is_empty() {
            Ok(serde_lite::Intermediate::None)
        } else if __arr.len() == 1 {
            Ok(__arr.pop().unwrap())
        } else {
            Ok(serde_lite::Intermediate::Array(__arr))
        }
    }
}

/// Expand Serialize for a given enum.
fn expand_for_enum(
    name: Ident,
    generics: Generics,
    data: DataEnum,
    attrs: &[Attribute],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if data.variants.is_empty() {
        panic!("enum with no variants cannot be serialized")
    }

    let mut serialize_variants = TokenStream::new();

    for variant in &data.variants {
        serialize_variants.extend(serialize_enum_variant(variant));
    }

    let mut serialize = quote! {
        let (__tag, __content) = match self {
            #serialize_variants
        };
    };

    if let Some(tag) = attributes::get_enum_tag(attrs) {
        if let Some(content) = attributes::get_enum_content(attrs) {
            serialize.extend(expand_adjacently_tagged_enum(&tag, &content));
        } else {
            serialize.extend(expand_internally_tagged_enum(&tag));
        }
    } else {
        serialize.extend(expand_externally_tagged_enum());
    }

    quote! {
        #[allow(unused_variables)]
        impl #impl_generics serde_lite::Serialize for #name #ty_generics #where_clause {
            fn serialize(&self) -> Result<serde_lite::Intermediate, serde_lite::Error> {
                #serialize
            }
        }
    }
}

/// Expand Serialize for an internally tagged enum.
fn expand_internally_tagged_enum(tag: &str) -> TokenStream {
    let ltag = Literal::string(tag);

    quote! {
        if __content.is_none() {
            let mut __map = serde_lite::Map::with_capacity(1);

            __map.insert(String::from(#ltag), serde_lite::Intermediate::String(__tag));

            return Ok(serde_lite::Intermediate::Map(__map));
        } else if let serde_lite::Intermediate::Map(__map) = __content {
            let mut __res = serde_lite::Map::with_capacity(__map.len() + 1);

            __res.insert(String::from(#ltag), serde_lite::Intermediate::String(__tag));
            __res.extend(__map);

            return Ok(serde_lite::Intermediate::Map(__res));
        } else if let serde_lite::Intermediate::Array(__arr) = __content {
            if __arr.is_empty() {
                let mut __map = serde_lite::Map::with_capacity(1);

                __map.insert(String::from(#ltag), serde_lite::Intermediate::String(__tag));

                return Ok(serde_lite::Intermediate::Map(__map));
            }
        }

        Err(serde_lite::Error::custom("enum cannot be tagged internally"))
    }
}

/// Expand Serialize for an adjacently tagged enum.
fn expand_adjacently_tagged_enum(tag: &str, content: &str) -> TokenStream {
    let ltag = Literal::string(tag);
    let lcont = Literal::string(content);

    quote! {
        let mut __map = serde_lite::Map::with_capacity(2);

        __map.insert(String::from(#ltag), serde_lite::Intermediate::String(__tag));
        __map.insert(String::from(#lcont), __content);

        Ok(serde_lite::Intermediate::Map(__map))
    }
}

/// Expand serialize for an externally tagged enum.
fn expand_externally_tagged_enum() -> TokenStream {
    quote! {
        if __content.is_none() {
            return Ok(serde_lite::Intermediate::String(__tag));
        } else if let serde_lite::Intermediate::Map(__map) = &__content {
            if __map.is_empty() {
                return Ok(serde_lite::Intermediate::String(__tag));
            }
        } else if let serde_lite::Intermediate::Array(__arr) = &__content {
            if __arr.is_empty() {
                return Ok(serde_lite::Intermediate::String(__tag));
            }
        }

        let mut __map = serde_lite::Map::with_capacity(1);

        __map.insert(__tag, __content);

        Ok(serde_lite::Intermediate::Map(__map))
    }
}

/// Generate code for serializing a given enum variant.
fn serialize_enum_variant(variant: &Variant) -> TokenStream {
    match &variant.fields {
        Fields::Named(fields) => serialize_struct_enum_variant(variant, fields),
        Fields::Unnamed(fields) => serialize_tuple_enum_variant(variant, fields),
        Fields::Unit => serialize_unit_enum_variant(variant),
    }
}

/// Generate code for serializing a given struct-like enum variant.
fn serialize_struct_enum_variant(variant: &Variant, fields: &FieldsNamed) -> TokenStream {
    let (deconstructor, serialize) = serialize_named_fields(fields);

    let ident = &variant.ident;
    let sname = attributes::get_variant_name(&variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident { #deconstructor } => {
            #serialize

            let __tag = String::from(#lname);
            let __content = serde_lite::Intermediate::Map(__map);

            (__tag, __content)
        }
    }
}

/// Generate code for serializing a given tuple-like enum variant.
fn serialize_tuple_enum_variant(variant: &Variant, fields: &FieldsUnnamed) -> TokenStream {
    let (deconstructor, serialize) = serialize_unnamed_fields(fields);

    let ident = &variant.ident;
    let sname = attributes::get_variant_name(&variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident ( #deconstructor ) => {
            #serialize

            let __tag = String::from(#lname);

            if __arr.is_empty() {
                (__tag, serde_lite::Intermediate::None)
            } else if __arr.len() == 1 {
                (__tag, __arr.pop().unwrap())
            } else {
                (__tag, serde_lite::Intermediate::Array(__arr))
            }
        }
    }
}

/// Generate code for serializing a given enum variant.
fn serialize_unit_enum_variant(variant: &Variant) -> TokenStream {
    let ident = &variant.ident;
    let sname = attributes::get_variant_name(&variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident => {
            let __tag = String::from(#lname);
            let __content = serde_lite::Intermediate::None;

            (__tag, __content)
        }
    }
}

/// Generate code for serializing given named field.
fn serialize_named_fields(fields: &FieldsNamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();

    let len = Literal::usize_unsuffixed(fields.named.len());

    let mut serialize = quote! {
        let mut __map = serde_lite::Map::with_capacity(#len);
        let mut __field_errors = Vec::new();
    };

    for field in &fields.named {
        let name = &field.ident;
        let sname = attributes::get_field_name(field);
        let lname = Literal::string(&sname);
        let skip = attributes::has_flag(&field.attrs, "skip")
            || attributes::has_flag(&field.attrs, "skip_serializing");

        deconstructor.extend(quote! {
            #name,
        });

        let serialize_field = if skip {
            continue;
        } else if attributes::has_flag(&field.attrs, "flatten") {
            quote! {
                match #name.serialize() {
                    Ok(serde_lite::Intermediate::Map(inner)) => __map.extend(inner),
                    Ok(_) => return Err(serde_lite::Error::custom(
                        concat!("field \"", stringify!(#name), "\" cannot be flattened")
                    )),
                    Err(serde_lite::Error::NamedFieldErrors(errors)) => {
                        __field_errors.extend(errors);
                    }
                    Err(err) => return Err(err),
                }
            }
        } else {
            quote! {
                match #name.serialize() {
                    Ok(v) => {
                        __map.insert(String::from(#lname), v);
                    }
                    Err(err) => __field_errors.push((String::from(#lname), err)),
                }
            }
        };

        if let Some(path) = attributes::get_skip_field_serializing_if(field) {
            let path = TokenStream::from_str(&path)
                .expect("invalid path given for the skip_serializing_if attribute");

            serialize.extend(quote! {
                if !#path(#name) {
                    #serialize_field
                }
            });
        } else {
            serialize.extend(serialize_field);
        }
    }

    serialize.extend(quote! {
        if !__field_errors.is_empty() {
            return Err(serde_lite::Error::NamedFieldErrors(__field_errors));
        }
    });

    (deconstructor, serialize)
}

/// Generate code for serializing given unnamed fields.
fn serialize_unnamed_fields(fields: &FieldsUnnamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();
    let mut serialize = TokenStream::new();

    if !fields.unnamed.is_empty() {
        let len = Literal::usize_unsuffixed(fields.unnamed.len());

        serialize.extend(quote! {
            let mut __arr = Vec::with_capacity(#len);
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

        serialize.extend(quote! {
            match #name.serialize() {
                Ok(v) => __arr.push(v),
                Err(err) => __field_errors.push((#lindex, err)),
            }
        });
    }

    match fields.unnamed.len() {
        0 => (),
        1 => serialize.extend(quote! {
            if let Some((_, err)) = __field_errors.pop() {
                return Err(err);
            }
        }),
        _ => serialize.extend(quote! {
            if !__field_errors.is_empty() {
                return Err(serde_lite::Error::UnnamedFieldErrors(__field_errors));
            }
        }),
    }

    (deconstructor, serialize)
}
