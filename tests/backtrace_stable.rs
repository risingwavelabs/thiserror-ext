#![cfg(all(feature = "backtrace", not(feature = "provide")))]

use thiserror::Error;
use thiserror_ext_derive::Box;

#[derive(Error, Debug, Box)]
#[thiserror_ext(newtype(name = StableBacktraceError, backtrace))]
enum StableBacktraceErrorInner {
    #[error("parse int")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },
}

fn parse_int(input: &str) -> Result<i32, StableBacktraceError> {
    fn parse_inner(input: &str) -> Result<i32, std::num::ParseIntError> {
        input.parse()
    }

    Ok(parse_inner(input)?)
}

#[test]
fn test_backtrace_method_available_on_stable() {
    let error = parse_int("not a number").unwrap_err();
    assert!(error.backtrace().is_some());
}
