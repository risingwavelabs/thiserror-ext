use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, GenericArgument, Member, PathArguments, Result, Type};

mod thiserror;

#[proc_macro_derive(Constructor, attributes(thiserror_ext))]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(&input, DeriveType::Constructor)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(ResultExt, attributes(thiserror_ext))]
pub fn derive_result_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(&input, DeriveType::ResultExt)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Box, attributes(thiserror_ext))]
pub fn derive_box(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(&input, DeriveType::Box)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

struct Args {
    other_args: Vec<TokenStream2>,
    source_arg: Option<TokenStream2>,
    ctor_args: Vec<TokenStream2>,
}

fn resolve(variant: &thiserror::ast::Variant<'_>) -> Args {
    let mut other_args = Vec::new();
    let mut source_arg = None;
    let mut ctor_args = Vec::new();

    for (i, field) in variant.fields.iter().enumerate() {
        let ty = &field.ty;
        let member = &field.member;

        let name = match &field.member {
            Member::Named(named) => named.clone(),
            Member::Unnamed(_) => {
                if field.attrs.source.is_some() {
                    format_ident!("source")
                } else {
                    format_ident!("arg_{}", i)
                }
            }
        };

        if field.is_backtrace() {
            let expr = if type_is_option(ty) {
                quote!(std::option::Option::Some(
                    std::backtrace::Backtrace::capture()
                ))
            } else {
                quote!(std::convert::From::from(
                    std::backtrace::Backtrace::capture()
                ))
            };
            ctor_args.push(quote!(#member: #expr,))
        } else if field.attrs.source.is_some() {
            source_arg = Some(quote!(#name: #ty,));
            ctor_args.push(quote!(#member: #name,));
        } else {
            other_args.push(quote!(#name: impl Into<#ty>,));
            ctor_args.push(quote!(#member: #name.into(),));
        }
    }

    Args {
        other_args,
        source_arg,
        ctor_args,
    }
}

enum DeriveType {
    Constructor,
    ResultExt,
    Box,
}

fn expand(input: &DeriveInput, t: DeriveType) -> Result<TokenStream2> {
    use thiserror::ast::Input;

    let input_type = input.ident.clone();

    let mut impl_type = None;
    for attr in &input.attrs {
        if attr.path().is_ident("thiserror_ext") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    let value = meta.value()?;
                    impl_type = Some(value.parse()?);
                }

                Ok(())
            })?;
        }
    }
    let impl_type = impl_type.unwrap_or_else(|| input_type.clone());

    let vis = &input.vis;

    if let DeriveType::Box = t {
        if impl_type == input_type {
            return Err(syn::Error::new_spanned(
                input,
                "should specify a different type for `Box` derive with `#[thiserror_ext(type = <type>)]`",
            ));
        }

        let generated = quote!(
            #[derive(thiserror_ext::__private::thiserror::Error, Debug)]
            #[error(transparent)]
            #vis struct #impl_type(
                #[from]
                #[backtrace]
                pub thiserror_ext::__private::ErrorBox<#input_type>,
            );

            // For `?` to work.
            impl<E> From<E> for #impl_type
            where
                E: Into<#input_type>,
            {
                fn from(error: E) -> Self {
                    Self(thiserror_ext::__private::ErrorBox::new(error.into()))
                }
            }
        );

        return Ok(generated);
    }

    let input = Input::from_syn(input)?;

    let input = match input {
        Input::Struct(input) => {
            return Err(syn::Error::new_spanned(
                input.original,
                "only `enum` is supported for `thiserror_ext`",
            ))
        }
        Input::Enum(input) => input,
    };

    let mut items = Vec::new();

    for variant in input.variants {
        let Some(source_field) = variant.source_field() else {
            continue;
        };
        if source_field.attrs.from.is_some() {
            continue;
        }

        let variant_name = &variant.ident;
        let source_ty = &source_field.ty;

        let Args {
            other_args,
            source_arg,
            ctor_args,
        } = resolve(&variant);

        let ctor_expr = quote!(#input_type::#variant_name {
            #(#ctor_args)*
        });

        let item = match t {
            DeriveType::Constructor => {
                let ctor_name = format_ident!(
                    "{}",
                    big_camel_case_to_snake_case(&variant_name.to_string())
                );

                quote!(
                    #vis fn #ctor_name(#source_arg #(#other_args)*) -> Self {
                        #ctor_expr.into()
                    }
                )
            }
            DeriveType::ResultExt => {
                let ext_name = format_ident!("{}ResultExt", variant_name);

                quote!(
                    #vis trait #ext_name<__T> {
                        fn context(self, #(#other_args)*) -> std::result::Result<__T, #impl_type>;
                    }
                    impl<__T> #ext_name<__T> for std::result::Result<__T, #source_ty> {
                        fn context(self, #(#other_args)*) -> std::result::Result<__T, #impl_type> {
                            self.map_err(|#source_arg| #ctor_expr.into())
                        }
                    }
                )
            }
            DeriveType::Box => unreachable!(),
        };

        items.push(item);
    }

    let generated = match t {
        DeriveType::Constructor => {
            quote!(
                #[automatically_derived]
                impl #impl_type {
                    #(#items)*
                }
            )
        }
        DeriveType::ResultExt => {
            quote!(#(#items)*)
        }
        DeriveType::Box => unreachable!(),
    };

    Ok(generated)
}

fn big_camel_case_to_snake_case(input: &str) -> String {
    let mut output = String::new();

    for (i, c) in input.char_indices() {
        if i == 0 {
            output.push(c.to_ascii_lowercase());
        } else if c.is_uppercase() {
            output.push('_');
            output.push(c.to_ascii_lowercase());
        } else {
            output.push(c);
        }
    }

    output
}

fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}
