#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The `wkt` crate provides conversions to and from the [WKT (Well Known Text)](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry)
//! geometry format.
//!
//! Conversions are available via the [`TryFromWkt`] and [`ToWkt`] traits, with implementations for
//! [`geo_types`] primitives enabled by default.
//!
//! For advanced usage, see the [`types`](crate::types) module for a list of internally used types.
//!
//! Enable the `serde` feature if you need to deserialise data into custom structs containing `WKT`
//! geometry fields.
//!
//! # Examples
//!
//! ## Read `geo_types` from a WKT string
#![cfg_attr(feature = "geo-types", doc = "```")]
#![cfg_attr(not(feature = "geo-types"), doc = "```ignore")]
//! // This example requires the geo-types feature (on by default).
//! use wkt::TryFromWkt;
//! use geo_types::Point;
//!
//! let point: Point<f64> = Point::try_from_wkt_str("POINT(10 20)").unwrap();
//! assert_eq!(point.y(), 20.0);
//! ```
//!
//! ## Write `geo_types` to a WKT string
#![cfg_attr(feature = "geo-types", doc = "```")]
#![cfg_attr(not(feature = "geo-types"), doc = "```ignore")]
//! // This example requires the geo-types feature (on by default).
//! use wkt::ToWkt;
//! use geo_types::Point;
//!
//! let point: Point<f64> = Point::new(1.0, 2.0);
//! assert_eq!(point.wkt_string(), "POINT(1 2)");
//! ```
//!
//! ## Read or write your own geometry types
//!
//! Not using `geo-types` for your geometries? No problem!
//!
//! You can use [`Wkt::from_str`] to parse a WKT string into this crate's intermediate geometry
//! structure. You can use that directly, or if have your own geometry types that you'd prefer to
//! use, utilize that [`Wkt`] struct to implement the [`ToWkt`] or [`TryFromWkt`] traits for your
//! own types.
//!
//! In doing so, you'll likely want to match on one of the WKT [`types`] (Point, Linestring, etc.)
//! stored in its `item` field
//! ```
//! use std::str::FromStr;
//! use wkt::Wkt;
//! use wkt::Geometry;
//!
//! let wktls: Wkt<f64> = Wkt::from_str("LINESTRING(10 20, 20 30)").unwrap();
//! let ls = match wktls.item {
//!     Geometry::LineString(line_string) => {
//!         // you now have access to the `wkt::types::LineString`.
//!         assert_eq!(line_string.0[0].x, 10.0);
//!     }
//!     _ => unreachable!(),
//! };
use std::default::Default;
use std::fmt;
use std::str::FromStr;

use crate::tokenizer::{PeekableTokens, Token, Tokens};
use crate::types::GeometryCollection;
use crate::types::LineString;
use crate::types::MultiLineString;
use crate::types::MultiPoint;
use crate::types::MultiPolygon;
use crate::types::Point;
use crate::types::Polygon;

mod to_wkt;
mod tokenizer;

/// `WKT` primitive types and collections
pub mod types;

#[cfg(feature = "geo-types")]
extern crate geo_types;

extern crate thiserror;

pub use crate::to_wkt::ToWkt;

#[cfg(feature = "geo-types")]
#[deprecated(note = "renamed module to `wkt::geo_types_from_wkt`")]
pub mod conversion;
#[cfg(feature = "geo-types")]
pub mod geo_types_from_wkt;
#[cfg(feature = "geo-types")]
mod geo_types_to_wkt;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
pub mod deserialize;
mod from_wkt;
pub use from_wkt::TryFromWkt;

#[cfg(all(feature = "serde", feature = "geo-types"))]
pub use deserialize::{deserialize_geometry, deserialize_point};

pub trait WktFloat: num_traits::Float + std::fmt::Debug {}
impl<T> WktFloat for T where T: num_traits::Float + std::fmt::Debug {}

#[derive(Clone, Debug)]
/// All supported WKT geometry [`types`]
pub enum Geometry<T>
where
    T: WktFloat,
{
    Point(Point<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

impl<T> Geometry<T>
where
    T: WktFloat + FromStr + Default,
{
    fn from_word_and_tokens(
        word: &str,
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        match word {
            w if w.eq_ignore_ascii_case("POINT") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("LINESTRING") || w.eq_ignore_ascii_case("LINEARRING") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("POLYGON") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINT") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRING") => {
                let x = <MultiLineString<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGON") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTION") => {
                let x = <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            _ => Err("Invalid type encountered"),
        }
    }
}

impl<T> fmt::Display for Geometry<T>
where
    T: WktFloat + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Geometry::Point(point) => point.fmt(f),
            Geometry::LineString(linestring) => linestring.fmt(f),
            Geometry::Polygon(polygon) => polygon.fmt(f),
            Geometry::MultiPoint(multipoint) => multipoint.fmt(f),
            Geometry::MultiLineString(multilinstring) => multilinstring.fmt(f),
            Geometry::MultiPolygon(multipolygon) => multipolygon.fmt(f),
            Geometry::GeometryCollection(geometrycollection) => geometrycollection.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
/// Container for WKT primitives and collections
///
/// This type can be fallibly converted to a [`geo_types`] primitive using [`std::convert::TryFrom`].
pub struct Wkt<T>
where
    T: WktFloat,
{
    pub item: Geometry<T>,
}

impl<T> Wkt<T>
where
    T: WktFloat + FromStr + Default,
{
    fn from_tokens(tokens: Tokens<T>) -> Result<Self, &'static str> {
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word
            }
            _ => return Err("Invalid WKT format"),
        };
        match Geometry::from_word_and_tokens(&word, &mut tokens) {
            Ok(item) => Ok(Wkt { item }),
            Err(s) => Err(s),
        }
    }
}

impl<T> FromStr for Wkt<T>
where
    T: WktFloat + FromStr + Default,
{
    type Err = &'static str;

    fn from_str(wkt_str: &str) -> Result<Self, Self::Err> {
        Wkt::from_tokens(Tokens::from_str(wkt_str))
    }
}

impl<T> fmt::Display for Wkt<T>
where
    T: WktFloat + fmt::Debug + fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.item.fmt(formatter)
    }
}

trait FromTokens<T>: Sized + Default
where
    T: WktFloat + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            Some(Token::Word(ref s)) if s.eq_ignore_ascii_case("EMPTY") => {
                return Ok(Default::default())
            }
            _ => return Err("Missing open parenthesis for type"),
        };
        let result = FromTokens::from_tokens(tokens);
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for type"),
        };
        result
    }

    fn from_tokens_with_optional_parens(
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        match tokens.peek() {
            Some(Token::ParenOpen) => Self::from_tokens_with_parens(tokens),
            _ => Self::from_tokens(tokens),
        }
    }

    fn comma_many<F>(f: F, tokens: &mut PeekableTokens<T>) -> Result<Vec<Self>, &'static str>
    where
        F: Fn(&mut PeekableTokens<T>) -> Result<Self, &'static str>,
    {
        let mut items = Vec::new();

        let item = f(tokens)?;
        items.push(item);

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next(); // throw away comma

            let item = f(tokens)?;
            items.push(item);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{Coord, MultiPolygon, Point};
    use crate::{Geometry, Wkt};
    use std::str::FromStr;

    #[test]
    fn empty_string() {
        let res: Result<Wkt<f64>, _> = Wkt::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn empty_items() {
        let wkt: Wkt<f64> = Wkt::from_str("POINT EMPTY").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        match wkt.item {
            Geometry::MultiPolygon(MultiPolygon(polygons)) => assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[test]
    fn lowercase_point() {
        let wkt: Wkt<f64> = Wkt::from_str("point EMPTY").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };
    }

    #[test]
    fn invalid_number() {
        if let Err(msg) = <Wkt<f64>>::from_str("POINT (10 20.1A)") {
            assert_eq!("Expected a number for the Y coordinate", msg);
        } else {
            panic!("Should not have parsed");
        }
    }

    #[test]
    fn support_jts_linearring() {
        let wkt: Wkt<f64> = Wkt::from_str("linearring (10 20, 30 40)").ok().unwrap();
        match wkt.item {
            Geometry::LineString(_ls) => (),
            _ => panic!("expected to be parsed as a LINESTRING"),
        };
    }

    #[test]
    fn test_debug() {
        let g = Geometry::Point(Point(Some(Coord {
            x: 1.0,
            y: 2.0,
            m: None,
            z: None,
        })));
        assert_eq!(
            format!("{:?}", g),
            "Point(Point(Some(Coord { x: 1.0, y: 2.0, z: None, m: None })))"
        );
    }

    #[test]
    fn test_display_on_wkt() {
        let wktls: Wkt<f64> = Wkt::from_str("LINESTRING(10 20, 20 30)").unwrap();

        assert_eq!(wktls.to_string(), "LINESTRING(10 20,20 30)");
    }
}
