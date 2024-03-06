use thiserror::Error;
use thiserror_ext::ReportDebug;

#[derive(Error, ReportDebug, Default)]
#[error("inner")]
struct Inner;

#[derive(Error, ReportDebug, Default)]
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
