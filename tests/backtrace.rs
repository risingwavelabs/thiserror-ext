#![feature(error_generic_member_access)]

use std::backtrace::Backtrace;

use sealed_test::prelude::*;
use thiserror::Error;
use thiserror_ext_derive::Box;

#[derive(Error, Debug)]
#[error("..")]
struct ParseFloatErrorWithBacktrace {
    #[from]
    source: std::num::ParseFloatError,
    #[backtrace]
    backtrace: Backtrace,
}

#[derive(Error, Debug, Box)]
#[thiserror_ext(newtype(name = MyError, backtrace))]
enum MyErrorInner {
    #[error("parse int")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },

    #[error("parse float with backtrace")]
    ParseFloatWithBacktrace {
        #[from]
        #[backtrace] // inner error has backtrace, provide it
        source: ParseFloatErrorWithBacktrace,
    },
}

fn parse_float_with_backtrace(input: &str) -> Result<f32, MyError> {
    fn parse_inner(input: &str) -> Result<f32, ParseFloatErrorWithBacktrace> {
        Ok(input.parse()?) // backtrace captured here
    }

    Ok(parse_inner(input)?) // already has backtrace, no need to capture
}

fn parse_int(input: &str) -> Result<i32, MyError> {
    fn parse_inner(input: &str) -> Result<i32, std::num::ParseIntError> {
        input.parse() // no backtrace captured here
    }

    Ok(parse_inner(input)?) // backtrace captured here by generated `MyError`
}

#[sealed_test(env = [("RUST_BACKTRACE", "1")])]
fn test_with_source_backtrace() {
    let error = parse_float_with_backtrace("not a number").unwrap_err();
    let backtrace = std::error::request_ref::<Backtrace>(&error)
        .unwrap()
        .to_string();

    assert!(backtrace.contains("parse_inner"), "{backtrace}");
}

#[sealed_test(env = [("RUST_BACKTRACE", "1")])]
fn test_without_source_backtrace() {
    let error = parse_int("not a number").unwrap_err();
    let backtrace = std::error::request_ref::<Backtrace>(&error)
        .unwrap()
        .to_string();

    assert!(!backtrace.contains("parse_inner"), "{backtrace}");
}
