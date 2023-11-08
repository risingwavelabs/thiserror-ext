// This module is ported from https://github.com/shepmaster/snafu and then modified.
// Below is the original license.

// Copyright 2019- Jake Goulding
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{backtrace::Backtrace, fmt};

/// Extension trait for [`std::error::Error`] that provides a [`Report`]
/// that formats the error and its sources in a cleaned-up way.
pub trait AsReport: crate::error_sealed::Sealed {
    /// Returns a [`Report`] that formats the error and its sources in a
    /// cleaned-up way.
    fn as_report(&self) -> Report<'_>;
}

impl<T: std::error::Error> AsReport for T {
    fn as_report(&self) -> Report<'_> {
        Report(self)
    }
}

macro_rules! impl_as_report {
    ($({$ty:ty },)*) => {
        $(
            impl AsReport for $ty {
                fn as_report(&self) -> Report<'_> {
                    Report(self)
                }
            }
        )*
    };
}
crate::for_dyn_error_types! { impl_as_report }

/// A wrapper around an error that provides a cleaned up error trace for
/// display and debug formatting.
///
/// # Formatting
///
/// The report can be formatted using [`fmt::Display`] or [`fmt::Debug`],
/// which differs based on the alternate flag (`#`).
///
/// - Without the alternate flag, the error is formatted in a compact way:
///   ```text
///   Outer error text: Middle error text: Inner error text
///   ```
///
/// - With the alternate flag, the error is formatted in a multi-line
///   format, which is more readable:
///   ```text
///   Outer error text
///
///   Caused by these errors (recent errors listed first):
///     1. Middle error text
///     2. Inner error text
///   ```
///
/// - Additionally, [`fmt::Debug`] provide backtraces if available.
///
/// # Error source cleaning
///
/// It's common for errors with a `source` to have a `Display`
/// implementation that includes their source text as well:
///
/// ```text
/// Outer error text: Middle error text: Inner error text
/// ```
///
/// This works for smaller errors without much detail, but can be
/// annoying when trying to format the error in a more structured way,
/// such as line-by-line:
///
/// ```text
/// 1. Outer error text: Middle error text: Inner error text
/// 2. Middle error text: Inner error text
/// 3. Inner error text
/// ```
///
/// This iterator compares each pair of errors in the source chain,
/// removing the source error's text from the containing error's text:
///
/// ```text
/// 1. Outer error text
/// 2. Middle error text
/// 3. Inner error text
/// ```
pub struct Report<'a>(pub &'a dyn std::error::Error);

impl<'a> fmt::Display for Report<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cleaned_error_trace(f, f.alternate())
    }
}

impl<'a> fmt::Debug for Report<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cleaned_error_trace(f, f.alternate())?;

        if let Some(bt) = std::error::request_ref::<Backtrace>(self.0) {
            // Print a newline if we're not in alternate mode.
            if !f.alternate() {
                writeln!(f)?;
            }
            writeln!(f, "\nBacktrace:\n{}", bt)?;
        }

        Ok(())
    }
}

impl<'a> Report<'a> {
    fn cleaned_error_trace(&self, f: &mut fmt::Formatter, pretty: bool) -> Result<(), fmt::Error> {
        let cleaned_messages: Vec<_> = CleanedErrorText::new(self.0)
            .flat_map(|(_error, msg, _cleaned)| Some(msg).filter(|msg| !msg.is_empty()))
            .collect();

        let mut visible_messages = cleaned_messages.iter();

        let head = match visible_messages.next() {
            Some(v) => v,
            None => return Ok(()),
        };

        write!(f, "{}", head)?;

        if pretty {
            match cleaned_messages.len() {
                0 | 1 => {}
                2 => writeln!(f, "\n\nCaused by this error:")?,
                _ => writeln!(
                    f,
                    "\n\nCaused by these errors (recent errors listed first):"
                )?,
            }

            for (i, msg) in visible_messages.enumerate() {
                // Let's use 1-based indexing for presentation
                let i = i + 1;
                writeln!(f, "{:3}: {}", i, msg)?;
            }
        } else {
            // No newline at the end.
            for msg in visible_messages {
                write!(f, ": {}", msg)?;
            }
        }

        Ok(())
    }
}

/// An iterator over an Error and its sources that removes duplicated
/// text from the error display strings.
struct CleanedErrorText<'a>(Option<CleanedErrorTextStep<'a>>);

impl<'a> CleanedErrorText<'a> {
    /// Constructs the iterator.
    fn new(error: &'a dyn std::error::Error) -> Self {
        Self(Some(CleanedErrorTextStep::new(error)))
    }
}

impl<'a> Iterator for CleanedErrorText<'a> {
    /// The original error, the display string and if it has been cleaned
    type Item = (&'a dyn std::error::Error, String, bool);

    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        let mut step = self.0.take()?;
        let mut error_text = mem::take(&mut step.error_text);

        match step.error.source() {
            Some(next_error) => {
                let next_error_text = next_error.to_string();

                let cleaned_text = error_text
                    .trim_end_matches(&next_error_text)
                    .trim_end()
                    .trim_end_matches(':');
                let cleaned = cleaned_text.len() != error_text.len();
                let cleaned_len = cleaned_text.len();
                error_text.truncate(cleaned_len);

                self.0 = Some(CleanedErrorTextStep {
                    error: next_error,
                    error_text: next_error_text,
                });

                Some((step.error, error_text, cleaned))
            }
            None => Some((step.error, error_text, false)),
        }
    }
}

struct CleanedErrorTextStep<'a> {
    error: &'a dyn std::error::Error,
    error_text: String,
}

impl<'a> CleanedErrorTextStep<'a> {
    fn new(error: &'a dyn std::error::Error) -> Self {
        let error_text = error.to_string();
        Self { error, error_text }
    }
}
