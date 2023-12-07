# `thiserror-ext`

[![Crate](https://img.shields.io/crates/v/thiserror-ext.svg)](https://crates.io/crates/thiserror-ext)
[![Docs](https://docs.rs/thiserror-ext/badge.svg)](https://docs.rs/thiserror-ext)

Useful extension utilities for `thiserror`. See the [documentation](https://docs.rs/thiserror-ext) for more details.

```rust
#[derive(
    Debug,
    thiserror::Error,
    thiserror_ext::Box,
    thiserror_ext::Construct,
    thiserror_ext::ContextInto,
    thiserror_ext::Macro,
)]
#[thiserror_ext(
    newtype(name = Error, backtrace),
    macro(path = "crate::foo"),
)]
enum ErrorKind {
    #[error("cannot parse int from `{from}`")]
    Parse {
        source: std::num::ParseIntError,
        from: String,
    },

    #[error("not yet implemented: {msg}")]
    NotImplemented {
        issue: Option<i32>,
        #[message] msg: String,
    },

    #[error("internal error: {0}")]
    Internal(String),
}

// `thiserror_ext::Construct`
let error: Error = Error::internal("oops");

// `thiserror_ext::Box`
assert_eq!(std::mem::size_of::<Error>(), std::mem::size_of::<usize>());
let bt: &Backtrace = std::error::request_ref(&error).unwrap();

// `thiserror_ext::ContextInto`
let result: Result<i32, Error> = "foo".parse().into_parse("foo");

// `thiserror_ext::AsReport`
//
// "cannot parse int from `foo`: invalid digit found in string"
println!("{}", result.unwrap_err().as_report());

// `thiserror_ext::Macro`
bail_not_implemented!(issue = 42, "an {} feature", "awesome");
```
