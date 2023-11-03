use crate::backtrace::WithBacktrace;

/// Workaround for https://github.com/rust-lang/rust/issues/117432.
#[derive(Clone)]
#[repr(transparent)]
pub struct ErrorBox<T, B>(Box<(T, B)>);

impl<T: std::error::Error, B: WithBacktrace> ErrorBox<T, B> {
    pub fn new(t: T) -> Self {
        let backtrace = B::capture(&t);
        Self(Box::new((t, backtrace)))
    }
}

impl<T, B> ErrorBox<T, B> {
    fn backtrace(&self) -> &B {
        &self.0.as_ref().1
    }

    pub fn inner(&self) -> &T {
        &self.0.as_ref().0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0.as_mut().0
    }

    pub fn into_inner(self) -> T {
        (*self.0).0
    }
}

impl<T, B> std::ops::Deref for ErrorBox<T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T, B> std::ops::DerefMut for ErrorBox<T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<T: std::fmt::Display, B> std::fmt::Display for ErrorBox<T, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner().fmt(f)
    }
}

impl<T: std::fmt::Debug, B> std::fmt::Debug for ErrorBox<T, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner().fmt(f)
    }
}

impl<T: std::error::Error, B: WithBacktrace> std::error::Error for ErrorBox<T, B> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        T::source(self.inner())
    }

    // https://github.com/rust-lang/rust/issues/117432
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        self.backtrace().provide(request);
        T::provide(self.inner(), request);
    }
}
