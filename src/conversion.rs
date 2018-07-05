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

use geo_types;

use std::fmt;
use types::*;
use Geometry;

pub enum Error {
    PointConversionError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PointConversionError => {
                f.write_str("impossible to convert empty point to geo_type point")
            }
        }
    }
}

fn try_into_point(point: &Point) -> Result<geo_types::Geometry<f64>, Error> {
    match point.0 {
        Some(ref c) => {
            let geo_point: geo_types::Point<f64> = (c.x, c.y).into();
            Ok(geo_point.into())
        }
        None => Err(Error::PointConversionError),
    }
}

pub fn try_into_geometry(geometry: &Geometry) -> Result<geo_types::Geometry<f64>, Error> {
    match geometry {
        Geometry::Point(point) => try_into_point(point),
        Geometry::LineString(linestring) => Ok(linestring.into()),
        Geometry::Polygon(polygon) => Ok(polygon.into()),
        Geometry::MultiLineString(multilinestring) => Ok(multilinestring.into()),
        Geometry::MultiPoint(multipoint) => Ok(multipoint.into()),
        Geometry::MultiPolygon(multipolygon) => Ok(multipolygon.into()),
        Geometry::GeometryCollection(geometrycollection) => try_into_geometrycollection(geometrycollection),
    }
}

impl<'a> From<&'a LineString> for geo_types::Geometry<f64> {
    fn from(linestring: &LineString) -> Self {
        let geo_linestring: geo_types::LineString<f64> =
            linestring.0.iter().map(|c| (c.x, c.y)).collect();

        geo_linestring.into()
    }
}

impl<'a> From<&'a MultiLineString> for geo_types::Geometry<f64> {
    fn from(multilinestring: &MultiLineString) -> Self {
        let geo_multilinestring: geo_types::MultiLineString<f64> = multilinestring
            .0
            .iter()
            .map(|l| l.0.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>())
            .collect();

        geo_multilinestring.into()
    }
}

fn w_polygon_to_g_polygon(polygon: &Polygon) -> geo_types::Polygon<f64> {
    let mut geo_linestrings: Vec<geo_types::LineString<f64>> = polygon
        .0
        .iter()
        .map(|l| l.0.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>().into())
        .collect();

    let interior: geo_types::LineString<f64>;
    if !geo_linestrings.is_empty() {
        interior = geo_linestrings.remove(0);
    } else {
        interior = geo_types::LineString(vec![]);
    }

    geo_types::Polygon::new(interior, geo_linestrings)
}

impl<'a> From<&'a Polygon> for geo_types::Geometry<f64> {
    fn from(polygon: &Polygon) -> Self {
        w_polygon_to_g_polygon(polygon).into()
    }
}

impl<'a> From<&'a MultiPoint> for geo_types::Geometry<f64> {
    fn from(multipoint: &MultiPoint) -> Self {
        let geo_multipoint: geo_types::MultiPoint<f64> = multipoint
            .0
            .iter()
            .filter_map(|p| p.0.as_ref())
            .map(|c| (c.x, c.y))
            .collect();

        geo_multipoint.into()
    }
}

impl<'a> From<&'a MultiPolygon> for geo_types::Geometry<f64> {
    fn from(multipolygon: &MultiPolygon) -> Self {
        let geo_multipolygon: geo_types::MultiPolygon<f64> = multipolygon
            .0
            .iter()
            .map(|p| w_polygon_to_g_polygon(p))
            .collect();

        geo_multipolygon.into()
    }
}

pub fn try_into_geometrycollection(
    geometrycollection: &GeometryCollection,
) -> Result<geo_types::Geometry<f64>, Error> {
    let geo_geometrycollection: geo_types::GeometryCollection<f64> = geometrycollection
        .0
        .iter()
        .map(|g| try_into_geometry(g))
        .collect::<Result<_, _>>()?;

    Ok(geo_types::Geometry::GeometryCollection(
        geo_geometrycollection,
    ))
}
