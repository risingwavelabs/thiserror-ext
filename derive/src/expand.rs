use either::{for_both, Either};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    spanned::Spanned, DeriveInput, GenericArgument, Ident, LitStr, Member, PathArguments, Result,
    Type, Visibility,
};

use crate::thiserror::ast::{Field, Input, Variant};

struct Args {
    other_args: Vec<TokenStream>,
    source_arg: Option<TokenStream>,
    ctor_args: Vec<TokenStream>,
}

enum SourceInto {
    Yes,
    No,
}

fn resolve_variant_args(variant: &Variant<'_>, source_into: SourceInto) -> Args {
    let mut other_args = Vec::new();
    let mut source_arg = None;
    let mut ctor_args = Vec::new();

    for (i, field) in variant.fields.iter().enumerate() {
        let ty = &field.ty;
        let member = &field.member;

        let name = match &field.member {
            Member::Named(named) => named.clone(),
            Member::Unnamed(_) => {
                if field.is_non_from_source() {
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
        } else if field.is_non_from_source() {
            match source_into {
                SourceInto::Yes => {
                    source_arg = Some(quote!(#name: impl Into<#ty>,));
                    ctor_args.push(quote!(#member: #name.into(),));
                }
                SourceInto::No => {
                    source_arg = Some(quote!(#name: #ty,));
                    ctor_args.push(quote!(#member: #name,));
                }
            }
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

struct MacroArgs {
    other_args: Vec<TokenStream>,
    other_call_args: Vec<TokenStream>,
    ctor_args: Vec<TokenStream>,
}

fn resolve_args_for_macro(fields: &[Field<'_>]) -> MacroArgs {
    let mut other_args = Vec::new();
    let mut other_call_args = Vec::new();
    let mut ctor_args = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let ty = &field.ty;
        let member = &field.member;

        let name = match &field.member {
            Member::Named(named) => named.clone(),
            Member::Unnamed(_) => format_ident!("arg_{}", i),
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
        } else if field.is_message() {
            ctor_args.push(quote!(#member: ::std::format!($($fmt_arg)*).into(),));
        } else {
            other_args.push(quote!(#name = $#name:expr,));
            other_call_args.push(quote!(#name));
            ctor_args.push(quote!(#member: $#name,));
        }
    }

    MacroArgs {
        other_args,
        other_call_args,
        ctor_args,
    }
}

struct DeriveMeta {
    impl_type: Ident,
    backtrace: bool,
    macro_mangle: bool,
    macro_path: Option<TokenStream>,
}

fn resolve_meta(input: &DeriveInput) -> Result<DeriveMeta> {
    let mut impl_type = None;
    let mut backtrace = false;
    let mut macro_mangle = false;
    let mut macro_path = None;

    for attr in &input.attrs {
        if attr.path().is_ident("thiserror_ext") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    let value = meta.value()?;
                    impl_type = Some(value.parse()?);
                } else if meta.path.is_ident("backtrace") {
                    backtrace = true;
                } else if meta.path.is_ident("macro") {
                    meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("mangle") {
                            macro_mangle = true;
                        } else if meta.path.is_ident("path") {
                            let value = meta.value()?;
                            let path: LitStr = value.parse()?;
                            let mut path = path.value();

                            if path.starts_with("crate") {
                                path.insert(0, '$');
                                if !path.ends_with("::") {
                                    path.push_str("::");
                                }
                                macro_path = Some(path.parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    meta.path,
                                    "macro path should start with `crate`",
                                ));
                            }
                        }
                        Ok(())
                    })?;
                }

                Ok(())
            })?;
        }
    }
    let impl_type = impl_type.unwrap_or_else(|| input.ident.clone());

    Ok(DeriveMeta {
        impl_type,
        backtrace,
        macro_mangle,
        macro_path,
    })
}

pub enum DeriveCtorType {
    Construct,
    ContextInto,
}

pub enum DeriveNewType {
    Box,
    Arc,
}

impl DeriveNewType {
    fn name(&self) -> &'static str {
        match self {
            DeriveNewType::Box => "Box",
            DeriveNewType::Arc => "Arc",
        }
    }

    fn ty_ident(&self) -> Ident {
        match self {
            DeriveNewType::Box => format_ident!("ErrorBox"),
            DeriveNewType::Arc => format_ident!("ErrorArc"),
        }
    }
}

pub fn derive_new_type(input: &DeriveInput, ty: DeriveNewType) -> Result<TokenStream> {
    let input_type = input.ident.clone();
    let vis = &input.vis;

    let DeriveMeta {
        impl_type,
        backtrace,
        ..
    } = resolve_meta(input)?;

    if impl_type == input_type {
        return Err(syn::Error::new_spanned(
            input,
            format!("should specify a different type for `{}` derive with `#[thiserror_ext(type = <type>)]`", ty.name()),
        ));
    }

    let backtrace_type_param = if backtrace {
        quote!(thiserror_ext::__private::MaybeBacktrace)
    } else {
        quote!(thiserror_ext::__private::NoExtraBacktrace)
    };

    let doc = format!("The `{}`-wrapped type of [`{}`].", ty.name(), input_type);
    let new_type = ty.ty_ident();
    let extra_derive = match ty {
        DeriveNewType::Box => quote!(),
        DeriveNewType::Arc => quote!(Clone),
    };

    let into_inner = match ty {
        DeriveNewType::Box => quote!(
            #[doc = "Consumes `self` and returns the inner error."]
            #vis fn into_inner(self) -> #input_type {
                self.0.into_inner()
            }
        ),
        DeriveNewType::Arc => quote!(),
    };

    let generated = quote!(
        #[doc = #doc]
        #[derive(thiserror_ext::__private::thiserror::Error, Debug, #extra_derive)]
        #[error(transparent)]
        #vis struct #impl_type(
            #[from]
            #[backtrace]
            thiserror_ext::__private::#new_type<
                #input_type,
                #backtrace_type_param,
            >,
        );

        // For `?` to work.
        impl<E> From<E> for #impl_type
        where
            E: Into<#input_type>,
        {
            fn from(error: E) -> Self {
                Self(thiserror_ext::__private::#new_type::new(error.into()))
            }
        }

        impl #impl_type {
            #[doc = "Returns the reference to the inner error."]
            #vis fn inner(&self) -> &#input_type {
                self.0.inner()
            }

            #into_inner
        }
    );

    Ok(generated)
}

pub fn derive_ctor(input: &DeriveInput, t: DeriveCtorType) -> Result<TokenStream> {
    let input_type = input.ident.clone();
    let vis = &input.vis;

    let DeriveMeta { impl_type, .. } = resolve_meta(input)?;

    let input = Input::from_syn(input)?;

    let input = match input {
        Input::Struct(input) => {
            return Err(syn::Error::new_spanned(
                input.original,
                "only `enum` is supported for `Construct` and `ContextInto`",
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

        let skipped = match t {
            DeriveCtorType::Construct => variant.attrs.construct_skip.is_some(),
            DeriveCtorType::ContextInto => variant.attrs.context_into_skip.is_some(),
        };
        if skipped {
            continue;
        }

        let variant_name = &variant.ident;

        let Args {
            other_args,
            source_arg,
            ctor_args,
        } = resolve_variant_args(
            &variant,
            match t {
                DeriveCtorType::Construct => SourceInto::Yes,
                DeriveCtorType::ContextInto => SourceInto::No,
            },
        );

        let ctor_expr = quote!(#input_type::#variant_name {
            #(#ctor_args)*
        });

        let item = match t {
            DeriveCtorType::Construct => {
                let ctor_name = format_ident!(
                    "{}",
                    big_camel_case_to_snake_case(&variant_name.to_string()),
                    span = variant.original.span()
                );
                let doc = format!("Constructs a [`{input_type}::{variant_name}`] variant.");

                quote!(
                    #[doc = #doc]
                    #vis fn #ctor_name(#source_arg #(#other_args)*) -> Self {
                        #ctor_expr.into()
                    }
                )
            }
            DeriveCtorType::ContextInto => {
                // It's implemented on `Result<T, SourceError>`, so there's must be the `source` field,
                // and we expect there's at least one argument.
                if source_arg.is_none() || other_args.is_empty() {
                    continue;
                }
                let source_ty = variant.source_field().unwrap().ty;
                let source_ty_name = get_type_string(source_ty);

                let ext_name =
                    format_ident!("Into{}", variant_name, span = variant.original.span());
                let method_name = format_ident!(
                    "into_{}",
                    big_camel_case_to_snake_case(&variant_name.to_string()),
                    span = variant.original.span()
                );
                let doc_trait = format!(
                    "Extension trait for converting [`{source_ty_name}`] \
                     into [`{input_type}::{variant_name}`] with the given context.",
                );
                let doc_method = format!(
                    "Converts [`{source_ty_name}`] \
                     into [`{input_type}::{variant_name}`] with the given context.",
                );

                quote!(
                    #[doc = #doc_trait]
                    #vis trait #ext_name {
                        type Ret;
                        #[doc = #doc_method]
                        fn #method_name(self, #(#other_args)*) -> Self::Ret;
                    }
                    impl<__T> #ext_name for std::result::Result<__T, #source_ty> {
                        type Ret = std::result::Result<__T, #impl_type>;
                        fn #method_name(self, #(#other_args)*) -> Self::Ret {
                            self.map_err(|#source_arg| #ctor_expr.into())
                        }
                    }
                    impl #ext_name for #source_ty {
                        type Ret = #impl_type;
                        fn #method_name(self, #(#other_args)*) -> Self::Ret {
                            (|#source_arg| #ctor_expr.into())(self)
                        }
                    }
                )
            }
        };

        items.push(item);
    }

    let generated = match t {
        DeriveCtorType::Construct => {
            quote!(
                #[automatically_derived]
                impl #impl_type {
                    #(#items)*
                }
            )
        }
        DeriveCtorType::ContextInto => {
            quote!(#(#items)*)
        }
    };

    Ok(generated)
}

pub fn derive_macro_inner(input: &DeriveInput, bail: bool) -> Result<TokenStream> {
    let input_type = input.ident.clone();
    let vis = &input.vis;

    let DeriveMeta {
        impl_type,
        macro_mangle,
        macro_path,
        ..
    } = resolve_meta(input)?;

    let input = Input::from_syn(input)?;

    let variants = match input {
        Input::Struct(input) => vec![Either::Left(input)],
        Input::Enum(input) => input.variants.into_iter().map(Either::Right).collect(),
    };

    let mut items = Vec::new();

    for variant in variants {
        // We only care about variants with `message` field and no `source` or `from` field.
        if for_both!(&variant, v => v.message_field()).is_none()
            || for_both!(&variant, v => v.source_field()).is_some()
        {
            continue;
        }

        let variant_name = for_both!(&variant, v => &v.ident);
        let ctor_path = match &variant {
            Either::Left(_s) => quote!(#macro_path #input_type),
            Either::Right(_v) => quote!(#macro_path #input_type::#variant_name),
        };

        let fields = for_both!(&variant, v => &v.fields);

        let MacroArgs {
            other_args,
            other_call_args,
            ctor_args,
        } = resolve_args_for_macro(fields);

        let ctor_expr = quote!(#ctor_path {
            #(#ctor_args)*
        });

        let bail_prefix = if bail { "bail_" } else { "" };
        let bail_suffix = if bail { "__bail" } else { "" };

        let ctor_span = for_both!(&variant, v => v.ident.span());

        let export_name = format_ident!(
            "{}{}",
            bail_prefix,
            big_camel_case_to_snake_case(&variant_name.to_string()),
            span = ctor_span,
        );
        let mangled_name = if macro_mangle {
            format_ident!(
                "__thiserror_ext_macro__{}__{}{}",
                big_camel_case_to_snake_case(&input_type.to_string()),
                big_camel_case_to_snake_case(&variant_name.to_string()),
                bail_suffix,
                span = ctor_span,
            )
        } else {
            export_name.clone()
        };

        let bail_doc = if bail { " and bails out" } else { "" };
        let doc = match &variant {
            Either::Left(_s) => {
                format!("Constructs a [`{input_type}`]{bail_doc}.")
            }
            Either::Right(_v) => {
                format!("Constructs a [`{input_type}::{variant_name}`] variant{bail_doc}.")
            }
        };

        let mut arms = Vec::new();

        let len = other_args.len();

        let message_arg = quote!($($fmt_arg:tt)*);
        let message_call_arg = quote!($($fmt_arg)*);

        for bitset in (0..(1 << len)).rev() {
            let mut args = Vec::new();
            let mut call_args = Vec::new();
            for (i, (arg, call_arg)) in (other_args.iter()).zip(other_call_args.iter()).enumerate()
            {
                if bitset & (1 << i) != 0 {
                    args.push(arg);
                    call_args.push(quote!(#call_arg = $#call_arg.into(),));
                } else {
                    call_args.push(quote!(#call_arg = ::std::default::Default::default(),));
                }
            }

            let arm = quote!(
                (#(#args)* #message_arg) => {
                    #export_name!(@ #(#call_args)* #message_call_arg)
                };
            );
            arms.push(arm);
        }

        let full_inner = if bail {
            quote!({
                let res: #macro_path #impl_type = (#ctor_expr).into();
                return ::std::result::Result::Err(res.into());
            })
        } else {
            quote!({
                let res: #macro_path #impl_type = (#ctor_expr).into();
                res
            })
        };

        let full = quote!(
            (@ #(#other_args)* #message_arg) => {
                #full_inner
            };
        );

        let macro_export = if let Visibility::Public(_) = &vis {
            quote!(#[macro_export])
        } else {
            quote!()
        };

        let item = quote!(
            #[doc = #doc]
            #[allow(unused_macros)]
            #macro_export
            macro_rules! #mangled_name {
                #full
                #(#arms)*
            }

            #[allow(unused_imports)]
            #vis use #mangled_name as #export_name;
        );

        items.push(item);
    }

    let generated = quote!(
        #( #items )*
    );

    Ok(generated)
}

pub fn derive_macro(input: &DeriveInput) -> Result<TokenStream> {
    let ctor = derive_macro_inner(input, false)?;
    let bail = derive_macro_inner(input, true)?;

    let generated = quote!(
        #ctor
        #bail
    );

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

fn get_type_string(type_: &Type) -> String {
    let tokens = type_.to_token_stream();
    let mut type_string = String::new();

    for token in tokens {
        let stringified = token.to_string();
        type_string.push_str(&stringified);
    }

    type_string
}
