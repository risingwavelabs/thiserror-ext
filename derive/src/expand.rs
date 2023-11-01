use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, GenericArgument, Member, PathArguments, Result, Type};

use crate::thiserror::ast::{Input, Variant};

struct Args {
    other_args: Vec<TokenStream>,
    source_arg: Option<TokenStream>,
    ctor_args: Vec<TokenStream>,
}

fn resolve(variant: &Variant<'_>) -> Args {
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

pub enum DeriveType {
    Construct,
    ResultExt,
    Box,
}

pub fn derive(input: &DeriveInput, t: DeriveType) -> Result<TokenStream> {
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

        let doc = format!("The boxed type of [`{}`].", input_type);
        let generated = quote!(
            #[doc = #doc]
            #[derive(thiserror_ext::__private::thiserror::Error, Debug)]
            #[error(transparent)]
            #vis struct #impl_type(
                #[from]
                #[backtrace]
                thiserror_ext::__private::ErrorBox<#input_type>,
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

            impl #impl_type {
                #[doc = "Returns the reference to the inner error."]
                #vis fn inner(&self) -> &#input_type {
                    self.0.inner()
                }

                #[doc = "Consumes `self` and returns the inner error."]
                #vis fn into_inner(self) -> #input_type {
                    self.0.into_inner()
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
        // Why not directly use `From`?
        if variant.from_field().is_some() {
            continue;
        }

        let variant_name = &variant.ident;

        let Args {
            other_args,
            source_arg,
            ctor_args,
        } = resolve(&variant);

        let ctor_expr = quote!(#input_type::#variant_name {
            #(#ctor_args)*
        });

        let item = match t {
            DeriveType::Construct => {
                let ctor_name = format_ident!(
                    "{}",
                    big_camel_case_to_snake_case(&variant_name.to_string())
                );
                let doc = format!("Constructs a [`{input_type}::{variant_name}`] variant.");

                quote!(
                    #[doc = #doc]
                    #vis fn #ctor_name(#source_arg #(#other_args)*) -> Self {
                        #ctor_expr.into()
                    }
                )
            }
            DeriveType::ResultExt => {
                // It's implemented on `Result<T, SourceError>`, and we expect there's at
                // least one argument.
                if source_arg.is_none() || other_args.is_empty() {
                    continue;
                }
                let source_ty = variant.source_field().unwrap().ty;

                let ext_name = format_ident!("{}ResultExt", variant_name);
                let method_name = format_ident!(
                    "into_{}",
                    big_camel_case_to_snake_case(&variant_name.to_string())
                );
                let doc_trait = format!(
                    "Extension trait for [`Result`] with [`{impl_type}`] error type \
                     to convert into [`{input_type}::{variant_name}`] with given contexts.",
                );
                let doc_method = format!(
                    "Converts [`Result`] into [`{input_type}::{variant_name}`] \
                     with the given context.",
                );

                quote!(
                    #[doc = #doc_trait]
                    #vis trait #ext_name<__T> {
                        #[doc = #doc_method]
                        fn #method_name(self, #(#other_args)*) -> std::result::Result<__T, #impl_type>;
                    }
                    impl<__T> #ext_name<__T> for std::result::Result<__T, #source_ty> {
                        fn #method_name(self, #(#other_args)*) -> std::result::Result<__T, #impl_type> {
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
        DeriveType::Construct => {
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
