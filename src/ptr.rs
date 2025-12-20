use crate::backtrace::WithBacktrace;
use alloc::boxed::Box;

/// A [`Box`] with optional backtrace.
#[derive(Clone)]
#[repr(transparent)]
pub struct ErrorBox<T, B>(Box<(T, B)>);

impl<T, B> ErrorBox<T, B> {
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0.as_mut().0
    }

    pub fn into_inner(self) -> T {
        (*self.0).0
    }
}

impl<T, B> core::ops::DerefMut for ErrorBox<T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

#[cfg(feature = "std")]
/// A [`Arc`] with optional backtrace.
#[repr(transparent)]
pub struct ErrorArc<T, B>(std::sync::Arc<(T, B)>);

#[cfg(feature = "std")]
impl<T, B> Clone for ErrorArc<T, B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

macro_rules! impl_methods {
    ($ty:ident) => {
        impl<T: core::error::Error, B: WithBacktrace> $ty<T, B> {
            pub fn new(t: T) -> Self {
                let backtrace = B::capture(&t);
                Self((t, backtrace).into())
            }
        }

        impl<T, B> $ty<T, B> {
            #[cfg_attr(not(feature = "provide"), allow(dead_code))]
            fn backtrace(&self) -> &B {
                &self.0.as_ref().1
            }

            pub fn inner(&self) -> &T {
                &self.0.as_ref().0
            }
        }

        impl<T, B> core::ops::Deref for $ty<T, B> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                self.inner()
            }
        }

        impl<T: core::fmt::Display, B> core::fmt::Display for $ty<T, B> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.inner().fmt(f)
            }
        }

        impl<T: core::fmt::Debug, B> core::fmt::Debug for $ty<T, B> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.inner().fmt(f)
            }
        }

        impl<T: core::error::Error, B: WithBacktrace> core::error::Error for $ty<T, B> {
            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                T::source(self.inner())
            }

            // https://github.com/rust-lang/rust/issues/117432
            #[cfg(feature = "provide")]
            fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
                self.backtrace().provide(request);
                T::provide(self.inner(), request);
            }
        }
    };
}

impl_methods!(ErrorBox);

#[cfg(feature = "std")]
impl_methods!(ErrorArc);
