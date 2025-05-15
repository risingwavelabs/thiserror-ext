// The code in this module is copied from
// https://github.com/dtolnay/thiserror/blob/89fb343a23b356845fce5098a095b419b94d1c88/impl/src/.
//
// See `LICENSE` for the original license.

#![allow(dead_code)]
#![allow(clippy::all)]

pub mod ast;
pub mod attr;
pub mod fmt;
pub mod generics;
pub mod props;
pub mod scan_expr;
pub mod unraw;
