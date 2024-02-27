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
#[should_panic]
fn test() {
    Err::<(), _>(Outer::default()).unwrap();
}
