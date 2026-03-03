#![cfg(feature = "nightly")]
#![feature(error_generic_member_access)]

use std::error::Error;

use thiserror::*;
use thiserror_ext::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ErrorCode(i32);

#[derive(Error, Debug, Arc, Construct)]
#[thiserror_ext(newtype(name = TestError, extra_provide = Self::my_extra_provide))]
enum TestErrorInner {
    #[error("custom error")]
    Custom(ErrorCode),
    #[error("parse error")]
    ParseError,
}

impl TestError {
    fn my_extra_provide(&self, request: &mut std::error::Request<'_>) {
        match self.inner() {
            TestErrorInner::Custom(error_code) => request.provide_value(*error_code),
            TestErrorInner::ParseError => request.provide_value(ErrorCode(42)),
        };
    }
}

fn request_error_code(error: &dyn Error) -> ErrorCode {
    std::error::request_value(error).unwrap()
}

#[test]
fn test_extra_provide() {
    let error_1 = TestError::custom(ErrorCode(114514));
    let error_2 = TestError::parse_error();

    assert_eq!(request_error_code(&error_1), ErrorCode(114514));
    assert_eq!(request_error_code(&error_2), ErrorCode(42));
}
