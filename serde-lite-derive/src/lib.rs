mod attributes;
mod deserialize;
mod serialize;
mod update;

use proc_macro::TokenStream;

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    serialize::derive_serialize(input)
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    deserialize::derive_deserialize(input)
}

#[proc_macro_derive(Update, attributes(serde))]
pub fn derive_update(input: TokenStream) -> TokenStream {
    update::derive_update(input)
}
