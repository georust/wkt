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

use geo_traits::{LineStringTrait, MultiLineStringTrait};

use crate::to_wkt::write_multi_linestring;
use crate::tokenizer::PeekableTokens;
use crate::types::linestring::LineString;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiLineString<T: WktNum>(pub Vec<LineString<T>>);

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
        result.map(MultiLineString)
    }
}

impl<T: WktNum> MultiLineStringTrait for MultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        // TODO: infer dimension from empty WKT
        if self.0.is_empty() {
            geo_traits::Dimensions::Xy
        } else {
            self.0[0].dim()
        }
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<T: WktNum> MultiLineStringTrait for &MultiLineString<T> {
    type T = T;
    type LineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        // TODO: infer dimension from empty WKT
        if self.0.is_empty() {
            geo_traits::Dimensions::Xy
        } else {
            self.0[0].dim()
        }
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{LineString, MultiLineString};
    use crate::types::Coord;
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_multilinestring() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))")
            .ok()
            .unwrap();
        let lines = match wkt {
            Wkt::MultiLineString(MultiLineString(lines)) => lines,
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn write_empty_multilinestring() {
        let multilinestring: MultiLineString<f64> = MultiLineString(vec![]);

        assert_eq!("MULTILINESTRING EMPTY", format!("{}", multilinestring));
    }

    #[test]
    fn write_multilinestring() {
        let multilinestring = MultiLineString(vec![
            LineString(vec![
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
            ]),
            LineString(vec![
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
            ]),
        ]);

        assert_eq!(
            "MULTILINESTRING((10.1 20.2,30.3 40.4),(50.5 60.6,70.7 80.8))",
            format!("{}", multilinestring)
        );
    }
}
