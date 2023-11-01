#![feature(error_generic_member_access)] // TODO: it's nightly-only

mod error_box;

pub use thiserror_ext_derive::*;

#[doc(hidden)]
pub mod __private {
    pub use crate::error_box::ErrorBox;
    pub use thiserror;
}
