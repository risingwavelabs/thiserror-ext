#![feature(error_generic_member_access)]

pub mod inner {
    use thiserror::Error;
    use thiserror_ext_derive::{Box, Macro};

    #[derive(Error, Debug, Macro, Box)]
    #[thiserror_ext(newtype(name = BoxMyError))]
    pub(super) enum MyError {
        #[error("foo {message}")]
        Foo { message: String },

        #[error("bar {message}")]
        Bar { issue: Option<i32>, message: String },

        #[error("baz {msg}")]
        Baz {
            issue: Option<i32>,
            pr: Option<i32>,
            #[message]
            msg: String,
        },

        #[error("qux {msg}")]
        Qux {
            extra: String,
            #[message]
            msg: Box<str>,
        },

        #[error("quux {message}")]
        Quux { message: String },
    }
    #[derive(Error, Debug, Macro)]
    #[error("not implemented: {message}, issue: {issue:?}")]
    #[thiserror_ext(macro(mangle, path = "crate::inner", vis = pub(super)))]
    pub struct NotImplemented {
        pub issue: Option<i32>,
        pub message: String,
    }
}

mod tests {
    use thiserror_ext::AsReport;

    use crate::inner::{BoxMyError, MyError, NotImplemented};

    #[test]
    fn test() {
        use crate::inner::{bar, baz, foo, qux};

        let _: BoxMyError = foo!("hello {}", 42);

        let _ = bar!("hello {}", 42);
        let _ = bar!(issue = 42, "hello {}", 42);
        let _ = bar!(issue = None, "hello {}", 42);
        let a = bar!(issue = Some(42), "hello {}", 42);
        assert!(
            matches!(a.inner(), MyError::Bar { issue: Some(42), message } if message == "hello 42")
        );

        let _ = baz!("hello {}", 42);
        let _ = baz!(issue = 42, pr = Some(88), "hello {}", 42);
        let _ = baz!(issue = None, pr = 88, "hello {}", 42);
        let _ = baz!(issue = 42, "hello {}", 42);
        let a = baz!(pr = 88, "hello {}", 42);
        let _ = baz!(pr = Some(88), "hello {}", 42);
        assert!(matches!(
            a.inner(),
            MyError::Baz {
                pr: Some(88),
                issue: None,
                ..
            }
        ));

        let _ = qux!(extra = "233", "hello {}", 42);
        let _ = qux!(extra = "233".to_owned(), "hello {}", 42);
        let a = qux!("hello {}", 42); // use default `extra`
        assert!(matches!(
            a.inner(),
            MyError::Qux {
                extra,
                msg,
                ..
            } if extra == "" && msg.as_ref() == "hello 42"
        ));
    }

    #[test]
    fn test_bail() {
        use crate::inner::bail_quux;

        fn test() -> Result<(), anyhow::Error> {
            match 1 + 1 {
                3 => Ok(()),
                _ => bail_quux!("1 + 1 != 3"),
            }
        }

        let error = test().unwrap_err();
        assert!(matches!(
            error.downcast_ref::<BoxMyError>().unwrap().inner(),
            MyError::Quux { message } if message == "1 + 1 != 3"
        ));
    }

    #[test]
    fn test_struct() {
        use crate::inner::bail_not_implemented;
        use crate::inner::not_implemented;

        // As we're mangling the macro name, we can't use the macro directly.
        //
        // use crate::__thiserror_ext_macro__not_implemented__not_implemented;
        // use crate::not_implemented;

        let a: NotImplemented = not_implemented!(issue = 42, "hello {}", 42);
        assert!(matches!(
            a,
            NotImplemented {
                issue: Some(42),
                message,
            } if message == "hello 42"
        ));

        fn test() -> Result<(), anyhow::Error> {
            match 1 + 1 {
                3 => Ok(()),
                _ => bail_not_implemented!(issue = 42, "1 + 1 != 3"),
            }
        }

        let error = test().unwrap_err();
        assert!(matches!(
            error.downcast_ref::<NotImplemented>().unwrap(),
            NotImplemented {
                issue: Some(42),
                message,
            } if message == "1 + 1 != 3"
        ));
    }

    #[test]
    fn test_error_as_message() {
        use crate::inner::not_implemented;

        let e = std::io::Error::new(std::io::ErrorKind::AddrInUse, "hello world");
        let a: NotImplemented = not_implemented!(issue = 42, e);

        expect_test::expect!["not implemented: hello world, issue: Some(42)"]
            .assert_eq(&a.to_report_string());
    }
}
