use core::fmt;

use thiserror::Error;

/// Generic errors for WKT writing and reading
#[derive(Error, Debug)]
pub enum Error {
    #[error("Only 2D input is supported when writing Rect to WKT.")]
    RectUnsupportedDimension,
    #[error("Only defined dimensions and undefined dimensions of 2, 3, or 4 are supported.")]
    UnknownDimension,
    /// Wrapper around `[std::fmt::Error]`
    #[error(transparent)]
    FmtError(#[from] core::fmt::Error),
}

impl From<Error> for fmt::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::FmtError(err) => err,
            _ => core::fmt::Error,
        }
    }
}
