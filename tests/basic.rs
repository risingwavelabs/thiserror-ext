#![feature(error_generic_member_access)]

use std::backtrace::Backtrace;
use thiserror::*;
use thiserror_ext::*;

#[derive(Error, Debug, Construct, ContextInto, Box)]
#[thiserror_ext(newtype(name = MyError))]
pub enum MyErrorInner {
    #[error("cannot parse int from `{from}`")]
    Parse {
        #[source]
        error: std::num::ParseIntError,
        from: String,
    },

    #[error("cannot parse int from `{from}`")]
    ParseImplicitSource {
        source: std::num::ParseIntError,
        from: String,
    },

    #[error("cannot parse int")]
    ParseUnnamed(#[source] std::num::ParseFloatError, String),

    #[error("cannot parse int")]
    ParseWithBacktrace {
        #[source]
        error: std::num::ParseIntError,
        backtrace: Backtrace,
    },

    #[error("cannot parse int from `{from}`")]
    ParseWithBacktraceAndContext {
        #[source]
        error: std::num::ParseIntError,
        backtrace: Backtrace,
        from: String,
    },

    #[error(transparent)]
    IoTransparent(std::io::Error),

    #[error("unsupported: {0}")]
    UnsupportedSingleField(String),

    #[error("bad id: {0}")]
    #[construct(skip)]
    BadId(String),
}

impl MyError {
    pub fn bad_id(id: impl ToString) -> Self {
        MyErrorInner::BadId(id.to_string()).into()
    }
}

#[test]
fn test() {}
