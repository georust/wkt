use std::fmt;

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

/// A coordinate axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
    M,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Axis::X => f.write_str("X"),
            Axis::Y => f.write_str("Y"),
            Axis::Z => f.write_str("Z"),
            Axis::M => f.write_str("M"),
        }
    }
}

/// Errors encountered while reading WKT
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    #[error("Maximum GeometryCollection nesting depth of {max_depth} exceeded")]
    MaxDepthExceeded { max_depth: usize },
    #[error("Invalid type encountered")]
    InvalidType,
    #[error("Invalid WKT format")]
    InvalidFormat,
    #[error("Encountered non-ascii word")]
    NonAsciiWord,
    #[error("Unable to parse input number as the desired output type")]
    InvalidNumber,
    #[error("Expected a number for the {0} coordinate")]
    ExpectedNumberForCoord(Axis),
    #[error("Expected a word in GEOMETRYCOLLECTION")]
    ExpectedWordInGeometryCollection,
    #[error("Unexpected word before open paren")]
    UnexpectedWordBeforeOpenParen,
    #[error("End of stream")]
    EndOfStream,
    #[error("Missing open parenthesis for type")]
    MissingOpenParenthesis,
    #[error("Missing closing parenthesis for type")]
    MissingClosingParenthesis,
    #[error("Unsupported WKT prefix {0}")]
    UnsupportedPrefix(String),
    #[error("Invalid WKT; no whitespace between geometry type and EMPTY.")]
    MissingWhitespaceBeforeEmpty,
    #[error("Invalid WKT; no '(' character and not EMPTY")]
    MissingOpenParenthesisNotEmpty,
}
