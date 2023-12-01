#![feature(error_generic_member_access)]

use std::{error::Error, num::ParseIntError};

use thiserror::*;
use thiserror_ext::*;

#[derive(Error, Debug, Arc, Construct)]
#[thiserror_ext(newtype(name = SharedMyError))]
pub enum MyErrorInner {
    #[error("foo: {foo}")]
    Foo { source: ParseIntError, foo: String },
}

#[test]
fn test() {
    let error = SharedMyError::foo("nope".parse::<i32>().unwrap_err(), "hello".to_owned());
    let error2 = error.clone();

    // Test source preserved.
    let source = error2.source().unwrap();
    assert_eq!(source.to_string(), "invalid digit found in string");
}
