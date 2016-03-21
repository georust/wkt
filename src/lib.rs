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

use std::ascii::AsciiExt;
use std::default::Default;

use types::GeometryCollection;
use types::LineString;
use types::Point;
use types::Polygon;
use types::MultiPoint;
use types::MultiLineString;
use types::MultiPolygon;

#[cfg(feature = "geo-interop")]
mod towkt;
pub mod types;

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
pub type GeometryCollectionType = Vec<New>;

pub enum New {
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


pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}

pub struct Wkt {
    pub items: Vec<Geometry>
}

impl Wkt {
    fn new() -> Self {
        Wkt {items: vec![]}
    }

    fn add_item(&mut self, item: Geometry) {
        self.items.push(item);
    }
}




#[cfg(test)]
mod tests {
    use {Wkt, Geometry};
    use types::{MultiPolygon, Point};
    use test::Bencher;

    #[test]
    fn empty_string() {
        let wkt = Wkt::from_str("").ok().unwrap();
        assert_eq!(0, wkt.items.len());
    }

    #[test]
    fn empty_items() {
        let mut wkt = Wkt::from_str("POINT EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };

        let mut wkt = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::MultiPolygon(MultiPolygon(polygons)) =>
                assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[bench]
    fn tmp(_: &mut Bencher) {
    }
}
