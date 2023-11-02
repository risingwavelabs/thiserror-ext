#![feature(error_generic_member_access)] // TODO: it's nightly-only

mod error_box;
mod report;

pub use report::{AsReport, Report};
pub use thiserror_ext_derive::{Box, Construct, ContextInto};

#[doc(hidden)]
pub mod __private {
    pub use crate::error_box::ErrorBox;
    pub use thiserror;
}
