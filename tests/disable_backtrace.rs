#![feature(error_generic_member_access)]
#![feature(assert_matches)]

use std::{
    assert_matches::assert_matches,
    backtrace::{Backtrace, BacktraceStatus},
};

use sealed_test::prelude::*;
use thiserror::Error;
use thiserror_ext::{Box, DisableBacktrace};

#[derive(Error, Debug)]
#[error("inner")]
struct Inner;

#[derive(Error, Debug, Box)]
#[thiserror_ext(newtype(name = BoxOuter, backtrace))]
#[error("outer")]
enum Outer {
    Disabled(
        #[from]
        #[backtrace]
        DisableBacktrace<Inner>,
    ),

    DisabledAnyhow(
        #[from]
        #[backtrace]
        DisableBacktrace<anyhow::Error>,
    ),

    Normal(
        #[from]
        #[backtrace]
        Inner,
    ),
}

#[sealed_test(env = [("RUST_BACKTRACE", "1")])]
fn test_disable_backtrace() {
    let e = DisableBacktrace(Inner);

    let bt = std::error::request_ref::<Backtrace>(&e).unwrap();
    assert_matches!(bt.status(), BacktraceStatus::Disabled);
}

#[sealed_test(env = [("RUST_BACKTRACE", "1")])]
fn test_disable_backtrace_nested() {
    // `BoxOuter::Normal`: captured
    {
        let e = Inner;
        let e = BoxOuter::from(e);

        let bt = std::error::request_ref::<Backtrace>(&e).unwrap();
        assert_matches!(bt.status(), BacktraceStatus::Captured);
    }

    // `BoxOuter::Disabled`: disabled
    {
        let e = DisableBacktrace(Inner);
        let e = BoxOuter::from(e);

        let bt = std::error::request_ref::<Backtrace>(&e).unwrap();
        assert_matches!(bt.status(), BacktraceStatus::Disabled);
    }
}
