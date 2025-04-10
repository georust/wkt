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

use geo_traits::PolygonTrait;

use crate::to_wkt::write_polygon;
use crate::tokenizer::PeekableTokens;
use crate::types::linestring::LineString;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon<T: WktNum> {
    pub(crate) dim: Dimension,
    pub(crate) rings: Vec<LineString<T>>,
}

impl<T: WktNum> Polygon<T> {
    pub fn new(rings: Vec<LineString<T>>, dim: Dimension) -> Self {
        Polygon { dim, rings }
    }

    /// Create a new empty polygon.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new polygon from a non-empty sequence of [LineString].
    ///
    /// This will infer the dimension from the first line string, and will not validate that all
    /// line strings have the same dimension.
    ///
    /// ## Panics
    ///
    /// If the input iterator is empty.
    pub fn from_rings(rings: impl IntoIterator<Item = LineString<T>>) -> Self {
        let rings = rings.into_iter().collect::<Vec<_>>();
        let dim = rings[0].dim;
        Self::new(rings, dim)
    }
}

impl<T> From<Polygon<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: Polygon<T>) -> Self {
        Wkt::Polygon(value)
    }
}

impl<T> fmt::Display for Polygon<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_polygon(f, self)?)
    }
}

impl<T> FromTokens<T> for Polygon<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <LineString<T> as FromTokens<T>>::from_tokens_with_parens,
            tokens,
            dim,
        );
        result.map(|rings| Polygon { rings, dim })
    }

    fn new_empty(dim: Dimension) -> Self {
        Self::empty(dim)
    }
}

impl<T: WktNum> PolygonTrait for Polygon<T> {
    type T = T;
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}

impl<T: WktNum> PolygonTrait for &Polygon<T> {
    type T = T;
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::{LineString, Polygon};
    use crate::types::{Coord, Dimension};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_polygon() {
        let wkt: Wkt<f64> = Wkt::from_str("POLYGON ((8 4, 4 0, 0 4, 8 4), (7 3, 4 1, 1 4, 7 3))")
            .ok()
            .unwrap();
        let rings = match wkt {
            Wkt::Polygon(Polygon { rings, dim: _ }) => rings,
            _ => unreachable!(),
        };
        assert_eq!(2, rings.len());
    }

    #[test]
    fn parse_empty_polygon() {
        let wkt: Wkt<f64> = Wkt::from_str("POLYGON EMPTY").ok().unwrap();
        match wkt {
            Wkt::Polygon(Polygon { rings, dim }) => {
                assert!(rings.is_empty());
                assert_eq!(dim, Dimension::XY);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("POLYGON Z EMPTY").ok().unwrap();
        match wkt {
            Wkt::Polygon(Polygon { rings, dim }) => {
                assert!(rings.is_empty());
                assert_eq!(dim, Dimension::XYZ);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("POLYGON M EMPTY").ok().unwrap();
        match wkt {
            Wkt::Polygon(Polygon { rings, dim }) => {
                assert!(rings.is_empty());
                assert_eq!(dim, Dimension::XYM);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("POLYGON ZM EMPTY").ok().unwrap();
        match wkt {
            Wkt::Polygon(Polygon { rings, dim }) => {
                assert!(rings.is_empty());
                assert_eq!(dim, Dimension::XYZM);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn write_empty_polygon() {
        let polygon: Polygon<f64> = Polygon::empty(Dimension::XY);
        assert_eq!("POLYGON EMPTY", format!("{}", polygon));

        let polygon: Polygon<f64> = Polygon::empty(Dimension::XYZ);
        assert_eq!("POLYGON Z EMPTY", format!("{}", polygon));

        let polygon: Polygon<f64> = Polygon::empty(Dimension::XYM);
        assert_eq!("POLYGON M EMPTY", format!("{}", polygon));

        let polygon: Polygon<f64> = Polygon::empty(Dimension::XYZM);
        assert_eq!("POLYGON ZM EMPTY", format!("{}", polygon));
    }

    #[test]
    fn write_polygon() {
        let polygon = Polygon::from_rings([
            LineString::from_coords([
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
            ]),
            LineString::from_coords([
                Coord {
                    x: 5.,
                    y: 5.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 30.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.,
                    y: 5.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 5.,
                    y: 5.,
                    z: None,
                    m: None,
                },
            ]),
        ]);

        assert_eq!(
            "POLYGON((0 0,20 40,40 0,0 0),(5 5,20 30,30 5,5 5))",
            format!("{}", polygon)
        );
    }
}
