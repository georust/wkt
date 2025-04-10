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

use geo_traits::{CoordTrait, PointTrait};

use crate::to_wkt::write_point;
use crate::tokenizer::PeekableTokens;
use crate::types::coord::Coord;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point<T: WktNum> {
    dim: Dimension,
    coord: Option<Coord<T>>,
}

impl<T: WktNum> Point<T> {
    pub fn new(coord: Option<Coord<T>>, dim: Dimension) -> Self {
        Point { dim, coord }
    }
}

impl<T> From<Point<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: Point<T>) -> Self {
        Wkt::Point(value)
    }
}

impl<T> fmt::Display for Point<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_point(f, self)?)
    }
}

impl<T> FromTokens<T> for Point<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = <Coord<T> as FromTokens<T>>::from_tokens(tokens, dim);
        result.map(|coord| Point(Some(coord)))
    }
}

impl<T: WktNum> PointTrait for Point<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        self.coord.as_ref()
    }
}

impl<T: WktNum> PointTrait for &Point<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        self.coord.as_ref()
    }
}
#[cfg(test)]
mod tests {
    use super::{Coord, Point};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_point() {
        let wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
        let coord = match wkt {
            Wkt::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_z() {
        let wkt = Wkt::from_str("POINT Z(-117 33 10)").ok().unwrap();
        let coord = match wkt {
            Wkt::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(-117.0, coord.x);
        assert_eq!(33.0, coord.y);
        assert_eq!(Some(10.0), coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_z_one_word() {
        let wkt = Wkt::from_str("POINTZ(-117 33 10)").ok().unwrap();
        let coord = match wkt {
            Wkt::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(-117.0, coord.x);
        assert_eq!(33.0, coord.y);
        assert_eq!(Some(10.0), coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_whitespace() {
        let wkt: Wkt<f64> = Wkt::from_str(" \n\t\rPOINT \n\t\r( \n\r\t10 \n\t\r-20 \n\t\r) \n\t\r")
            .ok()
            .unwrap();
        let coord = match wkt {
            Wkt::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn invalid_points() {
        <Wkt<f64>>::from_str("POINT ()").err().unwrap();
        <Wkt<f64>>::from_str("POINT (10)").err().unwrap();
        <Wkt<f64>>::from_str("POINT 10").err().unwrap();
    }

    #[test]
    fn write_empty_point() {
        let point: Point<f64> = Point(None);

        assert_eq!("POINT EMPTY", format!("{}", point));
    }

    #[test]
    fn write_2d_point() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: None,
            m: None,
        }));

        assert_eq!("POINT(10.12345 20.67891)", format!("{}", point));
    }

    #[test]
    fn write_point_with_z_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: Some(-32.56455),
            m: None,
        }));

        assert_eq!("POINT Z(10.12345 20.67891 -32.56455)", format!("{}", point));
    }

    #[test]
    fn write_point_with_m_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: None,
            m: Some(10.),
        }));

        assert_eq!("POINT M(10.12345 20.67891 10)", format!("{}", point));
    }

    #[test]
    fn write_point_with_zm_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: Some(-32.56455),
            m: Some(10.),
        }));

        assert_eq!(
            "POINT ZM(10.12345 20.67891 -32.56455 10)",
            format!("{}", point)
        );
    }
}
