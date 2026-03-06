#![cfg_attr(feature = "nightly", feature(error_generic_member_access))]

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
    fn fail_with<E>(error: E) -> Result<(), E> {
        if std::hint::black_box(true) {
            Err(error)
        } else {
            Ok(())
        }
    }

    fail_with(error).unwrap();
}

#[test]
#[should_panic]
fn test_expect() {
    let error = Outer::default();
    fn fail_with<E>(error: E) -> Result<(), E> {
        if std::hint::black_box(true) {
            Err(error)
        } else {
            Ok(())
        }
    }

    fail_with(error).expect("intentional panic");
}
