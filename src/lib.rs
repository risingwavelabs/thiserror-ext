//! Useful extension utilities for [`thiserror`].
//!
//! ## Painless construction
//!
//! With derive macros of [`Construct`], [`ContextInto`] and [`Macro`],
//! one can construct an error in a much more convenient way, no matter it's
//! from scratch or converted from other errors.
//!
//! ## Better formatting
//!
//! With extension [`AsReport`], one can format an error in a pretty and
//! concise way, without losing any information from the error sources.
//!
//! ## Easier to interact with
//!
//! With derive macros of [`derive@Box`] and [`derive@Arc`], one can easily
//! wrap an `enum` error type into a new type, reducing the size to improve
//! performance, and automatically capturing backtraces if needed.

#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]

mod as_dyn;
mod backtrace;
mod ptr;
mod report;

pub use as_dyn::AsDyn;
pub use report::{AsReport, Report};
pub use thiserror_ext_derive::*;

#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "backtrace")]
    pub use crate::backtrace::MaybeBacktrace;
    pub use crate::backtrace::NoExtraBacktrace;
    pub use crate::ptr::{ErrorArc, ErrorBox};
    pub use thiserror;
}

macro_rules! for_dyn_error_types {
    ($macro:ident) => {
        $macro! {
            { dyn std::error::Error },
            { dyn std::error::Error + Send },
            { dyn std::error::Error + Sync },
            { dyn std::error::Error + Send + Sync },
            { dyn std::error::Error + Send + Sync + std::panic::UnwindSafe },
        }
    };
}
pub(crate) use for_dyn_error_types;

pub(crate) mod error_sealed {
    pub trait Sealed {}

    impl<T: std::error::Error> Sealed for T {}

    macro_rules! impl_sealed {
        ($({$ty:ty },)*) => {
            $(
                impl Sealed for $ty {}
            )*
        };
    }
    for_dyn_error_types! { impl_sealed }
}
