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

use geo_traits::MultiPolygonTrait;

use crate::to_wkt::write_multi_polygon;
use crate::tokenizer::PeekableTokens;
use crate::types::polygon::Polygon;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiPolygon<T: WktNum> {
    pub(crate) dim: Dimension,
    pub(crate) polygons: Vec<Polygon<T>>,
}

impl<T: WktNum> MultiPolygon<T> {
    pub fn new(polygons: Vec<Polygon<T>>, dim: Dimension) -> Self {
        MultiPolygon { dim, polygons }
    }

    /// Create a new empty MultiPolygon.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new MultiPolygon from a non-empty sequence of [Polygon].
    ///
    /// This will infer the dimension from the first polygon, and will not validate that all
    /// polygons have the same dimension.
    ///
    /// ## Panics
    ///
    /// If the input iterator is empty.
    pub fn from_polygons(polygons: impl IntoIterator<Item = Polygon<T>>) -> Self {
        let polygons = polygons.into_iter().collect::<Vec<_>>();
        let dim = polygons[0].dim;
        Self::new(polygons, dim)
    }
}

impl<T> From<MultiPolygon<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: MultiPolygon<T>) -> Self {
        Wkt::MultiPolygon(value)
    }
}

impl<T> fmt::Display for MultiPolygon<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_multi_polygon(f, self)?)
    }
}

impl<T> FromTokens<T> for MultiPolygon<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <Polygon<T> as FromTokens<T>>::from_tokens_with_parens,
            tokens,
            dim,
        );
        result.map(|polygons| MultiPolygon { polygons, dim })
    }
}

impl<T: WktNum> MultiPolygonTrait for MultiPolygon<T> {
    type T = T;
    type PolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}

impl<T: WktNum> MultiPolygonTrait for &MultiPolygon<T> {
    type T = T;
    type PolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiPolygon, Polygon};
    use crate::types::{Coord, Dimension, LineString};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_multipolygon() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOLYGON (((8 4)), ((4 0)))")
            .ok()
            .unwrap();
        let polygons = match wkt {
            Wkt::MultiPolygon(MultiPolygon { polygons, dim: _ }) => polygons,
            _ => unreachable!(),
        };
        assert_eq!(2, polygons.len());
    }

    #[test]
    fn write_empty_multipolygon() {
        let multipolygon: MultiPolygon<f64> = MultiPolygon::empty(Dimension::XY);
        assert_eq!("MULTIPOLYGON EMPTY", format!("{}", multipolygon));

        let multipolygon: MultiPolygon<f64> = MultiPolygon::empty(Dimension::XYZ);
        assert_eq!("MULTIPOLYGON Z EMPTY", format!("{}", multipolygon));

        let multipolygon: MultiPolygon<f64> = MultiPolygon::empty(Dimension::XYM);
        assert_eq!("MULTIPOLYGON M EMPTY", format!("{}", multipolygon));

        let multipolygon: MultiPolygon<f64> = MultiPolygon::empty(Dimension::XYZM);
        assert_eq!("MULTIPOLYGON ZM EMPTY", format!("{}", multipolygon));
    }

    #[test]
    fn write_multipolygon() {
        let multipolygon = MultiPolygon::from_polygons([
            Polygon::from_rings([
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
            ]),
            Polygon::from_rings([LineString::from_coords([
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 45.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 45.,
                    y: 30.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
            ])]),
        ]);

        assert_eq!(
            "MULTIPOLYGON(((0 0,20 40,40 0,0 0),(5 5,20 30,30 5,5 5)),((40 40,20 45,45 30,40 40)))",
            format!("{}", multipolygon)
        );
    }
}
