use core::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
/// WKT to [`geo_types`] conversions errors
pub enum Error {
    // #[error("The WKT Point was empty, but geo_type::Points cannot be empty")]
    // RectWriter,
    #[error("Only defined dimensions and undefined dimensions of 2, 3, or 4 are supported.")]
    UnknownDimension,
    /// Wrapper around `[std::fmt::Error]`
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

impl From<Error> for fmt::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::FmtError(err) => err,
            _ => std::fmt::Error,
        }
    }
}
