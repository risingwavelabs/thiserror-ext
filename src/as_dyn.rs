/// Extension trait for [`std::error::Error`] that casts the error to
/// a trait object.
pub trait AsDyn: crate::error_sealed::Sealed {
    /// Casts the error to a trait object.
    fn as_dyn(&self) -> &(dyn std::error::Error + '_);
}

impl<T: std::error::Error> AsDyn for T {
    fn as_dyn(&self) -> &(dyn std::error::Error + '_) {
        self
    }
}

macro_rules! impl_as_dyn {
    ($({$ty:ty},)*) => {
        $(
            impl AsDyn for $ty {
                fn as_dyn(&self) -> &(dyn std::error::Error + '_) {
                    self
                }
            }
        )*
    };
}

crate::for_dyn_error_types! { impl_as_dyn }
