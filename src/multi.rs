use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{report::with_indent_adv, AsDyn, AsReport, Report};

pub struct MultiError<E: ?Sized = dyn Error + Send + Sync + 'static>(Vec<Box<E>>);

impl<E> Debug for MultiError<E>
where
    E: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else if self.0.len() == 1 {
            Debug::fmt(self.0.first().unwrap(), f)
        } else {
            f.debug_tuple("MultiError").field(&self.0).finish()
        }
    }
}

impl<E> Display for MultiError<E>
where
    E: ?Sized + AsDyn + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else if self.0.len() == 1 {
            Display::fmt(self.0.first().unwrap(), f)
        } else {
            f.write_str("Multiple errors occured\n")?;
            with_indent_adv(2, |curr, _| {
                for (i, error) in self.0.iter().enumerate() {
                    for _ in 0..curr {
                        f.write_str(" ")?;
                    }
                    write!(f, "* {}", Report(error.as_dyn()))?;
                    if i != self.0.len() - 1 {
                        f.write_str("\n")?;
                    }
                }
                Ok(())
            })
        }
    }
}

impl<E> Error for MultiError<E>
where
    E: ?Sized + AsDyn + Display + Debug,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if self.0.len() == 1 {
            self.0.first().unwrap().as_dyn().source()
        } else {
            None
        }
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        for error in &self.0 {
            error.as_dyn().provide(request);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use anyhow::anyhow;
    use expect_test::{expect, Expect};

    use crate::{multi::MultiError, AsDyn, AsReport};

    fn do_test(err: impl Error, expect: (Expect, Expect, Expect, Expect)) {
        expect.0.assert_eq(&format!("{}", err));
        expect.1.assert_eq(&format!("{:?}", err));
        expect.2.assert_eq(&format!("{}", err.as_report()));
        expect.3.assert_eq(&format!("{:#}", err.as_report()));
    }

    #[test]
    fn test() {
        let err: MultiError =
            MultiError(vec![anyhow!("foo").context("context").into(), "bar".into()]);

        do_test(
            err,
            (
                expect![[r#"
                    Multiple errors occured
                    * context: foo
                    * bar"#]],
                expect![[r#"
                    MultiError([context

                    Caused by:
                        foo, "bar"])"#]],
                expect![[r#"
                    Multiple errors occured
                    * context: foo
                    * bar"#]],
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
                expect![[r#"
                    Multiple errors occured
                    * context: baz
                    * Multiple errors occured
                      * context: foo
                      * bar"#]],
                expect![[r#"
                    MultiError([context

                    Caused by:
                        baz, MultiError([context

                    Caused by:
                        foo, "bar"])])"#]],
                expect![[r#"
                    Multiple errors occured
                    * context: baz
                    * Multiple errors occured
                      * context: foo
                      * bar"#]],
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
                expect![[r#"
                    Error { context: "outer error", source: Multiple errors occured
                    * context: foo
                    * bar }"#]],
                expect![[r#"
                    outer error: Multiple errors occured
                    * context: foo
                    * bar"#]],
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
                        Multiple errors occured
                        * context: foo
                        * bar }"#]],
                expect![[r#"
                    outer error: middle error: Multiple errors occured
                    * context: foo
                    * bar"#]],
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
                        Multiple errors occured
                        * context: baz
                        * Multiple errors occured
                          * context: foo
                          * bar }"#]],
                expect![[r#"
                    outer error: middle error: Multiple errors occured
                    * context: baz
                    * Multiple errors occured
                      * context: foo
                      * bar"#]],
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
}
