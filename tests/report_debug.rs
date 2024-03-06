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
    let debug = format!("{:?}", Outer::default());
    expect_test::expect![[r#"
        outer

        Caused by:
          inner
    "#]]
    .assert_eq(&debug);
}
