//! Useful extension utilities for [`thiserror`](https://docs.rs/thiserror).
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

#![cfg_attr(feature = "provide", feature(error_generic_member_access))]
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export the alloc crate for use within derived code.
#[doc(hidden)]
pub extern crate alloc;

mod as_dyn;
mod backtrace;
mod ptr;
mod report;

pub use as_dyn::AsDyn;
pub use report::{AsReport, Report};
pub use thiserror_ext_derive::*;

#[doc(hidden)]
pub mod __private {
    pub use crate::backtrace::AlwaysBacktrace;
    #[cfg(feature = "provide")]
    pub use crate::backtrace::MaybeBacktrace;
    pub use crate::backtrace::NoExtraBacktrace;
    pub use crate::ptr::ErrorArc;
    pub use crate::ptr::ErrorBox;
}

macro_rules! for_dyn_error_types {
    ($macro:ident) => {
        $macro! {
            { dyn core::error::Error },
            { dyn core::error::Error + core::marker::Send },
            { dyn core::error::Error + core::marker::Sync },
            { dyn core::error::Error + core::marker::Send + core::marker::Sync },
            { dyn core::error::Error + core::marker::Send + core::marker::Sync + core::panic::UnwindSafe },
        }
    };
}
pub(crate) use for_dyn_error_types;

pub(crate) mod error_sealed {
    pub trait Sealed {}

    impl<T: core::error::Error> Sealed for T {}

    macro_rules! impl_sealed {
        ($({$ty:ty },)*) => {
            $(
                impl Sealed for $ty {}
            )*
        };
    }
    for_dyn_error_types! { impl_sealed }
}
