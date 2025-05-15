#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]
#![allow(dead_code)]

use thiserror::Error;

// Follow the convention for opting out of the field named `source` by specifying `r#source`.
// https://github.com/dtolnay/thiserror/pull/350
mod opt_out_field_named_message_for_macro {
    use super::*;
    use std::fmt;
    use thiserror_ext::Macro;

    #[derive(Error, Debug, Macro)]
    enum Error {
        #[error(fmt = demo_fmt)]
        Demo {
            code: u16,
            r#message: Option<String>,
        },
    }

    fn demo_fmt(
        code: &u16,
        message: &Option<String>,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{code}")?;
        if let Some(msg) = message {
            write!(formatter, " - {msg}")?;
        }
        Ok(())
    }

    // This shows that we don't generate a macro named `bail_demo` with `derive(Macro)`.
    #[allow(unused_macros)]
    #[macro_export]
    macro_rules! bail_demo {
        () => {};
    }
}

#[test]
fn test() {}
