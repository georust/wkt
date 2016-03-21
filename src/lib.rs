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

// #![feature(test)]

#[cfg(feature = "geo-interop")]
mod towkt;

mod wkt;

#[cfg(feature = "geo-interop")]
pub use towkt::ToWkt;

// extern crate test;


/// Coordinate (x, y)
pub type Coord = (f64, f64);

pub type PointType = Option<Coord>;
pub type LineStringType = Vec<Coord>;
pub type PolygonType = Vec<LineStringType>;
pub type PolyhedralSurfaceType = Vec<PolygonType>;
pub type MultiPointType = Vec<PointType>;
pub type MultiLineStringType = Vec<LineStringType>;
pub type MultiPolygonType = Vec<PolygonType>;
pub type GeometryCollectionType = Vec<Geometry>;

pub enum Geometry {
    Point(PointType),
    LineString(LineStringType),
    Polygon(PolygonType),
    PolyhedralSurface(PolyhedralSurfaceType),
    Triangle(PolygonType),
    Tin(PolyhedralSurfaceType),
    MultiPoint(MultiPointType),
    MultiLineString(MultiLineStringType),
    MultiPolygon(MultiPolygonType),
    GeometryCollection(GeometryCollectionType),
}

pub struct Wkt(Geometry);

impl std::str::FromStr for Wkt {
    type Err = ();  // TODO: this should be an actual error type
    fn from_str(s: &str) -> Result<Self, ()> {
        match wkt::parse_GeometryTaggedText(s) {
            Ok(geom) => Ok(Wkt(geom)),
            Err(..) => Err(()),
        }
    }
}



#[cfg(test)]
mod tests {
    use {Wkt, Geometry};
    use std::str::FromStr;

    #[test]
    fn empty_string() {
        assert!(Wkt::from_str("").is_err());
    }

    #[test]
    fn empty_items() {
        let wkt = Wkt::from_str("POINT EMPTY").unwrap();
        match wkt.0 {
            Geometry::Point(None) => (),
            _ => unreachable!(),
        };

        let wkt = Wkt::from_str("MULTIPOLYGON EMPTY").unwrap();
        match wkt.0 {
            Geometry::MultiPolygon(polygons) =>
                assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[test]
    fn basic_polygon() {
        let wkt = Wkt::from_str("POLYGON ((8 4, 4 0, 0 4, 8 4), (7 3, 4 1, 1 4, 7 3))").unwrap();
        let lines = match wkt.0 {
            Geometry::Polygon(lines) => lines,
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn basic_point() {
        let wkt = Wkt::from_str("POINT (10 -20)").unwrap();
        let coord = match wkt.0 {
            Geometry::Point(Some(coord)) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.0);
        assert_eq!(-20.0, coord.1);
        //assert_eq!(None, coord.z);
        //assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_whitespace() {
        let wkt = Wkt::from_str(" \n\t\rPOINT \n\t\r( \n\r\t10 \n\t\r-20 \n\t\r) \n\t\r").unwrap();
        let coord = match wkt.0 {
            Geometry::Point(Some(coord)) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.0);
        assert_eq!(-20.0, coord.1);
        //assert_eq!(None, coord.z);
        //assert_eq!(None, coord.m);
    }

    #[test]
    fn invalid_points() {
        Wkt::from_str("POINT ()").err().unwrap();
        Wkt::from_str("POINT (10)").err().unwrap();
        Wkt::from_str("POINT 10").err().unwrap();
        Wkt::from_str("POINT (10 -20 40)").err().unwrap();
    }

    #[test]
    fn basic_multipolygon() {
        let wkt = Wkt::from_str("MULTIPOLYGON (((8 4)), ((4 0)))").unwrap();
        let polygons = match wkt.0 {
            Geometry::MultiPolygon(polygons) => polygons,
            _ => unreachable!(),
        };
        assert_eq!(2, polygons.len());
    }

    #[test]
    fn basic_multipoint() {
        let wkt = Wkt::from_str("MULTIPOINT ((8 4), (4 0))").unwrap();
        let points = match wkt.0 {
            Geometry::MultiPoint(points) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn basic_multilinestring() {
        let wkt = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))").unwrap();
        let lines = match wkt.0 {
            Geometry::MultiLineString(lines) => lines,
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn basic_linestring() {
        let wkt = Wkt::from_str("LINESTRING (10 -20, -0 -0.5)").unwrap();
        let coords = match wkt.0 {
            Geometry::LineString(coords) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(10.0, coords[0].0);
        assert_eq!(-20.0, coords[0].1);
        //assert_eq!(None, coords[0].z);
        //assert_eq!(None, coords[0].m);

        assert_eq!(0.0, coords[1].0);
        assert_eq!(-0.5, coords[1].1);
        //assert_eq!(None, coords[1].z);
        //assert_eq!(None, coords[1].m);
    }

    #[test]
    fn basic_geometrycollection() {
        let wkt = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4))").unwrap();
        let items = match wkt.0 {
            Geometry::GeometryCollection(items) => items,
            _ => unreachable!(),
        };
        assert_eq!(1, items.len());
    }
}
