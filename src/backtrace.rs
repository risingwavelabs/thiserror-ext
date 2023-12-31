use std::backtrace::Backtrace;

/// Provides backtrace to the error.
pub trait WithBacktrace {
    /// Capture backtrace based on whether the error already has one.
    fn capture(inner: &dyn std::error::Error) -> Self;

    /// Provide the backtrace, if any.
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>);
}

/// Do not capture extra backtrace.
#[derive(Clone, Copy)]
pub struct NoExtraBacktrace;

/// Capture backtrace if the error does not already have one.
pub struct MaybeBacktrace(Option<Backtrace>);

impl WithBacktrace for NoExtraBacktrace {
    fn capture(_inner: &dyn std::error::Error) -> Self {
        Self
    }

    fn provide<'a>(&'a self, _request: &mut std::error::Request<'a>) {}
}

impl WithBacktrace for MaybeBacktrace {
    fn capture(inner: &dyn std::error::Error) -> Self {
        let inner = if std::error::request_ref::<Backtrace>(inner).is_none() {
            Some(Backtrace::capture())
        } else {
            None
        };
        Self(inner)
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        if let Some(backtrace) = &self.0 {
            request.provide_ref(backtrace);
        }
    }
}
