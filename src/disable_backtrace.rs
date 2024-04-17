use std::{backtrace::Backtrace, ops::Deref};

use crate::AsDyn;

/// TODO
pub struct DisableBacktrace<E>(pub E);

impl<E: std::fmt::Display> std::fmt::Display for DisableBacktrace<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<E: AsDyn> std::fmt::Debug for DisableBacktrace<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0.as_dyn(), f)
    }
}

impl<E: AsDyn> std::error::Error for DisableBacktrace<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.as_dyn().source()
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        static DISABLED_BACKTRACE: Backtrace = Backtrace::disabled();

        request.provide_ref::<Backtrace>(&DISABLED_BACKTRACE);
        self.0.as_dyn().provide(request)
    }
}

impl<E> Deref for DisableBacktrace<E>
where
    E: Deref<Target = dyn std::error::Error + Send + Sync + 'static>,
{
    type Target = dyn std::error::Error + 'static;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

// impl<E: AsDyn>
