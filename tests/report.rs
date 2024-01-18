#![feature(error_generic_member_access)]

use expect_test::expect;
use sealed_test::prelude::*;
use thiserror::Error;
use thiserror_ext::AsReport;

#[derive(Error, Debug)]
#[error("inner error")]
struct Inner {}

#[derive(Error, Debug)]
#[error("middle error: {source}")] // some error may include source message in its message
                                   // the suffix should be cleaned up
struct Middle {
    #[from]
    source: Inner,
    #[backtrace]
    backtrace: Option<std::backtrace::Backtrace>,
}

#[derive(Error, Debug)]
#[error("{source}")] // some may duplicate source message to emulate `#[transparent]`
                     // the whole message should be cleaned up (as it's empty after removing source message)
struct MiddleTransparent {
    #[from]
    #[backtrace]
    source: Middle,
}

#[derive(Error, Debug)]
#[error("outer error")] // but the best practice is to not include
struct Outer {
    #[from]
    #[backtrace]
    source: MiddleTransparent,
}

fn inner() -> Result<(), Inner> {
    Err(Inner {})
}

fn middle(bt: bool) -> Result<(), Middle> {
    inner().map_err(|e| {
        if bt {
            Middle::from(e)
        } else {
            Middle {
                source: e,
                backtrace: None,
            }
        }
    })?;
    Ok(())
}

fn middle_transparent(bt: bool) -> Result<(), MiddleTransparent> {
    middle(bt)?;
    Ok(())
}

fn outer(bt: bool) -> Result<(), Outer> {
    middle_transparent(bt)?;
    Ok(())
}

#[test]
fn test_report_display() {
    let expect = expect!["outer error: middle error: inner error"];
    expect.assert_eq(&format!("{}", outer(true).unwrap_err().as_report()));
}

#[test]
fn test_report_display_alternate() {
    let expect = expect![[r#"
        outer error

        Caused by these errors (recent errors listed first):
         1: middle error
         2: inner error
    "#]];
    expect.assert_eq(&format!("{:#}", outer(true).unwrap_err().as_report()));
}

#[test]
fn test_report_display_alternate_single_source() {
    let expect = expect![[r#"
        middle error

        Caused by:
            inner error
    "#]];
    expect.assert_eq(&format!("{:#}", middle(true).unwrap_err().as_report()));
}

// Show that there's extra backtrace information compared to `Display`.
// Backtrace is intentionally disabled to make the test deterministic.
#[sealed_test(env = [("RUST_BACKTRACE", "0")])]
fn test_report_debug() {
    let expect = expect![[r#"
        outer error: middle error: inner error

        Backtrace:
        disabled backtrace
    "#]];
    expect.assert_eq(&format!("{:?}", outer(true).unwrap_err().as_report()));
}

// If there's no backtrace, the behavior should be exactly the same as `Display`.
#[test]
fn test_report_debug_no_backtrace() {
    let expect = expect!["outer error: middle error: inner error"];
    expect.assert_eq(&format!("{:?}", outer(false).unwrap_err().as_report()));
}

// Show that there's extra backtrace information compared to `Display`.
// Backtrace is intentionally disabled to make the test deterministic.
#[sealed_test(env = [("RUST_BACKTRACE", "0")])]
fn test_report_debug_alternate() {
    let expect = expect![[r#"
        outer error

        Caused by these errors (recent errors listed first):
         1: middle error
         2: inner error

        Backtrace:
        disabled backtrace
    "#]];
    expect.assert_eq(&format!("{:#?}", outer(true).unwrap_err().as_report()));
}

// If there's no backtrace, the behavior should be exactly the same as `Display`.
#[test]
fn test_report_debug_alternate_no_backtrace() {
    let expect = expect![[r#"
        outer error

        Caused by these errors (recent errors listed first):
         1: middle error
         2: inner error
    "#]];
    expect.assert_eq(&format!("{:#?}", outer(false).unwrap_err().as_report()));
}
