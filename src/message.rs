/// A macro combining `format!` and `AsReport::to_report_string`
/// to format the given arguments into a message `String`.
///
/// Used internally by macros derived by `Macro`.
#[macro_export]
#[doc(hidden)]
macro_rules! __message {
    ($msg:literal $(,)?) => {
        ::std::format!($msg)
    };
    ($err:expr $(,)?) => {{
        use $crate::AsReport;
        $err.to_report_string()
    }};
    ($fmt:expr, $($arg:tt)*) => {
        ::std::format!($fmt, $($arg)*)
    };
}
pub use __message as message;
