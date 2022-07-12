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

        Ok(__val)
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

            __map.insert_with_str_key(#ltag, serde_lite::Intermediate::String(String::from(__tag)));

            Ok(serde_lite::Intermediate::Map(__map))
        } else if let serde_lite::Intermediate::Map(__map) = __content {
            let mut __res = serde_lite::Map::with_capacity(__map.len() + 1);

            __res.insert_with_str_key(#ltag, serde_lite::Intermediate::String(String::from(__tag)));
            __res.extend(__map);

            Ok(serde_lite::Intermediate::Map(__res))
        } else {
            Err(serde_lite::Error::custom_static("enum cannot be tagged internally"))
        }
    }
}

/// Expand Serialize for an adjacently tagged enum.
fn expand_adjacently_tagged_enum(tag: &str, content: &str) -> TokenStream {
    let ltag = Literal::string(tag);
    let lcont = Literal::string(content);

    quote! {
        let mut __map = serde_lite::Map::with_capacity(2);

        __map.insert_with_str_key(#ltag, serde_lite::Intermediate::String(String::from(__tag)));
        __map.insert_with_str_key(#lcont, __content);

        Ok(serde_lite::Intermediate::Map(__map))
    }
}

/// Expand serialize for an externally tagged enum.
fn expand_externally_tagged_enum() -> TokenStream {
    quote! {
        if __content.is_none() {
            Ok(serde_lite::Intermediate::String(String::from(__tag)))
        } else {
            let mut __map = serde_lite::Map::with_capacity(1);

            __map.insert_with_str_key(__tag, __content);

            Ok(serde_lite::Intermediate::Map(__map))
        }
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
    let sname = attributes::get_variant_name(variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident { #deconstructor } => {
            #serialize

            (#lname, serde_lite::Intermediate::Map(__map))
        }
    }
}

/// Generate code for serializing a given tuple-like enum variant.
fn serialize_tuple_enum_variant(variant: &Variant, fields: &FieldsUnnamed) -> TokenStream {
    let (deconstructor, serialize) = serialize_unnamed_fields(fields);

    let ident = &variant.ident;
    let sname = attributes::get_variant_name(variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident ( #deconstructor ) => {
            #serialize

            (#lname, __val)
        }
    }
}

/// Generate code for serializing a given enum variant.
fn serialize_unit_enum_variant(variant: &Variant) -> TokenStream {
    let ident = &variant.ident;
    let sname = attributes::get_variant_name(variant);
    let lname = Literal::string(&sname);

    quote! {
        Self::#ident => {
            (#lname, serde_lite::Intermediate::None)
        }
    }
}

/// Generate code for serializing given named field.
fn serialize_named_fields(fields: &FieldsNamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();

    let len = Literal::usize_unsuffixed(fields.named.len());

    let mut serialize = quote! {
        let mut __map = serde_lite::Map::with_capacity(#len);
        let mut __field_errors = serde_lite::ErrorList::new();
    };

    for field in &fields.named {
        let name = &field.ident;
        let ty = &field.ty;
        let sname = attributes::get_field_name(field);
        let lname = Literal::string(&sname);
        let serializer = attributes::get_field_serializer(field)
            .map(|path| TokenStream::from_str(&path))
            .map(|res| res.expect("invalid path given for the serialize_with attribute"))
            .unwrap_or_else(|| {
                quote! {
                    <#ty as serde_lite::Serialize>::serialize
                }
            });
        let skip = attributes::has_flag(&field.attrs, "skip")
            || attributes::has_flag(&field.attrs, "skip_serializing");

        deconstructor.extend(quote! {
            #name,
        });

        let serialize_field = if skip {
            continue;
        } else if attributes::has_flag(&field.attrs, "flatten") {
            quote! {
                match #serializer(#name) {
                    Ok(serde_lite::Intermediate::Map(inner)) => __map.extend(inner),
                    Ok(_) => return Err(serde_lite::Error::custom_static(
                        concat!("field \"", stringify!(#name), "\" cannot be flattened")
                    )),
                    Err(serde_lite::Error::NamedFieldErrors(errors)) => {
                        __field_errors.append(errors);
                    }
                    Err(err) => return Err(err),
                }
            }
        } else {
            quote! {
                match #serializer(#name) {
                    Ok(v) => __map.insert_with_str_key(#lname, v),
                    Err(err) => {
                        __field_errors.push(serde_lite::NamedFieldError::new_static(#lname, err));
                    }
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
    match fields.unnamed.len() {
        0 => serialize_unnamed_fields_0(),
        1 => serialize_unnamed_fields_1(),
        _ => serialize_unnamed_fields_n(fields),
    }
}

/// Generate code for serializing given unnamed fields where the actual number
/// of fields is zero (e.g. zero-length tuple struct).
fn serialize_unnamed_fields_0() -> (TokenStream, TokenStream) {
    let deconstructor = TokenStream::new();

    let mut serialize = TokenStream::new();

    serialize.extend(quote! {
        let __val = serde_lite::Intermediate::None;
    });

    (deconstructor, serialize)
}

/// Generate code for serializing given unnamed fields where the actual number
/// of fields is one (e.g. single-element tuple struct).
fn serialize_unnamed_fields_1() -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();
    let mut serialize = TokenStream::new();

    let name = Ident::new("f0", Span::call_site());

    deconstructor.extend(quote! {
        #name
    });

    serialize.extend(quote! {
        let __val = #name.serialize()?;
    });

    (deconstructor, serialize)
}

/// Generate code for serializing given unnamed fields where the actual number
/// of fields is greater one (e.g. multiple-element tuple struct).
fn serialize_unnamed_fields_n(fields: &FieldsUnnamed) -> (TokenStream, TokenStream) {
    let mut deconstructor = TokenStream::new();
    let mut serialize = TokenStream::new();

    if !fields.unnamed.is_empty() {
        let len = Literal::usize_unsuffixed(fields.unnamed.len());

        serialize.extend(quote! {
            let mut __arr = Vec::with_capacity(#len);
            let mut __field_errors = serde_lite::ErrorList::new();
        });
    }

    for (index, _) in fields.unnamed.iter().enumerate() {
        let sname = format!("f{}", index);
        let name = Ident::new(&sname, Span::call_site());
        let lindex = Literal::usize_unsuffixed(index);

        deconstructor.extend(quote! {
            #name,
        });

        serialize.extend(quote! {
            match #name.serialize() {
                Ok(v) => __arr.push(v),
                Err(err) => __field_errors.push(serde_lite::UnnamedFieldError::new(#lindex, err)),
            }
        });
    }

    serialize.extend(quote! {
        if !__field_errors.is_empty() {
            return Err(serde_lite::Error::UnnamedFieldErrors(__field_errors));
        }

        let __val = serde_lite::Intermediate::Array(__arr);
    });

    (deconstructor, serialize)
}
