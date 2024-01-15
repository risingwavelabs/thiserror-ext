//! This example demonstrates how to achieve the similar functionality as
//! [`anyhow::Context`] with `thiserror_ext`, in a type-safer manner.

#![feature(error_generic_member_access)]

use thiserror::Error;
use thiserror_ext::{AsReport, Box, ContextInto, Macro};

#[derive(Error, Macro, Box, ContextInto, Debug)]
#[thiserror_ext(newtype(name = MyError))]
enum MyErrorKind {
    #[error("{0}")]
    EvaluationFailed(#[message] String),

    #[error("failed to evaluate expression `{expr}`")]
    Context {
        #[source]
        inner: MyError,
        expr: String,
    },
}

fn eval_add() -> Result<(), MyError> {
    bail_evaluation_failed!("not supported")
}

fn eval() -> Result<(), MyError> {
    eval_add().into_context("add")
}

fn main() {
    let err = eval().unwrap_err();

    assert_eq!(
        err.to_report_string(),
        "failed to evaluate expression `add`: not supported"
    );
}
