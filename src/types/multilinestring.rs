// Copyright 2015 The GeoRust Developers
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

use geo_traits::MultiLineStringTrait;

use crate::error::Error;
use crate::to_wkt::write_multi_linestring;
use crate::tokenizer::PeekableTokens;
use crate::types::linestring::LineString;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

/// A parsed MultiLineString.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiLineString<T: WktNum = f64> {
    pub(crate) line_strings: Vec<LineString<T>>,
    pub(crate) dim: Dimension,
}

impl<T: WktNum> MultiLineString<T> {
    /// Create a new LineString from a sequence of [LineString] and known [Dimension].
    pub fn new(line_strings: Vec<LineString<T>>, dim: Dimension) -> Self {
        MultiLineString { dim, line_strings }
    }

    /// Create a new empty MultiLineString.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new MultiLineString from a non-empty sequence of [LineString].
    ///
    /// This will infer the dimension from the first line string, and will not validate that all
    /// line strings have the same dimension.
    ///
    /// ## Errors
    ///
    /// If the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_line_strings(
        line_strings: impl IntoIterator<Item = LineString<T>>,
    ) -> Result<Self, Error> {
        let line_strings = line_strings.into_iter().collect::<Vec<_>>();
        if line_strings.is_empty() {
            Err(Error::UnknownDimension)
        } else {
            let dim = line_strings[0].dimension();
            Ok(Self::new(line_strings, dim))
        }
    }

    /// Return the [Dimension] of this geometry.
    pub fn dimension(&self) -> Dimension {
        self.dim
    }

    /// Access the inner line strings.
    pub fn line_strings(&self) -> &[LineString<T>] {
        &self.line_strings
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<LineString<T>>, Dimension) {
        (self.line_strings, self.dim)
    }
}

impl<T> From<MultiLineString<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: MultiLineString<T>) -> Self {
        Wkt::MultiLineString(value)
    }
}

impl<T> fmt::Display for MultiLineString<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_multi_linestring(f, self)?)
    }
}

impl<T> FromTokens<T> for MultiLineString<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <LineString<T> as FromTokens<T>>::from_tokens_with_parens,
            tokens,
            dim,
        );
        result.map(|line_strings| MultiLineString { line_strings, dim })
    }

    fn new_empty(dim: Dimension) -> Self {
        Self::empty(dim)
    }
}

impl<T: WktNum> MultiLineStringTrait for MultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_line_strings(&self) -> usize {
        self.line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.line_strings.get_unchecked(i)
    }
}

impl<T: WktNum> MultiLineStringTrait for &MultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_line_strings(&self) -> usize {
        self.line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.line_strings.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{LineString, MultiLineString};
    use crate::types::{Coord, Dimension};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_multilinestring() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))")
            .ok()
            .unwrap();
        let lines = match wkt {
            Wkt::MultiLineString(MultiLineString { line_strings, dim }) => {
                assert_eq!(dim, Dimension::XY);
                line_strings
            }
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn parse_empty_multilinestring() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING EMPTY").ok().unwrap();
        match wkt {
            Wkt::MultiLineString(MultiLineString { line_strings, dim }) => {
                assert!(line_strings.is_empty());
                assert_eq!(dim, Dimension::XY);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING Z EMPTY").ok().unwrap();
        match wkt {
            Wkt::MultiLineString(MultiLineString { line_strings, dim }) => {
                assert!(line_strings.is_empty());
                assert_eq!(dim, Dimension::XYZ);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING M EMPTY").ok().unwrap();
        match wkt {
            Wkt::MultiLineString(MultiLineString { line_strings, dim }) => {
                assert!(line_strings.is_empty());
                assert_eq!(dim, Dimension::XYM);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING ZM EMPTY").ok().unwrap();
        match wkt {
            Wkt::MultiLineString(MultiLineString { line_strings, dim }) => {
                assert!(line_strings.is_empty());
                assert_eq!(dim, Dimension::XYZM);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn write_empty_multilinestring() {
        let multilinestring: MultiLineString<f64> = MultiLineString::empty(Dimension::XY);
        assert_eq!("MULTILINESTRING EMPTY", format!("{}", multilinestring));

        let multilinestring: MultiLineString<f64> = MultiLineString::empty(Dimension::XYZ);
        assert_eq!("MULTILINESTRING Z EMPTY", format!("{}", multilinestring));

        let multilinestring: MultiLineString<f64> = MultiLineString::empty(Dimension::XYM);
        assert_eq!("MULTILINESTRING M EMPTY", format!("{}", multilinestring));

        let multilinestring: MultiLineString<f64> = MultiLineString::empty(Dimension::XYZM);
        assert_eq!("MULTILINESTRING ZM EMPTY", format!("{}", multilinestring));
    }

    #[test]
    fn write_multilinestring() {
        let multilinestring = MultiLineString::from_line_strings([
            LineString::from_coords([
                Coord {
                    x: 10.1,
                    y: 20.2,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.3,
                    y: 40.4,
                    z: None,
                    m: None,
                },
            ])
            .unwrap(),
            LineString::from_coords([
                Coord {
                    x: 50.5,
                    y: 60.6,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.7,
                    y: 80.8,
                    z: None,
                    m: None,
                },
            ])
            .unwrap(),
        ])
        .unwrap();

        assert_eq!(
            "MULTILINESTRING((10.1 20.2,30.3 40.4),(50.5 60.6,70.7 80.8))",
            format!("{}", multilinestring)
        );
    }
}
