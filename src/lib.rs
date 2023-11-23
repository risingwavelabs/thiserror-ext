#![feature(error_generic_member_access)] // TODO: it's nightly-only

mod as_dyn;
mod backtrace;
mod error_box;
mod report;

pub use as_dyn::AsDyn;
pub use report::{AsReport, Report};
pub use thiserror_ext_derive::*;

#[doc(hidden)]
pub mod __private {
    pub use crate::backtrace::{MaybeBacktrace, NoBacktrace};
    pub use crate::error_box::ErrorBox;
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
