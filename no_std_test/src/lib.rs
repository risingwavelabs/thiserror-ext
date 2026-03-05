#![no_std]
#![allow(dead_code)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use thiserror::Error;
use thiserror_ext::{Box, Construct, ContextInto, Macro};

#[derive(Error, Debug, Construct, ContextInto, Box)]
#[thiserror_ext(newtype(name = MyError))]
pub enum MyErrorInner {
    #[error("cannot parse int from `{from}`")]
    Parse {
        #[source]
        error: core::num::ParseIntError,
        from: alloc::string::String,
    },

    #[error("cannot parse int from `{from}`")]
    ParseImplicitSource {
        source: core::num::ParseIntError,
        from: alloc::string::String,
    },

    #[error("cannot parse int")]
    ParseUnnamed(#[source] core::num::ParseFloatError, alloc::string::String),

    #[error(transparent)]
    FmtTransparent(core::fmt::Error),

    #[error("unsupported: {0}")]
    UnsupportedSingleField(alloc::string::String),

    #[error("bad id: {0}")]
    #[construct(skip)]
    BadId(alloc::string::String),
}

impl MyError {
    pub fn bad_id(id: impl ToString) -> Self {
        MyErrorInner::BadId(id.to_string()).into()
    }
}

#[derive(Debug, Error, Macro)]
enum MacroError {
    #[error("Test error: {msg}")]
    Test {
        #[message]
        msg: Box<str>,
    },
}

#[test]
pub fn test_no_std_macro_expand() {
    let err = test!("Test message");
    assert_eq!(err.to_string(), "Test error: Test message");
}
