/// Extension trait for [`Error`] that casts the error to a trait object.
///
/// [`Error`]: core::error::Error
pub trait AsDyn: crate::error_sealed::Sealed {
    /// Casts the error to a trait object.
    fn as_dyn(&self) -> &(dyn core::error::Error + '_);
}

impl<T: core::error::Error> AsDyn for T {
    fn as_dyn(&self) -> &(dyn core::error::Error + '_) {
        self
    }
}

macro_rules! impl_as_dyn {
    ($({$ty:ty},)*) => {
        $(
            impl AsDyn for $ty {
                fn as_dyn(&self) -> &(dyn core::error::Error + '_) {
                    self
                }
            }
        )*
    };
}

crate::for_dyn_error_types! { impl_as_dyn }
