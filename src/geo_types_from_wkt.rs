//! This module provides conversions between WKT primitives and [`geo_types`] primitives.
//!
//! See the [`std::convert::From`] and [`std::convert::TryFrom`] impls on individual [`crate::types`] and [`Wkt`] for details.
// Copyright 2014-2018 The GeoRust Developers
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

use crate::types::*;
use crate::{TryFromWkt, Wkt};

use std::any::type_name;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::str::FromStr;

use geo_types::{coord, CoordNum};
use thiserror::Error;

#[derive(Error, Debug)]
/// WKT to [`geo_types`] conversions errors
pub enum Error {
    #[error("The WKT Point was empty, but geo_type::Points cannot be empty")]
    PointConversionError,
    #[error("Mismatched geometry (expected {expected:?}, found {found:?})")]
    MismatchedGeometry {
        expected: &'static str,
        found: &'static str,
    },
    #[error("Wrong number of Geometries: {0}")]
    WrongNumberOfGeometries(usize),
    #[error("Invalid WKT: {0}")]
    InvalidWKT(&'static str),
    #[error("External error: {0}")]
    External(Box<dyn std::error::Error>),
}

macro_rules! try_from_wkt_impl {
    ($($type: ident),+) => {
        $(
            /// Fallibly convert this WKT primitive into this [`geo_types`] primitive
            impl<T: CoordNum> TryFrom<Wkt<T>> for geo_types::$type<T> {
                type Error = Error;

                fn try_from(wkt: Wkt<T>) -> Result<Self, Self::Error> {
                    let geometry = geo_types::Geometry::try_from(wkt)?;
                    Self::try_from(geometry).map_err(|e| {
                        match e {
                            geo_types::Error::MismatchedGeometry { expected, found } => {
                                Error::MismatchedGeometry { expected, found }
                            }
                            // currently only one error type in geo-types error enum, but that seems likely to change
                            #[allow(unreachable_patterns)]
                            other => Error::External(Box::new(other)),
                        }
                    })
                }
            }
        )+
    }
}

try_from_wkt_impl!(
    Point,
    Line,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    // See impl below.
    // GeometryCollection,
    Rect,
    Triangle
);

/// Fallibly convert this WKT primitive into this [`geo_types`] primitive
impl<T: CoordNum> TryFrom<Wkt<T>> for geo_types::GeometryCollection<T> {
    type Error = Error;

    fn try_from(wkt: Wkt<T>) -> Result<Self, Self::Error> {
        match wkt {
            Wkt::GeometryCollection(collection) => {
                let geometries: Result<Vec<geo_types::Geometry<T>>, _> = collection
                    .geoms
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect();
                Ok(geo_types::GeometryCollection(geometries?))
            }
            // geo_types doesn't implement `Geometry::try_from(geom_collec)` yet
            // (see https://github.com/georust/geo/pull/821).
            // So instead we synthesize the type of error it *would* return.
            Wkt::Point(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::Point<T>>(),
            }),
            Wkt::LineString(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::LineString<T>>(),
            }),
            Wkt::Polygon(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::Polygon<T>>(),
            }),
            Wkt::MultiPoint(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::MultiPoint<T>>(),
            }),
            Wkt::MultiLineString(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::MultiLineString<T>>(),
            }),
            Wkt::MultiPolygon(_) => Err(Error::MismatchedGeometry {
                expected: type_name::<Self>(),
                found: type_name::<geo_types::MultiPolygon<T>>(),
            }),
        }
    }
}

impl<T> From<Coord<T>> for geo_types::Coord<T>
where
    T: CoordNum,
{
    /// Convert from a WKT Coordinate to a [`geo_types::Coordinate`]
    fn from(coord: Coord<T>) -> geo_types::Coord<T> {
        coord! { x: coord.x, y: coord.y }
    }
}

impl<T> TryFrom<Point<T>> for geo_types::Point<T>
where
    T: CoordNum,
{
    type Error = Error;

    /// Fallibly convert from a WKT `POINT` to a [`geo_types::Point`]
    fn try_from(point: Point<T>) -> Result<Self, Self::Error> {
        match point.coord {
            Some(coord) => Ok(Self::new(coord.x, coord.y)),
            None => Err(Error::PointConversionError),
        }
    }
}

#[deprecated(since = "0.9.0", note = "use `geometry.try_into()` instead")]
pub fn try_into_geometry<T>(geometry: &Wkt<T>) -> Result<geo_types::Geometry<T>, Error>
where
    T: CoordNum,
{
    geometry.clone().try_into()
}

impl<'a, T> From<&'a LineString<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    fn from(line_string: &'a LineString<T>) -> Self {
        Self::LineString(line_string.clone().into())
    }
}

impl<T> From<LineString<T>> for geo_types::LineString<T>
where
    T: CoordNum,
{
    /// Convert from a WKT `LINESTRING` to a [`geo_types::LineString`]
    fn from(line_string: LineString<T>) -> Self {
        let coords = line_string
            .coords
            .into_iter()
            .map(geo_types::Coord::from)
            .collect();

        geo_types::LineString(coords)
    }
}

impl<'a, T> From<&'a MultiLineString<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    fn from(multi_line_string: &'a MultiLineString<T>) -> geo_types::Geometry<T> {
        Self::MultiLineString(multi_line_string.clone().into())
    }
}

impl<T> From<MultiLineString<T>> for geo_types::MultiLineString<T>
where
    T: CoordNum,
{
    /// Convert from a WKT `MULTILINESTRING` to a [`geo_types::MultiLineString`]
    fn from(multi_line_string: MultiLineString<T>) -> geo_types::MultiLineString<T> {
        let geo_line_strings: Vec<geo_types::LineString<T>> = multi_line_string
            .line_strings
            .into_iter()
            .map(geo_types::LineString::from)
            .collect();

        geo_types::MultiLineString(geo_line_strings)
    }
}

impl<'a, T> From<&'a Polygon<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    fn from(polygon: &'a Polygon<T>) -> geo_types::Geometry<T> {
        Self::Polygon(polygon.clone().into())
    }
}

impl<T> From<Polygon<T>> for geo_types::Polygon<T>
where
    T: CoordNum,
{
    /// Convert from a WKT `POLYGON` to a [`geo_types::Polygon`]
    fn from(polygon: Polygon<T>) -> Self {
        let mut iter = polygon.rings.into_iter().map(geo_types::LineString::from);
        match iter.next() {
            Some(interior) => geo_types::Polygon::new(interior, iter.collect()),
            None => geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]),
        }
    }
}

impl<'a, T> TryFrom<&'a MultiPoint<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    type Error = Error;

    fn try_from(multi_point: &'a MultiPoint<T>) -> Result<Self, Self::Error> {
        Ok(Self::MultiPoint(multi_point.clone().try_into()?))
    }
}

impl<T> TryFrom<MultiPoint<T>> for geo_types::MultiPoint<T>
where
    T: CoordNum,
{
    type Error = Error;
    /// Fallibly convert from a WKT `MULTIPOINT` to a [`geo_types::MultiPoint`]
    fn try_from(multi_point: MultiPoint<T>) -> Result<Self, Self::Error> {
        let points: Vec<geo_types::Point<T>> = multi_point
            .points
            .into_iter()
            .map(geo_types::Point::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(geo_types::MultiPoint(points))
    }
}

impl<'a, T> From<&'a MultiPolygon<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        Self::MultiPolygon(multi_polygon.clone().into())
    }
}

impl<T> From<MultiPolygon<T>> for geo_types::MultiPolygon<T>
where
    T: CoordNum,
{
    /// Convert from a WKT `MULTIPOLYGON` to a [`geo_types::MultiPolygon`]
    fn from(multi_polygon: MultiPolygon<T>) -> Self {
        let geo_polygons: Vec<geo_types::Polygon<T>> = multi_polygon
            .polygons
            .into_iter()
            .map(geo_types::Polygon::from)
            .collect();

        geo_types::MultiPolygon(geo_polygons)
    }
}

#[deprecated(since = "0.9.0", note = "use `geometry_collection.try_into()` instead")]
pub fn try_into_geometry_collection<T>(
    geometry_collection: &GeometryCollection<T>,
) -> Result<geo_types::Geometry<T>, Error>
where
    T: CoordNum,
{
    Ok(geo_types::Geometry::GeometryCollection(
        geometry_collection.clone().try_into()?,
    ))
}

impl<T> TryFrom<GeometryCollection<T>> for geo_types::GeometryCollection<T>
where
    T: CoordNum,
{
    type Error = Error;

    fn try_from(geometry_collection: GeometryCollection<T>) -> Result<Self, Self::Error> {
        let geo_geometries = geometry_collection
            .geoms
            .into_iter()
            .map(Wkt::try_into)
            .collect::<Result<_, _>>()?;

        Ok(geo_types::GeometryCollection(geo_geometries))
    }
}

impl<T> TryFrom<Wkt<T>> for geo_types::Geometry<T>
where
    T: CoordNum,
{
    type Error = Error;

    fn try_from(geometry: Wkt<T>) -> Result<Self, Self::Error> {
        Ok(match geometry {
            Wkt::Point(g) => {
                // Special case as `geo::Point` can't be empty
                if g.coord.is_some() {
                    geo_types::Point::try_from(g)?.into()
                } else {
                    geo_types::MultiPoint(vec![]).into()
                }
            }
            Wkt::LineString(g) => geo_types::Geometry::LineString(g.into()),
            Wkt::Polygon(g) => geo_types::Geometry::Polygon(g.into()),
            Wkt::MultiLineString(g) => geo_types::Geometry::MultiLineString(g.into()),
            Wkt::MultiPoint(g) => geo_types::Geometry::MultiPoint(g.try_into()?),
            Wkt::MultiPolygon(g) => geo_types::Geometry::MultiPolygon(g.into()),
            Wkt::GeometryCollection(g) => geo_types::Geometry::GeometryCollection(g.try_into()?),
        })
    }
}

/// Macro for implementing TryFromWkt for all the geo-types.
/// Alternatively, we could try to have a kind of blanket implementation on TryFrom<Wkt<T>>,
/// but:
///   1. what would be the type of TryFromWkt::Error?
///   2. that would preclude ever having a specialized implementation for geo-types as they'd
///      be ambiguous/redundant.
macro_rules! try_from_wkt_impl {
   ($($type: ty),*$(,)?)  => {
       $(
            impl<T: CoordNum + FromStr + Default> TryFromWkt<T> for $type {
                type Error = Error;
                fn try_from_wkt_str(wkt_str: &str) -> Result<Self, Self::Error> {
                    let wkt = Wkt::from_str(wkt_str).map_err(|e| Error::InvalidWKT(e))?;
                    Self::try_from(wkt)
                }

                fn try_from_wkt_reader(mut wkt_reader: impl Read) -> Result<Self, Self::Error> {
                    let mut bytes = vec![];
                    wkt_reader.read_to_end(&mut bytes).map_err(|e| Error::External(Box::new(e)))?;
                    let wkt_str = String::from_utf8(bytes).map_err(|e| Error::External(Box::new(e)))?;
                    Self::try_from_wkt_str(&wkt_str)
                }
            }
       )*
   }
}

try_from_wkt_impl![
    geo_types::Geometry<T>,
    geo_types::Point<T>,
    geo_types::Line<T>,
    geo_types::LineString<T>,
    geo_types::Polygon<T>,
    geo_types::MultiPoint<T>,
    geo_types::MultiLineString<T>,
    geo_types::MultiPolygon<T>,
    geo_types::GeometryCollection<T>,
    geo_types::Triangle<T>,
    geo_types::Rect<T>,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_single_item_wkt() {
        let wkt = Wkt::from(Point::from_coord(Coord {
            x: 1.0,
            y: 2.0,
            z: None,
            m: None,
        }));

        let converted = geo_types::Geometry::try_from(wkt).unwrap();
        let g_point: geo_types::Point<f64> = geo_types::Point::new(1.0, 2.0);

        assert_eq!(converted, geo_types::Geometry::Point(g_point));
    }

    #[test]
    fn convert_empty_point() {
        let point = Point::empty(Dimension::XY);
        let res: Result<geo_types::Point<f64>, Error> = point.try_into();
        assert!(res.is_err());
    }

    #[test]
    fn convert_point() {
        let point = Wkt::from(Point::from_coord(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        }));

        let g_point: geo_types::Point<f64> = (10., 20.).into();
        assert_eq!(
            geo_types::Geometry::Point(g_point),
            point.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_linestring() {
        let w_linestring = Wkt::from(LineString::empty(Dimension::XY));
        let g_linestring: geo_types::LineString<f64> = geo_types::LineString(vec![]);
        assert_eq!(
            geo_types::Geometry::LineString(g_linestring),
            w_linestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_linestring() {
        let w_linestring: Wkt = LineString::from_coords([
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
        let g_linestring: geo_types::LineString<f64> = vec![(10., 20.), (30., 40.)].into();
        assert_eq!(
            geo_types::Geometry::LineString(g_linestring),
            w_linestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_polygon() {
        let w_polygon: Wkt = Polygon::empty(Dimension::XY).into();
        let g_polygon: geo_types::Polygon<f64> =
            geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]);
        assert_eq!(
            geo_types::Geometry::Polygon(g_polygon),
            w_polygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_polygon() {
        let w_polygon: Wkt = Polygon::from_rings([
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
            ])
            .unwrap(),
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
            ])
            .unwrap(),
        ])
        .unwrap()
        .into();
        let g_polygon: geo_types::Polygon<f64> = geo_types::Polygon::new(
            vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
            vec![vec![(5., 5.), (20., 30.), (30., 5.), (5., 5.)].into()],
        );
        assert_eq!(
            geo_types::Geometry::Polygon(g_polygon),
            w_polygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multilinestring() {
        let w_multilinestring: Wkt = MultiLineString::empty(Dimension::XY).into();
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiLineString(g_multilinestring),
            w_multilinestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_multilinestring() {
        let w_multilinestring: Wkt = MultiLineString::from_line_strings([
            LineString::from_coords([
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
            .unwrap(),
            LineString::from_coords([
                Coord {
                    x: 50.,
                    y: 60.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.,
                    y: 80.,
                    z: None,
                    m: None,
                },
            ])
            .unwrap(),
        ])
        .unwrap()
        .into();
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![
            vec![(10., 20.), (30., 40.)].into(),
            vec![(50., 60.), (70., 80.)].into(),
        ]);
        assert_eq!(
            geo_types::Geometry::MultiLineString(g_multilinestring),
            w_multilinestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multipoint() {
        let w_multipoint: Wkt = MultiPoint::empty(Dimension::XY).into();
        let g_multipoint: geo_types::MultiPoint<f64> = geo_types::MultiPoint(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPoint(g_multipoint),
            w_multipoint.try_into().unwrap()
        );
    }

    #[test]
    fn convert_multipoint() {
        let w_multipoint: Wkt = MultiPoint::from_points([
            Point::from_coord(Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            }),
            Point::from_coord(Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            }),
        ])
        .unwrap()
        .into();
        let g_multipoint: geo_types::MultiPoint<f64> = vec![(10., 20.), (30., 40.)].into();
        assert_eq!(
            geo_types::Geometry::MultiPoint(g_multipoint),
            w_multipoint.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multipolygon() {
        let w_multipolygon: Wkt = MultiPolygon::empty(Dimension::XY).into();
        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPolygon(g_multipolygon),
            w_multipolygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_multipolygon() {
        let w_multipolygon: Wkt = MultiPolygon::from_polygons([
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
                ])
                .unwrap(),
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
                ])
                .unwrap(),
            ])
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

        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![
            geo_types::Polygon::new(
                vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
                vec![vec![(5., 5.), (20., 30.), (30., 5.), (5., 5.)].into()],
            ),
            geo_types::Polygon::new(
                vec![(40., 40.), (20., 45.), (45., 30.), (40., 40.)].into(),
                vec![],
            ),
        ]);
        assert_eq!(
            geo_types::Geometry::MultiPolygon(g_multipolygon),
            w_multipolygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_geometrycollection() {
        let w_geometrycollection: Wkt = GeometryCollection::empty(Dimension::XY).into();
        let g_geometrycollection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![]);
        assert_eq!(
            geo_types::Geometry::GeometryCollection(g_geometrycollection),
            w_geometrycollection.try_into().unwrap()
        );
    }

    #[test]
    fn convert_geometrycollection() {
        let w_point = Point::from_coord(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        })
        .into();

        let w_linestring = LineString::from_coords([
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

        let w_polygon = Polygon::from_rings([LineString::from_coords([
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
        .unwrap()
        .into();

        let w_multilinestring = MultiLineString::from_line_strings([
            LineString::from_coords([
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
            .unwrap(),
            LineString::from_coords([
                Coord {
                    x: 50.,
                    y: 60.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.,
                    y: 80.,
                    z: None,
                    m: None,
                },
            ])
            .unwrap(),
        ])
        .unwrap()
        .into();

        let w_multipoint = MultiPoint::from_points([
            Point::from_coord(Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            }),
            Point::from_coord(Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            }),
        ])
        .unwrap()
        .into();

        let w_multipolygon = MultiPolygon::from_polygons([
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

        let w_geometrycollection: Wkt = GeometryCollection::from_geometries([
            w_point,
            w_multipoint,
            w_linestring,
            w_multilinestring,
            w_polygon,
            w_multipolygon,
        ])
        .unwrap()
        .into();

        let g_point: geo_types::Point<f64> = (10., 20.).into();
        let g_linestring: geo_types::LineString<f64> = vec![(10., 20.), (30., 40.)].into();
        let g_polygon: geo_types::Polygon<f64> = geo_types::Polygon::new(
            vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
            vec![],
        );
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![
            vec![(10., 20.), (30., 40.)].into(),
            vec![(50., 60.), (70., 80.)].into(),
        ]);
        let g_multipoint: geo_types::MultiPoint<f64> = vec![(10., 20.), (30., 40.)].into();
        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![
            geo_types::Polygon::new(
                vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
                vec![],
            ),
            geo_types::Polygon::new(
                vec![(40., 40.), (20., 45.), (45., 30.), (40., 40.)].into(),
                vec![],
            ),
        ]);

        let g_geometrycollection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![
                geo_types::Geometry::Point(g_point),
                geo_types::Geometry::MultiPoint(g_multipoint),
                geo_types::Geometry::LineString(g_linestring),
                geo_types::Geometry::MultiLineString(g_multilinestring),
                geo_types::Geometry::Polygon(g_polygon),
                geo_types::Geometry::MultiPolygon(g_multipolygon),
            ]);
        assert_eq!(
            geo_types::Geometry::GeometryCollection(g_geometrycollection),
            w_geometrycollection.try_into().unwrap()
        );
    }

    #[test]
    fn geom_collection_from_wkt_str() {
        // geometry collections have some special handling vs. other geometries, so we test them separately.
        let collection = geo_types::GeometryCollection::<f64>::try_from_wkt_str(
            "GeometryCollection(POINT(1 2))",
        )
        .unwrap();
        let point: geo_types::Point<_> = collection[0].clone().try_into().unwrap();
        assert_eq!(point.y(), 2.0);
    }

    #[test]
    fn geom_collection_from_invalid_wkt_str() {
        // geometry collections have some special handling vs. other geometries, so we test them separately.
        let err = geo_types::GeometryCollection::<f64>::try_from_wkt_str("GeomColl(POINT(1 2))")
            .unwrap_err();
        match err {
            Error::InvalidWKT(err_text) => assert_eq!(err_text, "Invalid type encountered"),
            e => panic!("Not the error we expected. Found: {}", e),
        }
    }

    #[test]
    fn geom_collection_from_other_wkt_str() {
        // geometry collections have some special handling vs. other geometries, so we test them separately.
        let not_a_collection = geo_types::GeometryCollection::<f64>::try_from_wkt_str("POINT(1 2)");
        let err = not_a_collection.unwrap_err();
        match err {
            Error::MismatchedGeometry {
                expected: "geo_types::geometry::geometry_collection::GeometryCollection",
                found: "geo_types::geometry::point::Point",
            } => {}
            e => panic!("Not the error we expected. Found: {}", e),
        }
    }

    #[test]
    fn from_invalid_wkt_str() {
        let a_point_too_many = geo_types::Point::<f64>::try_from_wkt_str("PINT(1 2)");
        let err = a_point_too_many.unwrap_err();
        match err {
            Error::InvalidWKT(err_text) => assert_eq!(err_text, "Invalid type encountered"),
            e => panic!("Not the error we expected. Found: {}", e),
        }
    }

    #[test]
    fn from_other_geom_wkt_str() {
        let not_actually_a_line_string =
            geo_types::LineString::<f64>::try_from_wkt_str("POINT(1 2)");
        let err = not_actually_a_line_string.unwrap_err();
        match err {
            Error::MismatchedGeometry {
                expected: "geo_types::geometry::line_string::LineString",
                found: "geo_types::geometry::point::Point",
            } => {}
            e => panic!("Not the error we expected. Found: {}", e),
        }
    }

    #[test]
    fn integer_geometry() {
        use crate::to_wkt::ToWkt;
        let point: geo_types::Point<i32> =
            geo_types::Point::try_from_wkt_str("POINT(1 2)").unwrap();
        assert_eq!(point, geo_types::Point::new(1, 2));

        let wkt_string = point.wkt_string();
        assert_eq!("POINT(1 2)", &wkt_string);
    }

    #[test]
    fn integer_geometries_from_float() {
        let wkt_str = "POINT(1.1 1.9)";

        let _sanity_check = geo_types::Point::<f32>::try_from_wkt_str(wkt_str).unwrap();

        let result = geo_types::Point::<i32>::try_from_wkt_str(wkt_str);
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Invalid WKT: Unable to parse input number as the desired output type"
        );
    }
}
