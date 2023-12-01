#![feature(error_generic_member_access)]

use expect_test::expect;
use sealed_test::prelude::*;

#[derive(thiserror::Error, Debug, thiserror_ext::Box)]
#[thiserror_ext(newtype(name = MyError, backtrace, report_debug))]
pub enum MyErrorInner {
    #[error("bad id: {0}")]
    BadId(String),
}

#[sealed_test(env = [("RUST_BACKTRACE", "0")])]
fn test_newtype_report_debug() {
    let error: MyError = MyErrorInner::BadId("233".to_owned()).into();

    let expect = expect![[r#"
        bad id: 233

        Backtrace:
        disabled backtrace
    "#]];
    expect.assert_eq(&format!("{:?}", error));
}
