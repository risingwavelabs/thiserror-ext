#![feature(error_generic_member_access)]

pub mod inner {
    use thiserror::Error;
    use thiserror_ext_derive::{Box, Macro};

    #[derive(Error, Debug, Macro, Box)]
    #[thiserror_ext(type = BoxMyError)]
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
}

mod tests {
    use crate::inner::{BoxMyError, MyError};

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
    fn test_bail() -> Result<(), anyhow::Error> {
        use crate::inner::bail_quux;

        match 1 + 1 {
            2 => Ok(()),
            _ => bail_quux!("1 + 1 != 2"),
        }
    }
}
