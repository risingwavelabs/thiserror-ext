pub mod inner {
    use thiserror::Error;
    use thiserror_ext_derive::Macro;

    #[derive(Error, Debug, Macro)]
    pub enum MyError {
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
    }
}

mod tests {
    use std::num::NonZeroI32;

    use crate::inner::MyError;
    use crate::inner::{bar, baz, foo, qux};

    #[test]
    fn test() {
        let _ = foo!("hello {}", 42);

        let _ = bar!("hello {}", 42);
        let _ = bar!(issue = Some(42), "hello {}", 42);
        let _ = bar!(issue = 42, "hello {}", 42);
        let _ = bar!(issue = None, "hello {}", 42);

        let _ = baz!("hello {}", 42);
        let _ = baz!(issue = 42, pr = Some(88), "hello {}", 42);
        let _ = baz!(issue = None, pr = 88, "hello {}", 42);
        let _ = baz!(issue = 42, "hello {}", 42);
        let _ = baz!(pr = 88, "hello {}", 42);
        let _ = baz!(pr = Some(88), "hello {}", 42);

        let _ = qux!(extra = "233", "hello {}", 42);
        let _ = qux!(extra = "233".to_owned(), "hello {}", 42);
        // let _ = qux!("hello {}", 42);
    }
}
