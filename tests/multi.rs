use std::error::Error;

use anyhow::anyhow;
use expect_test::{expect, Expect};

use thiserror_ext::{AsDyn, AsReport, MultiError};

fn do_test(err: impl Error, expect: (Expect, Expect, Expect, Expect)) {
    expect.0.assert_eq(&format!("{}", err));
    expect.1.assert_eq(&format!("{:?}", err));
    expect.2.assert_eq(&format!("{}", err.as_report()));
    expect.3.assert_eq(&format!("{:#}", err.as_report()));
}

#[test]
fn test() {
    let err: MultiError = MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);

    do_test(
        err,
        (
            expect!["Multiple errors occured: [context: foo], [bar]"],
            expect![[r#"
                    MultiError([context

                    Caused by:
                        foo, "bar"])"#]],
            expect!["Multiple errors occured: [context: foo], [bar]"],
            expect![[r#"
                    Multiple errors occured
                    * context: foo
                    * bar"#]],
        ),
    );
}

#[test]
fn test_nested() {
    let inner: MultiError =
        MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);
    let outer: MultiError = MultiError(vec![
        anyhow!("baz").context("context").into(),
        Box::new(inner),
    ]);

    do_test(
            outer,
            (
                expect!["Multiple errors occured: [context: baz], [Multiple errors occured: [context: foo], [bar]]"],
                expect![[r#"
                    MultiError([context

                    Caused by:
                        baz, MultiError([context

                    Caused by:
                        foo, "bar"])])"#]],
                expect!["Multiple errors occured: [context: baz], [Multiple errors occured: [context: foo], [bar]]"],
                expect![[r#"
                    Multiple errors occured
                    * context: baz
                    * Multiple errors occured
                      * context: foo
                      * bar"#]],
            ),
        );
}

#[test]
fn test_source_depth_1() {
    let source: MultiError =
        MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);
    // let err = anyhow!(source).context("middle error");
    let err = anyhow!(source).context("outer error");

    do_test(
        err.as_dyn(),
        (
            expect!["outer error"],
            expect![[
                r#"Error { context: "outer error", source: Multiple errors occured: [context: foo], [bar] }"#
            ]],
            expect!["outer error: Multiple errors occured: [context: foo], [bar]"],
            expect![[r#"
                    outer error

                    Caused by:
                        Multiple errors occured
                        * context: foo
                        * bar
                "#]],
        ),
    );
}

#[test]
fn test_source_depth_2() {
    let source: MultiError =
        MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);
    let err = anyhow!(source).context("middle error");
    let err = err.context("outer error");

    do_test(
        err.as_dyn(),
        (
            expect!["outer error"],
            expect![[r#"
                    Error { context: "outer error", source: middle error

                    Caused by:
                        Multiple errors occured: [context: foo], [bar] }"#]],
            expect!["outer error: middle error: Multiple errors occured: [context: foo], [bar]"],
            expect![[r#"
                    outer error

                    Caused by these errors (recent errors listed first):
                     1: middle error
                     2: Multiple errors occured
                        * context: foo
                        * bar
                "#]],
        ),
    );
}

#[test]
fn test_nested_source() {
    let inner: MultiError =
        MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);
    let outer: MultiError = MultiError(vec![
        anyhow!("baz").context("context").into(),
        Box::new(inner),
    ]);

    let err = anyhow!(outer).context("middle error");
    let err = err.context("outer error");

    do_test(
            err.as_dyn(),
            (
                expect!["outer error"],
                expect![[r#"
                    Error { context: "outer error", source: middle error

                    Caused by:
                        Multiple errors occured: [context: baz], [Multiple errors occured: [context: foo], [bar]] }"#]],
                expect!["outer error: middle error: Multiple errors occured: [context: baz], [Multiple errors occured: [context: foo], [bar]]"],
                expect![[r#"
                    outer error

                    Caused by these errors (recent errors listed first):
                     1: middle error
                     2: Multiple errors occured
                        * context: baz
                        * Multiple errors occured
                          * context: foo
                          * bar
                "#]],
            ),
        );
}
