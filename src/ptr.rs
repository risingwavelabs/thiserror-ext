#![allow(missing_docs)] // used in generated code only

use std::sync::Arc;

use crate::backtrace::WithBacktrace;

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

impl<T, B> std::ops::DerefMut for ErrorBox<T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

/// A [`Arc`] with optional backtrace.
#[repr(transparent)]
pub struct ErrorArc<T, B>(Arc<(T, B)>);

impl<T, B> Clone for ErrorArc<T, B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

macro_rules! impl_methods {
    ($ty:ident) => {
        impl<T: std::error::Error, B: WithBacktrace> $ty<T, B> {
            pub fn new(t: T) -> Self {
                let backtrace = B::capture(&t);
                Self((t, backtrace).into())
            }
        }

        impl<T, B> $ty<T, B> {
            fn backtrace(&self) -> &B {
                &self.0.as_ref().1
            }

            pub fn inner(&self) -> &T {
                &self.0.as_ref().0
            }
        }

        impl<T, B> std::ops::Deref for $ty<T, B> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                self.inner()
            }
        }

        impl<T: std::fmt::Display, B> std::fmt::Display for $ty<T, B> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner().fmt(f)
            }
        }

        impl<T: std::fmt::Debug, B> std::fmt::Debug for $ty<T, B> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner().fmt(f)
            }
        }

        impl<T: std::error::Error, B: WithBacktrace> std::error::Error for $ty<T, B> {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                T::source(self.inner())
            }

            // https://github.com/rust-lang/rust/issues/117432
            fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
                self.backtrace().provide(request);
                T::provide(self.inner(), request);
            }
        }
    };
}

impl_methods!(ErrorBox);
impl_methods!(ErrorArc);
