use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    report::{in_pretty_report, with_indent_adv},
    AsDyn, Report,
};

pub struct MultiError<E: ?Sized = dyn Error + Send + Sync + 'static>(
    /* TODO: make it private */ pub Vec<Box<E>>,
);

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
            f.write_str("Multiple errors occured")?;
            if in_pretty_report() || f.alternate() {
                with_indent_adv(2, |curr, _| {
                    f.write_str("\n")?;
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
            } else {
                f.write_str(": ")?;
                for (i, error) in self.0.iter().enumerate() {
                    write!(f, "[{}]", Report(error.as_dyn()))?;
                    if i != self.0.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                Ok(())
            }
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
