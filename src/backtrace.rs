/// Provides backtrace to the error.
pub trait WithBacktrace {
    /// Capture backtrace based on whether the error already has one.
    fn capture(inner: &dyn core::error::Error) -> Self;

    /// Get the captured backtrace, if any.
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace>;

    #[cfg(feature = "provide")]
    /// Provide the backtrace, if any.
    fn provide<'a>(&'a self, request: &mut core::error::Request<'a>);
}

/// Do not capture extra backtrace.
#[derive(Clone, Copy)]
pub struct NoExtraBacktrace;

impl WithBacktrace for NoExtraBacktrace {
    fn capture(_inner: &dyn core::error::Error) -> Self {
        Self
    }

    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        None
    }

    #[cfg(feature = "provide")]
    fn provide<'a>(&'a self, _request: &mut core::error::Request<'a>) {}
}

mod captured {
    use super::WithBacktrace;
    use std::backtrace::Backtrace;

    /// Always capture a new backtrace.
    pub struct CapturedBacktrace(Backtrace);

    impl WithBacktrace for CapturedBacktrace {
        fn capture(_inner: &dyn core::error::Error) -> Self {
            Self(Backtrace::capture())
        }

        fn backtrace(&self) -> Option<&Backtrace> {
            Some(&self.0)
        }

        #[cfg(feature = "provide")]
        fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
            request.provide_ref(&self.0);
        }
    }
}

pub use captured::CapturedBacktrace;

#[cfg(feature = "provide")]
mod maybe {
    use super::WithBacktrace;
    use std::backtrace::Backtrace;

    /// Capture backtrace if the error does not already have one.
    pub struct MaybeBacktrace(Option<Backtrace>);

    impl WithBacktrace for MaybeBacktrace {
        fn capture(inner: &dyn core::error::Error) -> Self {
            let inner = if core::error::request_ref::<Backtrace>(inner).is_none() {
                Some(Backtrace::capture())
            } else {
                None
            };
            Self(inner)
        }

        fn backtrace(&self) -> Option<&Backtrace> {
            self.0.as_ref()
        }

        fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
            if let Some(backtrace) = &self.0 {
                request.provide_ref(backtrace);
            }
        }
    }
}

#[cfg(feature = "provide")]
pub use maybe::MaybeBacktrace;
