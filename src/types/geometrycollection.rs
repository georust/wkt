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

use crate::to_wkt::write_geometry_collection;
use crate::tokenizer::{PeekableTokens, Token};
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

/// A parsed GeometryCollection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryCollection<T: WktNum = f64> {
    pub(crate) geoms: Vec<Wkt<T>>,
    pub(crate) dim: Dimension,
}

impl<T: WktNum> GeometryCollection<T> {
    /// Create a new GeometryCollection from a sequence of [Wkt].
    pub fn new(geoms: Vec<Wkt<T>>, dim: Dimension) -> Self {
        Self { geoms, dim }
    }

    /// Create a new empty GeometryCollection.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new GeometryCollection from a non-empty sequence of [Wkt].
    ///
    /// This will infer the dimension from the first geometry, and will not validate that all
    /// geometries have the same dimension.
    ///
    /// ## Errors
    ///
    /// If the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_geometries(geoms: impl IntoIterator<Item = Wkt<T>>) -> Option<Self> {
        let geoms = geoms.into_iter().collect::<Vec<_>>();
        if geoms.is_empty() {
            None
        } else {
            let dim = geoms[0].dimension();
            Some(Self::new(geoms, dim))
        }
    }

    /// Return the [Dimension] of this geometry.
    pub fn dimension(&self) -> Dimension {
        self.dim
    }

    /// Access the underlying [Wkt] geometries.
    pub fn geometries(&self) -> &[Wkt<T>] {
        &self.geoms
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Wkt<T>>, Dimension) {
        (self.geoms, self.dim)
    }
}

impl<T> From<GeometryCollection<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: GeometryCollection<T>) -> Self {
        Wkt::GeometryCollection(value)
    }
}

impl<T> fmt::Display for GeometryCollection<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_geometry_collection(f, self)?)
    }
}

impl<T> FromTokens<T> for GeometryCollection<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let mut items = Vec::new();

        let word = match tokens.next().transpose()? {
            Some(Token::Word(w)) => w,
            _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
        };

        let item = Wkt::from_word_and_tokens(&word, tokens)?;
        items.push(item);

        while let Some(&Ok(Token::Comma)) = tokens.peek() {
            tokens.next(); // throw away comma

            let word = match tokens.next().transpose()? {
                Some(Token::Word(w)) => w,
                _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
            };

            let item = Wkt::from_word_and_tokens(&word, tokens)?;
            items.push(item);
        }

        Ok(GeometryCollection { geoms: items, dim })
    }

    fn new_empty(dim: Dimension) -> Self {
        Self::empty(dim)
    }
}

#[cfg(feature = "geo-traits_0_2")]
impl<T: WktNum> geo_traits_0_2::GeometryCollectionTrait for GeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = &'a Wkt<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits_0_2::Dimensions {
        self.dim.into()
    }

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}

#[cfg(feature = "geo-traits_0_2")]
impl<T: WktNum> geo_traits_0_2::GeometryCollectionTrait for &GeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = &'a Wkt<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits_0_2::Dimensions {
        self.dim.into()
    }

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}

#[cfg(feature = "geo-traits_0_3")]
impl<T: WktNum> geo_traits_0_3::GeometryCollectionTrait for GeometryCollection<T> {
    type GeometryType<'a>
        = &'a Wkt<T>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}

#[cfg(feature = "geo-traits_0_3")]
impl<T: WktNum> geo_traits_0_3::GeometryCollectionTrait for &GeometryCollection<T> {
    type GeometryType<'a>
        = &'a Wkt<T>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::GeometryCollection;
    use crate::types::*;
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_geometrycollection() {
        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4)))")
            .ok()
            .unwrap();
        let geoms = match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert_eq!(dim, Dimension::XY);
                geoms
            }
            _ => unreachable!(),
        };
        assert_eq!(1, geoms.len());
    }

    #[test]
    fn complex_geometrycollection() {
        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4),LINESTRING(4 6,7 10)))")
            .ok()
            .unwrap();
        let geoms = match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert_eq!(dim, Dimension::XY);
                geoms
            }
            _ => unreachable!(),
        };
        assert_eq!(2, geoms.len());
    }

    #[test]
    fn parse_empty_geometrycollection() {
        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION EMPTY").ok().unwrap();
        match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert!(geoms.is_empty());
                assert_eq!(dim, Dimension::XY);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION Z EMPTY").ok().unwrap();
        match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert!(geoms.is_empty());
                assert_eq!(dim, Dimension::XYZ);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION M EMPTY").ok().unwrap();
        match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert!(geoms.is_empty());
                assert_eq!(dim, Dimension::XYM);
            }
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION ZM EMPTY").ok().unwrap();
        match wkt {
            Wkt::GeometryCollection(GeometryCollection { geoms, dim }) => {
                assert!(geoms.is_empty());
                assert_eq!(dim, Dimension::XYZM);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn write_empty_geometry_collection() {
        let geometry_collection: GeometryCollection<f64> = GeometryCollection::empty(Dimension::XY);
        assert_eq!(
            "GEOMETRYCOLLECTION EMPTY",
            format!("{}", geometry_collection)
        );

        let geometry_collection: GeometryCollection<f64> =
            GeometryCollection::empty(Dimension::XYZ);
        assert_eq!(
            "GEOMETRYCOLLECTION Z EMPTY",
            format!("{}", geometry_collection)
        );

        let geometry_collection: GeometryCollection<f64> =
            GeometryCollection::empty(Dimension::XYM);
        assert_eq!(
            "GEOMETRYCOLLECTION M EMPTY",
            format!("{}", geometry_collection)
        );

        let geometry_collection: GeometryCollection<f64> =
            GeometryCollection::empty(Dimension::XYZM);
        assert_eq!(
            "GEOMETRYCOLLECTION ZM EMPTY",
            format!("{}", geometry_collection)
        );
    }

    #[test]
    fn write_geometry_collection() {
        let point = Point::from_coord(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        })
        .into();

        let multipoint = MultiPoint::from_points([
            Point::from_coord(Coord {
                x: 10.1,
                y: 20.2,
                z: None,
                m: None,
            }),
            Point::from_coord(Coord {
                x: 30.3,
                y: 40.4,
                z: None,
                m: None,
            }),
        ])
        .unwrap()
        .into();

        let linestring = LineString::from_coords([
            Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            },
            Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            },
        ])
        .unwrap()
        .into();

        let polygon = Polygon::from_rings([LineString::from_coords([
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
        ])
        .unwrap()])
        .unwrap();

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
        .unwrap()
        .into();

        let multipolygon = MultiPolygon::from_polygons([
            Polygon::from_rings([LineString::from_coords([
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
            ])
            .unwrap()])
            .unwrap(),
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
            ])
            .unwrap()])
            .unwrap(),
        ])
        .unwrap()
        .into();

        let geoms: Vec<Wkt<f64>> = vec![
            point,
            multipoint,
            linestring,
            polygon.into(),
            multilinestring,
            multipolygon,
        ];
        let geometrycollection = GeometryCollection::from_geometries(geoms).unwrap();

        assert_eq!(
            "GEOMETRYCOLLECTION(\
             POINT(10 20),\
             MULTIPOINT((10.1 20.2),(30.3 40.4)),\
             LINESTRING(10 20,30 40),\
             POLYGON((0 0,20 40,40 0,0 0)),\
             MULTILINESTRING((10.1 20.2,30.3 40.4),(50.5 60.6,70.7 80.8)),\
             MULTIPOLYGON(((0 0,20 40,40 0,0 0)),((40 40,20 45,45 30,40 40)))\
             )",
            format!("{}", geometrycollection)
        );
    }
}
