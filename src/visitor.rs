//! Parse WKT with a visitor, without building a [`Wkt`](crate::Wkt) value.
//!
//! [`visit_wkt`] parses its input and gives each part of the geometry to a visitor. The parser
//! keeps one fixed-size coordinate buffer and gives the coordinates to the visitor in batches, so
//! it does not allocate memory for coordinate sequences. A visitor that converts WKT into a
//! different format can write its output directly.
//!
//! # Visitors
//!
//! Each kind of geometry has its own visitor trait. The parser calls one method of a
//! [`GeometryVisitor`] to start the geometry, which returns the visitor for that kind of
//! geometry:
//!
//! - `POINT`: a [`PointVisitor`]. It receives one coordinate, or no coordinate if the point is
//!   `EMPTY`.
//! - `LINESTRING`: a [`CoordsVisitor`]. It receives zero or more batches of coordinates.
//! - `POLYGON`: a [`PolygonVisitor`]. It gives a [`CoordsVisitor`] for each ring.
//! - `MULTIPOINT`, `MULTILINESTRING`, `MULTIPOLYGON`: a [`MultiPointVisitor`],
//!   [`MultiLineStringVisitor`] or [`MultiPolygonVisitor`]. It gives a visitor for each member.
//!   The members have the dimension of the parent.
//! - `GEOMETRYCOLLECTION`: a [`GeometryCollectionVisitor`]. It gives a [`GeometryVisitor`] for
//!   each member. Each member has its own dimension.
//!
//! The parser ends each visitor when its part of the input ends, and the visitor then returns
//! its output. The output of the top-level visitor is the result of [`visit_wkt`]. Member
//! visitors return `()`. They borrow their parent, so they can write into it.
//!
//! The visitor types make the order of the calls certain: for example, a polygon visitor cannot
//! receive coordinates, and a point visitor receives at most one coordinate.
//!
//! # Coordinates
//!
//! Coordinates are a flat slice of interleaved values (for example, `x y z x y z` for
//! [`Dimension::XYZ`]), together with their [`Dimension`]. Each batch contains one or more whole
//! coordinates of one sequence. The parser can divide a sequence into more than one batch.
//!
//! # Errors
//!
//! Parsing stops at the first error, from the parser or from a visitor, and [`visit_wkt`]
//! returns that error. The parser makes no more calls to the visitors after the error.
//!
//! # Example
//!
//! This visitor counts the coordinates of a geometry. Each visitor type is the same struct,
//! which borrows the count.
//!
//! ```
//! use wkt::error::ParseError;
//! use wkt::types::Dimension;
//! use wkt::visitor::*;
//!
//! struct Counter<'a>(&'a mut usize);
//!
//! impl Counter<'_> {
//!     fn add(&mut self, values: &[f64], dim: Dimension) {
//!         let stride = match dim {
//!             Dimension::XY => 2,
//!             Dimension::XYZ | Dimension::XYM => 3,
//!             Dimension::XYZM => 4,
//!         };
//!         *self.0 += values.len() / stride;
//!     }
//!
//!     fn member(&mut self) -> Result<Counter<'_>, ParseError> {
//!         Ok(Counter(&mut *self.0))
//!     }
//! }
//!
//! impl GeometryVisitor<f64> for Counter<'_> {
//!     type Error = ParseError;
//!     type Output = ();
//!     type Point = Self;
//!     type LineString = Self;
//!     type Polygon = Self;
//!     type MultiPoint = Self;
//!     type MultiLineString = Self;
//!     type MultiPolygon = Self;
//!     type GeometryCollection = Self;
//!
//!     fn point(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn line_string(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn polygon(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn multi_point(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn multi_line_string(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn multi_polygon(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//!     fn geometry_collection(self, _: Dimension) -> Result<Self, ParseError> { Ok(self) }
//! }
//!
//! impl PointVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     fn coord(mut self, values: &[f64], dim: Dimension) -> Result<(), ParseError> {
//!         self.add(values, dim);
//!         Ok(())
//!     }
//!     fn empty(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl CoordsVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     fn coords(&mut self, values: &[f64], dim: Dimension) -> Result<(), ParseError> {
//!         self.add(values, dim);
//!         Ok(())
//!     }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl PolygonVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     type Ring<'b> = Counter<'b> where Self: 'b;
//!     fn ring(&mut self) -> Result<Counter<'_>, ParseError> { self.member() }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl MultiPointVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     type Point<'b> = Counter<'b> where Self: 'b;
//!     fn point(&mut self) -> Result<Counter<'_>, ParseError> { self.member() }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl MultiLineStringVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     type LineString<'b> = Counter<'b> where Self: 'b;
//!     fn line_string(&mut self) -> Result<Counter<'_>, ParseError> { self.member() }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl MultiPolygonVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     type Polygon<'b> = Counter<'b> where Self: 'b;
//!     fn polygon(&mut self) -> Result<Counter<'_>, ParseError> { self.member() }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! impl GeometryCollectionVisitor<f64, ParseError> for Counter<'_> {
//!     type Output = ();
//!     type Geometry<'b> = Counter<'b> where Self: 'b;
//!     fn geometry(&mut self) -> Result<Counter<'_>, ParseError> { self.member() }
//!     fn end(self) -> Result<(), ParseError> { Ok(()) }
//! }
//!
//! let mut count = 0;
//! visit_wkt("POLYGON Z ((0 0 1, 1 0 1, 1 1 1, 0 0 1))", Counter(&mut count)).unwrap();
//! assert_eq!(count, 4);
//! ```

use std::str::FromStr;

use crate::error::{Axis, ParseError};
use crate::tokenizer::{PeekableTokens, Token, Tokens};
use crate::types::{Dimension, GeometryType};
use crate::WktNum;

/// The maximum `GEOMETRYCOLLECTION` nesting depth. Parsing fails at a greater depth.
pub(crate) const MAX_DEPTH: usize = 128;

/// The number of values in the coordinate buffer. This is 64 coordinates of four values.
const BUFFER_LEN: usize = 256;

/// Receives one geometry of any type.
///
/// The parser calls one method, for the type of the geometry, and then uses the visitor that
/// it returns.
pub trait GeometryVisitor<T: WktNum>: Sized {
    /// The error type that [`visit_wkt`] returns. It must hold parse errors.
    type Error: From<ParseError>;
    /// The value that the visitor of the geometry returns when it ends.
    type Output;

    /// The visitor for a `POINT`.
    type Point: PointVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `LINESTRING`.
    type LineString: CoordsVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `POLYGON`.
    type Polygon: PolygonVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `MULTIPOINT`.
    type MultiPoint: MultiPointVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `MULTILINESTRING`.
    type MultiLineString: MultiLineStringVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `MULTIPOLYGON`.
    type MultiPolygon: MultiPolygonVisitor<T, Self::Error, Output = Self::Output>;
    /// The visitor for a `GEOMETRYCOLLECTION`.
    type GeometryCollection: GeometryCollectionVisitor<T, Self::Error, Output = Self::Output>;

    /// Starts a `POINT` of dimension `dim`.
    fn point(self, dim: Dimension) -> Result<Self::Point, Self::Error>;
    /// Starts a `LINESTRING` of dimension `dim`.
    fn line_string(self, dim: Dimension) -> Result<Self::LineString, Self::Error>;
    /// Starts a `POLYGON` of dimension `dim`.
    fn polygon(self, dim: Dimension) -> Result<Self::Polygon, Self::Error>;
    /// Starts a `MULTIPOINT` of dimension `dim`.
    fn multi_point(self, dim: Dimension) -> Result<Self::MultiPoint, Self::Error>;
    /// Starts a `MULTILINESTRING` of dimension `dim`.
    fn multi_line_string(self, dim: Dimension) -> Result<Self::MultiLineString, Self::Error>;
    /// Starts a `MULTIPOLYGON` of dimension `dim`.
    fn multi_polygon(self, dim: Dimension) -> Result<Self::MultiPolygon, Self::Error>;
    /// Starts a `GEOMETRYCOLLECTION` of dimension `dim`.
    fn geometry_collection(self, dim: Dimension) -> Result<Self::GeometryCollection, Self::Error>;
}

/// Receives a point.
pub trait PointVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;

    /// Receives the coordinate of the point as interleaved values, and ends the point.
    fn coord(self, values: &[T], dim: Dimension) -> Result<Self::Output, E>;

    /// Ends an `EMPTY` point.
    fn empty(self) -> Result<Self::Output, E>;
}

/// Receives the coordinates of a line string or a ring.
pub trait CoordsVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;

    /// Receives one or more coordinates as interleaved values.
    fn coords(&mut self, values: &[T], dim: Dimension) -> Result<(), E>;

    /// Ends the line string or ring.
    fn end(self) -> Result<Self::Output, E>;
}

/// Receives the rings of a polygon.
pub trait PolygonVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;
    /// The visitor for a ring.
    type Ring<'a>: CoordsVisitor<T, E, Output = ()>
    where
        Self: 'a;

    /// Starts a ring.
    fn ring(&mut self) -> Result<Self::Ring<'_>, E>;

    /// Ends the polygon.
    fn end(self) -> Result<Self::Output, E>;
}

/// Receives the points of a multi-point.
pub trait MultiPointVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;
    /// The visitor for a member point.
    type Point<'a>: PointVisitor<T, E, Output = ()>
    where
        Self: 'a;

    /// Starts a member point.
    fn point(&mut self) -> Result<Self::Point<'_>, E>;

    /// Ends the multi-point.
    fn end(self) -> Result<Self::Output, E>;
}

/// Receives the line strings of a multi-line string.
pub trait MultiLineStringVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;
    /// The visitor for a member line string.
    type LineString<'a>: CoordsVisitor<T, E, Output = ()>
    where
        Self: 'a;

    /// Starts a member line string.
    fn line_string(&mut self) -> Result<Self::LineString<'_>, E>;

    /// Ends the multi-line string.
    fn end(self) -> Result<Self::Output, E>;
}

/// Receives the polygons of a multi-polygon.
pub trait MultiPolygonVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;
    /// The visitor for a member polygon.
    type Polygon<'a>: PolygonVisitor<T, E, Output = ()>
    where
        Self: 'a;

    /// Starts a member polygon.
    fn polygon(&mut self) -> Result<Self::Polygon<'_>, E>;

    /// Ends the multi-polygon.
    fn end(self) -> Result<Self::Output, E>;
}

/// Receives the geometries of a geometry collection.
pub trait GeometryCollectionVisitor<T: WktNum, E> {
    /// The value that the visitor returns when it ends.
    type Output;
    /// The visitor for a member geometry.
    type Geometry<'a>: GeometryVisitor<T, Error = E, Output = ()>
    where
        Self: 'a;

    /// Starts a member geometry.
    fn geometry(&mut self) -> Result<Self::Geometry<'_>, E>;

    /// Ends the geometry collection.
    fn end(self) -> Result<Self::Output, E>;
}

/// Parses `input` with `visitor`, and returns the output of the visitor.
///
/// # Errors
///
/// Returns the first parse error or visitor error. See the [module documentation](self).
pub fn visit_wkt<T, V>(input: &str, visitor: V) -> Result<V::Output, V::Error>
where
    T: WktNum + FromStr + Default,
    V: GeometryVisitor<T>,
{
    let mut parser = Parser {
        tokens: Tokens::from_str(input).peekable(),
        buffer: [T::default(); BUFFER_LEN],
    };
    let word = match parser.tokens.next().transpose()? {
        Some(Token::Word(word)) => {
            if !word.is_ascii() {
                return Err(ParseError::NonAsciiWord.into());
            }
            word
        }
        _ => return Err(ParseError::InvalidFormat.into()),
    };
    parser.geometry(word, 0, visitor)
}

/// Parses `input` and discards the result.
///
/// # Errors
///
/// Returns the first parse error.
///
/// # Example
///
/// ```
/// assert!(wkt::visitor::validate::<f64>("POINT (1 2)").is_ok());
/// assert!(wkt::visitor::validate::<f64>("POINT (1)").is_err());
/// ```
pub fn validate<T>(input: &str) -> Result<(), ParseError>
where
    T: WktNum + FromStr + Default,
{
    visit_wkt::<T, _>(input, Validator)
}

/// A visitor that ignores its input. It is the visitor for each kind of geometry.
struct Validator;

impl<T: WktNum> GeometryVisitor<T> for Validator {
    type Error = ParseError;
    type Output = ();
    type Point = Self;
    type LineString = Self;
    type Polygon = Self;
    type MultiPoint = Self;
    type MultiLineString = Self;
    type MultiPolygon = Self;
    type GeometryCollection = Self;

    fn point(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn line_string(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn polygon(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn multi_point(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn multi_line_string(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn multi_polygon(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
    fn geometry_collection(self, _: Dimension) -> Result<Self, ParseError> {
        Ok(self)
    }
}

impl<T: WktNum> PointVisitor<T, ParseError> for Validator {
    type Output = ();
    fn coord(self, _: &[T], _: Dimension) -> Result<(), ParseError> {
        Ok(())
    }
    fn empty(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> CoordsVisitor<T, ParseError> for Validator {
    type Output = ();
    fn coords(&mut self, _: &[T], _: Dimension) -> Result<(), ParseError> {
        Ok(())
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> PolygonVisitor<T, ParseError> for Validator {
    type Output = ();
    type Ring<'a> = Validator;
    fn ring(&mut self) -> Result<Validator, ParseError> {
        Ok(Validator)
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> MultiPointVisitor<T, ParseError> for Validator {
    type Output = ();
    type Point<'a> = Validator;
    fn point(&mut self) -> Result<Validator, ParseError> {
        Ok(Validator)
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> MultiLineStringVisitor<T, ParseError> for Validator {
    type Output = ();
    type LineString<'a> = Validator;
    fn line_string(&mut self) -> Result<Validator, ParseError> {
        Ok(Validator)
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> MultiPolygonVisitor<T, ParseError> for Validator {
    type Output = ();
    type Polygon<'a> = Validator;
    fn polygon(&mut self) -> Result<Validator, ParseError> {
        Ok(Validator)
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

impl<T: WktNum> GeometryCollectionVisitor<T, ParseError> for Validator {
    type Output = ();
    type Geometry<'a> = Validator;
    fn geometry(&mut self) -> Result<Validator, ParseError> {
        Ok(Validator)
    }
    fn end(self) -> Result<(), ParseError> {
        Ok(())
    }
}

/// Reads an optional `Z`, `M` or `ZM` word after the geometry type word.
fn infer_geom_dimension<T: WktNum + FromStr + Default>(
    tokens: &mut PeekableTokens<T>,
) -> Result<Dimension, ParseError> {
    let dim = match tokens.peek() {
        None | Some(Err(_)) => return Err(ParseError::EndOfStream),
        Some(Ok(Token::Word(w))) => match w {
            w if w.eq_ignore_ascii_case("Z") => Dimension::XYZ,
            w if w.eq_ignore_ascii_case("M") => Dimension::XYM,
            w if w.eq_ignore_ascii_case("ZM") => Dimension::XYZM,
            w if w.eq_ignore_ascii_case("EMPTY") => return Ok(Dimension::XY),
            _ => return Err(ParseError::UnexpectedWordBeforeOpenParen),
        },
        // Not a word, for example an open parenthesis.
        Some(Ok(_)) => return Ok(Dimension::XY),
    };
    tokens.next();
    Ok(dim)
}

/// Returns the number of values in one coordinate of dimension `dim`.
pub(crate) fn stride(dim: Dimension) -> usize {
    match dim {
        Dimension::XY => 2,
        Dimension::XYZ | Dimension::XYM => 3,
        Dimension::XYZM => 4,
    }
}

/// Returns the geometry type and the optional dimension suffix of a geometry type word, for
/// example `POINTZ`. The comparison ignores ASCII case.
fn parse_type_word(word: &str) -> Option<(GeometryType, Option<Dimension>)> {
    const TYPES: [(&str, GeometryType); 7] = [
        ("POINT", GeometryType::Point),
        ("LINESTRING", GeometryType::LineString),
        ("POLYGON", GeometryType::Polygon),
        ("MULTIPOINT", GeometryType::MultiPoint),
        ("MULTILINESTRING", GeometryType::MultiLineString),
        ("MULTIPOLYGON", GeometryType::MultiPolygon),
        ("GEOMETRYCOLLECTION", GeometryType::GeometryCollection),
    ];
    // LINEARRING has no form with a dimension suffix.
    if word.eq_ignore_ascii_case("LINEARRING") {
        return Some((GeometryType::LineString, None));
    }
    TYPES.iter().find_map(|&(name, geometry_type)| {
        let prefix = word.get(..name.len())?;
        if !prefix.eq_ignore_ascii_case(name) {
            return None;
        }
        let dim = match &word[name.len()..] {
            "" => None,
            s if s.eq_ignore_ascii_case("Z") => Some(Dimension::XYZ),
            s if s.eq_ignore_ascii_case("M") => Some(Dimension::XYM),
            s if s.eq_ignore_ascii_case("ZM") => Some(Dimension::XYZM),
            _ => return None,
        };
        Some((geometry_type, dim))
    })
}

/// Reads one number for the `axis` coordinate.
fn number<T>(tokens: &mut PeekableTokens<T>, axis: Axis) -> Result<T, ParseError>
where
    T: WktNum + FromStr,
{
    match tokens.next().transpose()? {
        Some(Token::Number(n)) => Ok(n),
        _ => Err(ParseError::ExpectedNumberForCoord(axis)),
    }
}

/// Reads one coordinate of dimension `dim` into `out`, which has `stride(dim)` values.
fn read_coord<T>(
    tokens: &mut PeekableTokens<T>,
    dim: Dimension,
    out: &mut [T],
) -> Result<(), ParseError>
where
    T: WktNum + FromStr,
{
    out[0] = number(tokens, Axis::X)?;
    out[1] = number(tokens, Axis::Y)?;
    match dim {
        Dimension::XY => {}
        Dimension::XYZ => out[2] = number(tokens, Axis::Z)?,
        Dimension::XYM => out[2] = number(tokens, Axis::M)?,
        Dimension::XYZM => {
            out[2] = number(tokens, Axis::Z)?;
            out[3] = number(tokens, Axis::M)?;
        }
    }
    Ok(())
}

struct Parser<'a, T: WktNum + FromStr> {
    tokens: PeekableTokens<'a, T>,
    // The coordinate buffer for `coord_sequence`. It is a field so that it is initialised
    // one time for each parse.
    buffer: [T; BUFFER_LEN],
}

impl<T> Parser<'_, T>
where
    T: WktNum + FromStr + Default,
{
    /// Parses the geometry that begins with the type word `word`, at collection nesting depth
    /// `depth`.
    fn geometry<V: GeometryVisitor<T>>(
        &mut self,
        word: &str,
        depth: usize,
        v: V,
    ) -> Result<V::Output, V::Error> {
        let (geometry_type, dim) = parse_type_word(word).ok_or(ParseError::InvalidType)?;
        if geometry_type == GeometryType::GeometryCollection && depth >= MAX_DEPTH {
            return Err(ParseError::MaxDepthExceeded {
                max_depth: MAX_DEPTH,
            }
            .into());
        }
        let dim = match dim {
            Some(dim) => dim,
            None => infer_geom_dimension(&mut self.tokens)?,
        };
        match geometry_type {
            GeometryType::Point => self.point(dim, v.point(dim)?),
            GeometryType::LineString => self.line_string(dim, v.line_string(dim)?),
            GeometryType::Polygon => self.polygon(dim, v.polygon(dim)?),
            GeometryType::MultiPoint => self.multi_point(dim, v.multi_point(dim)?),
            GeometryType::MultiLineString => self.multi_line_string(dim, v.multi_line_string(dim)?),
            GeometryType::MultiPolygon => self.multi_polygon(dim, v.multi_polygon(dim)?),
            GeometryType::GeometryCollection => {
                self.geometry_collection(depth, v.geometry_collection(dim)?)
            }
        }
    }

    /// Consumes `(` and returns `true`, or consumes `EMPTY` and returns `false`.
    fn open_paren(&mut self) -> Result<bool, ParseError> {
        match self.tokens.next().transpose()? {
            Some(Token::ParenOpen) => Ok(true),
            Some(Token::Word(s)) if s.eq_ignore_ascii_case("EMPTY") => Ok(false),
            _ => Err(ParseError::MissingOpenParenthesis),
        }
    }

    fn close_paren(&mut self) -> Result<(), ParseError> {
        match self.tokens.next().transpose()? {
            Some(Token::ParenClose) => Ok(()),
            _ => Err(ParseError::MissingClosingParenthesis),
        }
    }

    /// Calls `item` for each item of a comma-separated list of one or more items.
    fn comma_many<E, F>(&mut self, mut item: F) -> Result<(), E>
    where
        F: FnMut(&mut Self) -> Result<(), E>,
    {
        item(self)?;
        while let Some(Ok(Token::Comma)) = self.tokens.peek() {
            self.tokens.next();
            item(self)?;
        }
        Ok(())
    }

    /// Parses `(x y)` or `EMPTY`.
    fn point<E, V>(&mut self, dim: Dimension, v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: PointVisitor<T, E>,
    {
        if !self.open_paren()? {
            return v.empty();
        }
        let output = self.point_coord(dim, v)?;
        self.close_paren()?;
        Ok(output)
    }

    /// Parses `x y` without parentheses.
    fn point_coord<E, V>(&mut self, dim: Dimension, v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: PointVisitor<T, E>,
    {
        let values = &mut [T::default(); 4][..stride(dim)];
        read_coord(&mut self.tokens, dim, values)?;
        v.coord(values, dim)
    }

    /// Parses `(x y, ...)` or `EMPTY`, for a line string or a ring.
    fn line_string<E, V>(&mut self, dim: Dimension, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: CoordsVisitor<T, E>,
    {
        if self.open_paren()? {
            self.coord_sequence(dim, &mut v)?;
            self.close_paren()?;
        }
        v.end()
    }

    fn polygon<E, V>(&mut self, dim: Dimension, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: PolygonVisitor<T, E>,
    {
        if self.open_paren()? {
            self.comma_many(|p| {
                let ring = v.ring()?;
                p.line_string(dim, ring)
            })?;
            self.close_paren()?;
        }
        v.end()
    }

    fn multi_point<E, V>(&mut self, dim: Dimension, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: MultiPointVisitor<T, E>,
    {
        if self.open_paren()? {
            self.comma_many(|p| {
                let point = v.point()?;
                // A member can be `EMPTY`, `(x y)`, or `x y`.
                let has_parens = match p.tokens.peek() {
                    Some(Ok(Token::ParenOpen)) => true,
                    Some(Ok(Token::Word(w))) => w.eq_ignore_ascii_case("EMPTY"),
                    _ => false,
                };
                if has_parens {
                    p.point(dim, point)
                } else {
                    p.point_coord(dim, point)
                }
            })?;
            self.close_paren()?;
        }
        v.end()
    }

    fn multi_line_string<E, V>(&mut self, dim: Dimension, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: MultiLineStringVisitor<T, E>,
    {
        if self.open_paren()? {
            self.comma_many(|p| {
                let line_string = v.line_string()?;
                p.line_string(dim, line_string)
            })?;
            self.close_paren()?;
        }
        v.end()
    }

    fn multi_polygon<E, V>(&mut self, dim: Dimension, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: MultiPolygonVisitor<T, E>,
    {
        if self.open_paren()? {
            self.comma_many(|p| {
                let polygon = v.polygon()?;
                p.polygon(dim, polygon)
            })?;
            self.close_paren()?;
        }
        v.end()
    }

    /// Parses the body of a geometry collection at nesting depth `depth`.
    fn geometry_collection<E, V>(&mut self, depth: usize, mut v: V) -> Result<V::Output, E>
    where
        E: From<ParseError>,
        V: GeometryCollectionVisitor<T, E>,
    {
        if self.open_paren()? {
            self.comma_many(|p| {
                let word = match p.tokens.next().transpose()? {
                    Some(Token::Word(w)) => w,
                    _ => return Err(ParseError::ExpectedWordInGeometryCollection.into()),
                };
                let geometry = v.geometry()?;
                p.geometry(word, depth + 1, geometry)
            })?;
            self.close_paren()?;
        }
        v.end()
    }

    /// Parses a comma-separated list of one or more coordinates and gives them to `v` in
    /// batches.
    fn coord_sequence<E, V>(&mut self, dim: Dimension, v: &mut V) -> Result<(), E>
    where
        E: From<ParseError>,
        V: CoordsVisitor<T, E>,
    {
        let n = stride(dim);
        // The number of values in the buffer. It is local to this function, so a batch
        // contains only coordinates of this sequence.
        let mut len = 0;
        loop {
            if len + n > BUFFER_LEN {
                v.coords(&self.buffer[..len], dim)?;
                len = 0;
            }
            read_coord(&mut self.tokens, dim, &mut self.buffer[len..len + n])?;
            len += n;
            match self.tokens.peek() {
                Some(Ok(Token::Comma)) => {
                    self.tokens.next();
                }
                _ => break,
            }
        }
        v.coords(&self.buffer[..len], dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestError {
        Parse(ParseError),
        Visitor,
    }

    impl From<ParseError> for TestError {
        fn from(e: ParseError) -> Self {
            TestError::Parse(e)
        }
    }

    #[derive(Default)]
    struct Log {
        events: Vec<String>,
        // The number of values in each batch of coordinates.
        batches: Vec<usize>,
        // Return an error at this event index.
        fail_at: Option<usize>,
    }

    // Records each call as a short string. It is the visitor for each kind of geometry.
    struct Recorder<'a>(&'a mut Log);

    impl Recorder<'_> {
        fn push(&mut self, event: String) -> Result<(), TestError> {
            if self.0.fail_at == Some(self.0.events.len()) {
                return Err(TestError::Visitor);
            }
            self.0.events.push(event);
            Ok(())
        }

        fn start(mut self, ty: &str, dim: Dimension) -> Result<Self, TestError> {
            self.push(format!("start {ty} {dim:?}"))?;
            Ok(self)
        }

        fn member(&mut self) -> Result<Recorder<'_>, TestError> {
            Ok(Recorder(&mut *self.0))
        }

        fn end(mut self) -> Result<(), TestError> {
            self.push("end".into())
        }
    }

    impl GeometryVisitor<f64> for Recorder<'_> {
        type Error = TestError;
        type Output = ();
        type Point = Self;
        type LineString = Self;
        type Polygon = Self;
        type MultiPoint = Self;
        type MultiLineString = Self;
        type MultiPolygon = Self;
        type GeometryCollection = Self;

        fn point(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("Point", dim)
        }
        fn line_string(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("LineString", dim)
        }
        fn polygon(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("Polygon", dim)
        }
        fn multi_point(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("MultiPoint", dim)
        }
        fn multi_line_string(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("MultiLineString", dim)
        }
        fn multi_polygon(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("MultiPolygon", dim)
        }
        fn geometry_collection(self, dim: Dimension) -> Result<Self, TestError> {
            self.start("GeometryCollection", dim)
        }
    }

    impl PointVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        fn coord(mut self, values: &[f64], dim: Dimension) -> Result<(), TestError> {
            self.push(format!("coord {values:?} {dim:?}"))?;
            self.end()
        }
        fn empty(self) -> Result<(), TestError> {
            self.end()
        }
    }

    impl CoordsVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        fn coords(&mut self, values: &[f64], dim: Dimension) -> Result<(), TestError> {
            self.0.batches.push(values.len());
            self.push(format!("coords {values:?} {dim:?}"))
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    impl PolygonVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        type Ring<'b>
            = Recorder<'b>
        where
            Self: 'b;
        fn ring(&mut self) -> Result<Recorder<'_>, TestError> {
            self.push("ring".into())?;
            self.member()
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    impl MultiPointVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        type Point<'b>
            = Recorder<'b>
        where
            Self: 'b;
        fn point(&mut self) -> Result<Recorder<'_>, TestError> {
            self.push("point".into())?;
            self.member()
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    impl MultiLineStringVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        type LineString<'b>
            = Recorder<'b>
        where
            Self: 'b;
        fn line_string(&mut self) -> Result<Recorder<'_>, TestError> {
            self.push("line string".into())?;
            self.member()
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    impl MultiPolygonVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        type Polygon<'b>
            = Recorder<'b>
        where
            Self: 'b;
        fn polygon(&mut self) -> Result<Recorder<'_>, TestError> {
            self.push("polygon".into())?;
            self.member()
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    impl GeometryCollectionVisitor<f64, TestError> for Recorder<'_> {
        type Output = ();
        type Geometry<'b>
            = Recorder<'b>
        where
            Self: 'b;
        fn geometry(&mut self) -> Result<Recorder<'_>, TestError> {
            self.member()
        }
        fn end(self) -> Result<(), TestError> {
            Recorder::end(self)
        }
    }

    fn record(input: &str) -> Log {
        let mut log = Log::default();
        visit_wkt(input, Recorder(&mut log)).unwrap();
        log
    }

    fn events(input: &str) -> Vec<String> {
        record(input).events
    }

    #[test]
    fn point() {
        assert_eq!(
            events("POINT Z (1 2 3)"),
            ["start Point XYZ", "coord [1.0, 2.0, 3.0] XYZ", "end"]
        );
        assert_eq!(events("point empty"), ["start Point XY", "end"]);
        assert_eq!(events("POINTM EMPTY"), ["start Point XYM", "end"]);
    }

    #[test]
    fn polygon_rings() {
        assert_eq!(
            events("POLYGON ((0 0, 1 0, 0 0), EMPTY)"),
            [
                "start Polygon XY",
                "ring",
                "coords [0.0, 0.0, 1.0, 0.0, 0.0, 0.0] XY",
                "end",
                "ring",
                "end",
                "end"
            ]
        );
    }

    #[test]
    fn multi_point_members() {
        assert_eq!(
            events("MULTIPOINT (1 2, (3 4), EMPTY)"),
            [
                "start MultiPoint XY",
                "point",
                "coord [1.0, 2.0] XY",
                "end",
                "point",
                "coord [3.0, 4.0] XY",
                "end",
                "point",
                "end",
                "end"
            ]
        );
    }

    #[test]
    fn collection_members_have_own_dimension() {
        assert_eq!(
            events("GEOMETRYCOLLECTION (POINT Z (1 2 3), LINESTRING EMPTY)"),
            [
                "start GeometryCollection XY",
                "start Point XYZ",
                "coord [1.0, 2.0, 3.0] XYZ",
                "end",
                "start LineString XY",
                "end",
                "end"
            ]
        );
    }

    #[test]
    fn long_sequence_is_split_into_whole_coordinates() {
        // 100 XYZ coordinates: 85 fit in one buffer of 256 values.
        let coords: Vec<String> = (0..100).map(|i| format!("{i} {i} {i}")).collect();
        let input = format!("LINESTRING Z ({})", coords.join(","));
        assert_eq!(record(&input).batches, [255, 45]);
    }

    #[test]
    fn batches_contain_one_sequence() {
        // The second ring must not continue the batch of the first ring.
        let log = record("MULTILINESTRING Z ((0 0 0, 1 1 1), (2 2 2))");
        assert_eq!(log.batches, [6, 3]);
    }

    #[test]
    fn visitor_error_stops_parsing() {
        let mut log = Log {
            fail_at: Some(1),
            ..Default::default()
        };
        let result = visit_wkt("MULTIPOINT (1 2, 3 4)", Recorder(&mut log));
        assert_eq!(result, Err(TestError::Visitor));
        assert_eq!(log.events, ["start MultiPoint XY"]);
    }

    #[test]
    fn parse_error_reaches_caller() {
        let mut log = Log::default();
        let result = visit_wkt("LINESTRING (1 2, 3)", Recorder(&mut log));
        assert_eq!(
            result,
            Err(TestError::Parse(ParseError::ExpectedNumberForCoord(
                Axis::Y
            )))
        );
    }

    #[test]
    fn validate_rejects_invalid_input() {
        assert_eq!(validate::<f64>("POINT (1 2)"), Ok(()));
        assert_eq!(
            validate::<f64>("POINTX (1 2)"),
            Err(ParseError::InvalidType)
        );
        assert_eq!(
            validate::<f64>("POLYGON ((1 2)"),
            Err(ParseError::MissingClosingParenthesis)
        );
    }
}
