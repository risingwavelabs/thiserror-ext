/// Workaround for https://github.com/rust-lang/rust/issues/117432.
#[derive(Clone)]
#[repr(transparent)]
pub struct ErrorBox<T>(pub Box<T>);

impl<T> ErrorBox<T> {
    pub fn new(t: T) -> Self {
        Self(Box::new(t))
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        *self.0
    }
}

impl<T> std::ops::Deref for ErrorBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ErrorBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ErrorBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ErrorBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: std::error::Error> std::error::Error for ErrorBox<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        T::source(&*self.0)
    }

    // https://github.com/rust-lang/rust/issues/117432
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        T::provide(&*self.0, request)
    }
}
