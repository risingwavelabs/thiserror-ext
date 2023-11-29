use expand::{DeriveCtorType, DeriveNewType};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod thiserror;

#[proc_macro_derive(Construct, attributes(thiserror_ext))]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_ctor(&input, DeriveCtorType::Construct)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(ContextInto, attributes(thiserror_ext))]
pub fn derive_context_into(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_ctor(&input, DeriveCtorType::ContextInto)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Box, attributes(thiserror_ext))]
pub fn derive_box(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_new_type(&input, DeriveNewType::Box)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Arc, attributes(thiserror_ext))]
pub fn derive_arc(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_new_type(&input, DeriveNewType::Arc)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Macro, attributes(thiserror_ext, message))]
pub fn derive_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand::derive_macro(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
