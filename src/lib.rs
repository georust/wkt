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

// The unstable `doc_auto_cfg` feature annotates documentation with any required cfg/features
// needed for optional items. We set the `docsrs` config when building for docs.rs. To use it
// in a local docs build, run: `cargo +nightly rustdoc --all-features -- --cfg docsrs`
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! The `wkt` crate provides conversions to and from the [WKT (Well Known Text)](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry)
//! geometry format.
//!
//! Conversions are available via the [`TryFromWkt`] and [`ToWkt`] traits, with implementations for
//! [`geo_types`] primitives enabled by default.
//!
//! For advanced usage, see the [`types`](crate::types) module for a list of internally used types.
//!
//! This crate has optional `serde` integration for deserializing fields containing WKT. See
//! [`deserialize`] for an example.
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

use num_traits::{Float, Num, NumCast};

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
#[cfg(feature = "serde")]
pub use deserialize::deserialize_wkt;

mod from_wkt;
pub use from_wkt::TryFromWkt;

#[cfg(all(feature = "serde", feature = "geo-types"))]
#[allow(deprecated)]
pub use deserialize::geo_types::deserialize_geometry;

#[cfg(all(feature = "serde", feature = "geo-types"))]
#[deprecated(
    since = "0.10.2",
    note = "instead: use wkt::deserialize::geo_types::deserialize_point"
)]
pub use deserialize::geo_types::deserialize_point;

pub trait WktNum: Num + NumCast + PartialOrd + PartialEq + Copy + fmt::Debug {}
impl<T> WktNum for T where T: Num + NumCast + PartialOrd + PartialEq + Copy + fmt::Debug {}

pub trait WktFloat: WktNum + Float {}
impl<T> WktFloat for T where T: WktNum + Float {}

#[derive(Clone, Debug, PartialEq)]
/// All supported WKT geometry [`types`]
pub enum Geometry<T>
where
    T: WktNum,
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
    T: WktNum + FromStr + Default,
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
            w if w.eq_ignore_ascii_case("POINTZ") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_parens(tokens)?;
                if let Some(coord) = &x.0 {
                    if coord.z.is_none() {
                        return Err("POINTZ must have a z-coordinate.");
                    }
                }
                Ok(x.as_item())
            }
            w if w.eq_ignore_ascii_case("POINTM") => {
                let mut x = <Point<T> as FromTokens<T>>::from_tokens_with_parens(tokens)?;
                if let Some(coord) = &mut x.0 {
                    if coord.z.is_none() {
                        return Err("POINTM must have an m-coordinate.");
                    } else {
                        coord.m = coord.z.take();
                    }
                }
                Ok(x.as_item())
            }
            w if w.eq_ignore_ascii_case("POINTZM") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_parens(tokens)?;
                if let Some(coord) = &x.0 {
                    if coord.z.is_none() || coord.m.is_none() {
                        return Err("POINTZM must have both a z- and m-coordinate");
                    }
                }
                Ok(x.as_item())
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
    T: WktNum + fmt::Display,
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
    T: WktNum,
{
    pub item: Geometry<T>,
}

impl<T> Wkt<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: Tokens<T>) -> Result<Self, &'static str> {
        let mut tokens = tokens.peekable();
        let word = match tokens.next().transpose()? {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word
            }
            _ => return Err("Invalid WKT format"),
        };
        Geometry::from_word_and_tokens(&word, &mut tokens).map(|item| Wkt { item })
    }
}

impl<T> FromStr for Wkt<T>
where
    T: WktNum + FromStr + Default,
{
    type Err = &'static str;

    fn from_str(wkt_str: &str) -> Result<Self, Self::Err> {
        Wkt::from_tokens(Tokens::from_str(wkt_str))
    }
}

impl<T> fmt::Display for Wkt<T>
where
    T: WktNum + fmt::Debug + fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.item.fmt(formatter)
    }
}

trait FromTokens<T>: Sized + Default
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        match tokens.next().transpose()? {
            Some(Token::ParenOpen) => (),
            Some(Token::Word(ref s)) if s.eq_ignore_ascii_case("EMPTY") => {
                return Ok(Default::default())
            }
            _ => return Err("Missing open parenthesis for type"),
        };
        let result = FromTokens::from_tokens(tokens);
        match tokens.next().transpose()? {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for type"),
        };
        result
    }

    fn from_tokens_with_optional_parens(
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        match tokens.peek() {
            Some(Ok(Token::ParenOpen)) => Self::from_tokens_with_parens(tokens),
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

        while let Some(&Ok(Token::Comma)) = tokens.peek() {
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
        let msg = <Wkt<f64>>::from_str("POINT (10 20.1A)").unwrap_err();
        assert_eq!(
            "Unable to parse input number as the desired output type",
            msg
        );
    }

    #[test]
    fn test_points() {
        // point(x, y)
        let wkt = <Wkt<f64>>::from_str("POINT (10 20.1)").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(Some(coord))) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, None);
                assert_eq!(coord.m, None);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, z)
        let wkt = <Wkt<f64>>::from_str("POINTZ (10 20.1 5)").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(Some(coord))) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, Some(5.0));
                assert_eq!(coord.m, None);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, m)
        let wkt = <Wkt<f64>>::from_str("POINTM (10 20.1 80)").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(Some(coord))) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, None);
                assert_eq!(coord.m, Some(80.0));
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, z, m)
        let wkt = <Wkt<f64>>::from_str("POINTZM (10 20.1 5 80)").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(Some(coord))) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, Some(5.0));
                assert_eq!(coord.m, Some(80.0));
            }
            _ => panic!("excepted to be parsed as a POINT"),
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
