#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]

use thiserror::Error;
use thiserror_ext::{Box, ReportDebug};

#[derive(Error, ReportDebug, Default)]
#[error("inner")]
struct Inner;

#[derive(Error, ReportDebug, Default, Box)]
#[thiserror_ext(newtype(name = BoxOuter))]
#[error("outer")]
struct Outer {
    #[source]
    inner: Inner,
}

#[test]
fn test_report_debug() {
    let error = Outer::default();

    expect_test::expect!["outer: inner"].assert_eq(&format!("{:?}", error));

    expect_test::expect![[r#"
    outer

    Caused by:
      inner
"#]]
    .assert_eq(&format!("{:#?}", error));

    let boxed = BoxOuter::from(error);

    expect_test::expect!["outer: inner"].assert_eq(&format!("{:?}", boxed));
}

#[test]
#[should_panic]
fn test_unwrap() {
    let error = Outer::default();
    let _ = Err::<(), _>(error).unwrap();
}

#[test]
#[should_panic]
fn test_expect() {
    let error = Outer::default();
    let _ = Err::<(), _>(error).expect("intentional panic");
}
