//! This example demonstrates how to achieve the similar functionality as
//! [`anyhow::Context`] with `thiserror_ext`, in a type-safer manner.

#![feature(error_generic_member_access)]

use expect_test::expect;
use thiserror::Error;
use thiserror_ext::{AsReport, ContextInto};

#[derive(Error, Debug)]
#[error("foo")]
struct FooError;

#[derive(Error, Debug)]
#[error("bar")]
struct BarError;

#[derive(Error, ContextInto, Debug)]
enum MyError {
    #[error("{context}")]
    Foo {
        #[source]
        foo: FooError,
        context: String,
    },

    #[error("{context1} && {context2}")]
    Bar {
        #[source]
        bar: BarError,
        context1: String,
        context2: Box<str>,
    },
}

fn foo() -> Result<(), FooError> {
    Err(FooError)
}

fn bar() -> Result<(), BarError> {
    Err(BarError)
}

#[test]
fn test_result_into() {
    let err: MyError = foo().into_foo("hello").unwrap_err();
    expect!["hello: foo"].assert_eq(&err.to_report_string());

    let err: MyError = bar().into_bar("hello", "world").unwrap_err();
    expect!["hello && world: bar"].assert_eq(&err.to_report_string());
}

#[test]
fn test_result_into_with() {
    let err: MyError = foo().into_foo_with(|| "hello").unwrap_err();
    expect!["hello: foo"].assert_eq(&err.to_report_string());

    let err: MyError = bar()
        .into_bar_with(|| ("hello", format!("wo{}", "rld")))
        .unwrap_err();
    expect!["hello && world: bar"].assert_eq(&err.to_report_string());
}

#[test]
fn test_error_into() {
    let err: MyError = FooError.into_foo("hello");
    expect!["hello: foo"].assert_eq(&err.to_report_string());

    let err: MyError = BarError.into_bar("hello", "world");
    expect!["hello && world: bar"].assert_eq(&err.to_report_string());
}

#[test]
fn test_error_into_with() {
    let err: MyError = FooError.into_foo_with(|| "hello");
    expect!["hello: foo"].assert_eq(&err.to_report_string());

    let err: MyError = BarError.into_bar_with(|| ("hello", format!("wo{}", "rld")));
    expect!["hello && world: bar"].assert_eq(&err.to_report_string());
}
