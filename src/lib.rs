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
//! [`geo_types`] and [`geo_traits`] primitives enabled by default.
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
//! As of `wkt` version 0.12, this crate provides read and write integration with [`geo_traits`],
//! a collection of geometry access traits, to provide zero-copy integration with geometry
//! representations other than `geo-types`.
//!
//! This integration allows you to transparently read data from this crate's intermediate geometry
//! structure, and it allows you to write WKT strings directly from your geometry without any
//! intermediate representation.
//!
//! ### Reading
//!
//! You can use [`Wkt::from_str`] to parse a WKT string into this crate's intermediate geometry
//! structure. `Wkt` (and all structs defined in [types]) implement traits from [geo_traits]. You
//! can write functions in terms of those traits and you'll be able to work with the parsed WKT
//! without any further overhead.
//!
//! ```
//! use std::str::FromStr;
//! use wkt::Wkt;
//! use geo_traits::{GeometryTrait, GeometryType};
//!
//! fn is_line_string(geom: &impl GeometryTrait<T = f64>) {
//!     assert!(matches!(geom.as_type(), GeometryType::LineString(_)))
//! }
//!
//! let wktls: Wkt<f64> = Wkt::from_str("LINESTRING(10 20, 20 30)").unwrap();
//! is_line_string(&wktls);
//! ```
//!
//! Working with the trait definition is preferable to working with `wkt::Wkt` directly, as the
//! geometry trait will work with many different geometry representations; not just the one from
//! this crate.
//!
//! ### Writing
//!
//! Consult the functions provided in [`to_wkt`]. Those functions will write any `geo_traits` object to WKT without any intermediate overhead.
//!
//! Implement [`geo_traits`] on your own geometry representation and those functions will work out
//! of the box on your data.
use std::default::Default;
use std::fmt;
use std::str::FromStr;

use geo_traits::GeometryTrait;
use num_traits::{Float, Num, NumCast};

use crate::to_wkt::write_geometry;
use crate::tokenizer::{PeekableTokens, Token, Tokens};
use crate::types::{
    Dimension, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

pub mod to_wkt;
mod tokenizer;
mod wkt_macro;

/// Error variant for this crate
pub mod error;
/// `WKT` primitive types and collections
pub mod types;

mod infer_type;

pub use infer_type::infer_type;

#[cfg(feature = "geo-types")]
extern crate geo_types;

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
pub enum Wkt<T: WktNum = f64> {
    Point(Point<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

impl<T> Wkt<T>
where
    T: WktNum,
{
    /// Return the [Dimension] of this geometry.
    pub fn dimension(&self) -> Dimension {
        match self {
            Self::Point(g) => g.dimension(),
            Self::LineString(g) => g.dimension(),
            Self::Polygon(g) => g.dimension(),
            Self::MultiPoint(g) => g.dimension(),
            Self::MultiLineString(g) => g.dimension(),
            Self::MultiPolygon(g) => g.dimension(),
            Self::GeometryCollection(g) => g.dimension(),
        }
    }
}

impl<T> Wkt<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_word_and_tokens(
        word: &str,
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        // Normally Z/M/ZM is separated by a space from the primary WKT word. E.g. `POINT Z`
        // instead of `POINTZ`. However we wish to support both types (in reading). When written
        // without a space, `POINTZ` is considered a single word, which means we need to include
        // matches here.
        match word {
            w if w.eq_ignore_ascii_case("POINT") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POINTZ") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POINTM") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POINTZM") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("LINESTRING") || w.eq_ignore_ascii_case("LINEARRING") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("LINESTRINGZ") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("LINESTRINGM") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("LINESTRINGZM") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POLYGON") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POLYGONZ") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POLYGONM") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("POLYGONZM") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINT") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINTZ") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINTM") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINTZM") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRING") => {
                let x =
                    <MultiLineString<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRINGZ") => {
                let x = <MultiLineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRINGM") => {
                let x = <MultiLineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRINGZM") => {
                let x = <MultiLineString<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGON") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGONZ") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGONM") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGONZM") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTION") => {
                let x =
                    <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_header(tokens, None);
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTIONZ") => {
                let x = <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZ),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTIONM") => {
                let x = <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYM),
                );
                x.map(|y| y.into())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTIONZM") => {
                let x = <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_header(
                    tokens,
                    Some(Dimension::XYZM),
                );
                x.map(|y| y.into())
            }
            _ => Err("Invalid type encountered"),
        }
    }
}

impl<T> fmt::Display for Wkt<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_geometry(f, self)?)
    }
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
        Wkt::from_word_and_tokens(&word, &mut tokens)
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

impl<T: WktNum> GeometryTrait for Wkt<T> {
    type T = T;
    type PointType<'b>
        = Point<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<T>
    where
        Self: 'b;
    type RectType<'b>
        = geo_traits::UnimplementedRect<T>
    where
        Self: 'b;
    type LineType<'b>
        = geo_traits::UnimplementedLine<T>
    where
        Self: 'b;
    type TriangleType<'b>
        = geo_traits::UnimplementedTriangle<T>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        match self {
            Wkt::Point(geom) => geom.dim(),
            Wkt::LineString(geom) => geom.dim(),
            Wkt::Polygon(geom) => geom.dim(),
            Wkt::MultiPoint(geom) => geom.dim(),
            Wkt::MultiLineString(geom) => geom.dim(),
            Wkt::MultiPolygon(geom) => geom.dim(),
            Wkt::GeometryCollection(geom) => geom.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        match self {
            Wkt::Point(geom) => geo_traits::GeometryType::Point(geom),
            Wkt::LineString(geom) => geo_traits::GeometryType::LineString(geom),
            Wkt::Polygon(geom) => geo_traits::GeometryType::Polygon(geom),
            Wkt::MultiPoint(geom) => geo_traits::GeometryType::MultiPoint(geom),
            Wkt::MultiLineString(geom) => geo_traits::GeometryType::MultiLineString(geom),
            Wkt::MultiPolygon(geom) => geo_traits::GeometryType::MultiPolygon(geom),
            Wkt::GeometryCollection(geom) => geo_traits::GeometryType::GeometryCollection(geom),
        }
    }
}

impl<T: WktNum> GeometryTrait for &Wkt<T> {
    type T = T;
    type PointType<'b>
        = Point<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<T>
    where
        Self: 'b;
    type RectType<'b>
        = geo_traits::UnimplementedRect<T>
    where
        Self: 'b;
    type LineType<'b>
        = geo_traits::UnimplementedLine<T>
    where
        Self: 'b;
    type TriangleType<'b>
        = geo_traits::UnimplementedTriangle<T>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        match self {
            Wkt::Point(geom) => geom.dim(),
            Wkt::LineString(geom) => geom.dim(),
            Wkt::Polygon(geom) => geom.dim(),
            Wkt::MultiPoint(geom) => geom.dim(),
            Wkt::MultiLineString(geom) => geom.dim(),
            Wkt::MultiPolygon(geom) => geom.dim(),
            Wkt::GeometryCollection(geom) => geom.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        match self {
            Wkt::Point(geom) => geo_traits::GeometryType::Point(geom),
            Wkt::LineString(geom) => geo_traits::GeometryType::LineString(geom),
            Wkt::Polygon(geom) => geo_traits::GeometryType::Polygon(geom),
            Wkt::MultiPoint(geom) => geo_traits::GeometryType::MultiPoint(geom),
            Wkt::MultiLineString(geom) => geo_traits::GeometryType::MultiLineString(geom),
            Wkt::MultiPolygon(geom) => geo_traits::GeometryType::MultiPolygon(geom),
            Wkt::GeometryCollection(geom) => geo_traits::GeometryType::GeometryCollection(geom),
        }
    }
}

// Specialized implementations on each WKT concrete type.

macro_rules! impl_specialization {
    ($geometry_type:ident) => {
        impl<T: WktNum> GeometryTrait for $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = geo_traits::UnimplementedRect<T>
            where
                Self: 'b;
            type LineType<'b>
                = geo_traits::UnimplementedLine<T>
            where
                Self: 'b;
            type TriangleType<'b>
                = geo_traits::UnimplementedTriangle<T>
            where
                Self: 'b;

            fn dim(&self) -> geo_traits::Dimensions {
                self.dim.into()
            }

            fn as_type(
                &self,
            ) -> geo_traits::GeometryType<
                '_,
                Point<T>,
                LineString<T>,
                Polygon<T>,
                MultiPoint<T>,
                MultiLineString<T>,
                MultiPolygon<T>,
                GeometryCollection<T>,
                Self::RectType<'_>,
                Self::TriangleType<'_>,
                Self::LineType<'_>,
            > {
                geo_traits::GeometryType::$geometry_type(self)
            }
        }

        impl<'a, T: WktNum + 'a> GeometryTrait for &'a $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = geo_traits::UnimplementedRect<T>
            where
                Self: 'b;
            type LineType<'b>
                = geo_traits::UnimplementedLine<T>
            where
                Self: 'b;
            type TriangleType<'b>
                = geo_traits::UnimplementedTriangle<T>
            where
                Self: 'b;

            fn dim(&self) -> geo_traits::Dimensions {
                self.dim.into()
            }

            fn as_type(
                &self,
            ) -> geo_traits::GeometryType<
                '_,
                Point<T>,
                LineString<T>,
                Polygon<T>,
                MultiPoint<T>,
                MultiLineString<T>,
                MultiPolygon<T>,
                GeometryCollection<T>,
                Self::RectType<'_>,
                Self::TriangleType<'_>,
                Self::LineType<'_>,
            > {
                geo_traits::GeometryType::$geometry_type(self)
            }
        }
    };
}

impl_specialization!(Point);
impl_specialization!(LineString);
impl_specialization!(Polygon);
impl_specialization!(MultiPoint);
impl_specialization!(MultiLineString);
impl_specialization!(MultiPolygon);
impl_specialization!(GeometryCollection);

fn infer_geom_dimension<T: WktNum + FromStr + Default>(
    tokens: &mut PeekableTokens<T>,
) -> Result<Dimension, &'static str> {
    if let Some(Ok(c)) = tokens.peek() {
        match c {
            // If we match a word check if it's Z/M/ZM and consume the token from the stream
            Token::Word(w) => match w.as_str() {
                w if w.eq_ignore_ascii_case("Z") => {
                    tokens.next().unwrap().unwrap();
                    Ok(Dimension::XYZ)
                }
                w if w.eq_ignore_ascii_case("M") => {
                    tokens.next().unwrap().unwrap();

                    Ok(Dimension::XYM)
                }
                w if w.eq_ignore_ascii_case("ZM") => {
                    tokens.next().unwrap().unwrap();
                    Ok(Dimension::XYZM)
                }
                w if w.eq_ignore_ascii_case("EMPTY") => Ok(Dimension::XY),
                _ => Err("Unexpected word before open paren"),
            },
            // Not a word, e.g. an open paren
            _ => Ok(Dimension::XY),
        }
    } else {
        Err("End of stream")
    }
}

trait FromTokens<T>: Sized + Default
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str>;

    fn new_empty(dim: Dimension) -> Self;

    /// The preferred top-level FromTokens API, which additionally checks for the presence of Z, M,
    /// and ZM in the token stream.
    fn from_tokens_with_header(
        tokens: &mut PeekableTokens<T>,
        dim: Option<Dimension>,
    ) -> Result<Self, &'static str> {
        let dim = if let Some(dim) = dim {
            dim
        } else {
            infer_geom_dimension(tokens)?
        };
        FromTokens::from_tokens_with_parens(tokens, dim)
    }

    fn from_tokens_with_parens(
        tokens: &mut PeekableTokens<T>,
        dim: Dimension,
    ) -> Result<Self, &'static str> {
        match tokens.next().transpose()? {
            Some(Token::ParenOpen) => (),
            Some(Token::Word(ref s)) if s.eq_ignore_ascii_case("EMPTY") => {
                return Ok(Self::new_empty(dim));
            }
            _ => return Err("Missing open parenthesis for type"),
        };
        let result = FromTokens::from_tokens(tokens, dim);
        match tokens.next().transpose()? {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for type"),
        };
        result
    }

    fn from_tokens_with_optional_parens(
        tokens: &mut PeekableTokens<T>,
        dim: Dimension,
    ) -> Result<Self, &'static str> {
        match tokens.peek() {
            Some(Ok(Token::ParenOpen)) => Self::from_tokens_with_parens(tokens, dim),
            _ => Self::from_tokens(tokens, dim),
        }
    }

    fn comma_many<F>(
        f: F,
        tokens: &mut PeekableTokens<T>,
        dim: Dimension,
    ) -> Result<Vec<Self>, &'static str>
    where
        F: Fn(&mut PeekableTokens<T>, Dimension) -> Result<Self, &'static str>,
    {
        let mut items = Vec::new();

        let item = f(tokens, dim)?;
        items.push(item);

        while let Some(&Ok(Token::Comma)) = tokens.peek() {
            tokens.next(); // throw away comma

            let item = f(tokens, dim)?;
            items.push(item);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{Dimension, MultiPolygon, Point};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn empty_string() {
        let res: Result<Wkt<f64>, _> = Wkt::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn empty_items() {
        let wkt: Wkt<f64> = Wkt::from_str("POINT EMPTY").ok().unwrap();
        match wkt {
            Wkt::Point(Point { coord, dim }) => {
                assert_eq!(coord, None);
                assert_eq!(dim, Dimension::XY);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        match wkt {
            Wkt::MultiPolygon(MultiPolygon { polygons, dim }) => {
                assert_eq!(polygons.len(), 0);
                assert_eq!(dim, Dimension::XY);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn lowercase_point() {
        let wkt: Wkt<f64> = Wkt::from_str("point EMPTY").ok().unwrap();
        match wkt {
            Wkt::Point(Point { coord, dim }) => {
                assert_eq!(coord, None);
                assert_eq!(dim, Dimension::XY);
            }
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
        match wkt {
            Wkt::Point(Point {
                coord: Some(coord),
                dim,
            }) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, None);
                assert_eq!(coord.m, None);
                assert_eq!(dim, Dimension::XY);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, z)
        let wkt = <Wkt<f64>>::from_str("POINT Z (10 20.1 5)").ok().unwrap();
        match wkt {
            Wkt::Point(Point {
                coord: Some(coord),
                dim,
            }) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, Some(5.0));
                assert_eq!(coord.m, None);
                assert_eq!(dim, Dimension::XYZ);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, m)
        let wkt = <Wkt<f64>>::from_str("POINT M (10 20.1 80)").ok().unwrap();
        match wkt {
            Wkt::Point(Point {
                coord: Some(coord),
                dim,
            }) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, None);
                assert_eq!(coord.m, Some(80.0));
                assert_eq!(dim, Dimension::XYM);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }

        // point(x, y, z, m)
        let wkt = <Wkt<f64>>::from_str("POINT ZM (10 20.1 5 80)")
            .ok()
            .unwrap();
        match wkt {
            Wkt::Point(Point {
                coord: Some(coord),
                dim,
            }) => {
                assert_eq!(coord.x, 10.0);
                assert_eq!(coord.y, 20.1);
                assert_eq!(coord.z, Some(5.0));
                assert_eq!(coord.m, Some(80.0));
                assert_eq!(dim, Dimension::XYZM);
            }
            _ => panic!("excepted to be parsed as a POINT"),
        }
    }

    #[test]
    fn support_jts_linearring() {
        let wkt: Wkt<f64> = Wkt::from_str("linearring (10 20, 30 40)").ok().unwrap();
        match wkt {
            Wkt::LineString(_ls) => (),
            _ => panic!("expected to be parsed as a LINESTRING"),
        };
    }

    #[test]
    fn test_display_on_wkt() {
        let wktls: Wkt<f64> = Wkt::from_str("LINESTRING(10 20, 20 30)").unwrap();

        assert_eq!(wktls.to_string(), "LINESTRING(10 20,20 30)");
    }
}
