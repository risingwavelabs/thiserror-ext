use expand::DeriveType;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod thiserror;

#[proc_macro_derive(Construct, attributes(thiserror_ext))]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive(&input, DeriveType::Construct)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(ResultExt, attributes(thiserror_ext))]
pub fn derive_result_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive(&input, DeriveType::ResultExt)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Box, attributes(thiserror_ext))]
pub fn derive_box(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive(&input, DeriveType::Box)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
