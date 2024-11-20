//! Demonstrate that we can emulate a lightweight version of `anyhow` with `thiserror_ext`.

#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]

use thiserror::Error;
use thiserror_ext::Box;
use thiserror_ext_derive::Macro;

#[derive(Error, Debug, Box, Macro)]
#[thiserror_ext(newtype(name = Anyhow))]
#[error("{message}")]
struct AnyhowInner {
    source: Option<Anyhow>,
    message: Box<str>,
}

mod tests {
    use expect_test::expect;
    use thiserror_ext::AsReport;

    use super::*;

    #[test]
    fn test() {
        fn test() -> Result<(), Anyhow> {
            let a = anyhow!(); // empty input -> empty message -> ignored in report
            let b = anyhow!(source = a, "base");
            bail_anyhow!(source = b, "upper {}", 233);
        }

        let report = test().unwrap_err().to_report_string();
        expect!["upper 233: base"].assert_eq(&report);
    }
}
