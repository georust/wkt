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

use geo_traits::{MultiPointTrait, PointTrait};

use crate::tokenizer::PeekableTokens;
use crate::types::point::Point;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiPoint<T: WktNum>(pub Vec<Point<T>>);

impl<T> From<MultiPoint<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: MultiPoint<T>) -> Self {
        Wkt::MultiPoint(value)
    }
}

impl<T> fmt::Display for MultiPoint<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("MULTIPOINT EMPTY")
        } else {
            let strings = self
                .0
                .iter()
                .filter_map(|p| p.0.as_ref())
                .map(|c| format!("({} {})", c.x, c.y))
                .collect::<Vec<_>>()
                .join(",");

            write!(f, "MULTIPOINT({})", strings)
        }
    }
}

impl<T> FromTokens<T> for MultiPoint<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <Point<T> as FromTokens<T>>::from_tokens_with_optional_parens,
            tokens,
            dim,
        );
        result.map(MultiPoint)
    }
}

impl<T: WktNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type PointType<'a> = &'a Point<T> where Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.0.is_empty() {
            geo_traits::Dimensions::XY
        } else {
            self.0[0].dim()
        }
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        &self.0[i]
    }
}

impl<T: WktNum> MultiPointTrait for &MultiPoint<T> {
    type T = T;
    type PointType<'a> = &'a Point<T> where Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.0.is_empty() {
            geo_traits::Dimensions::XY
        } else {
            self.0[0].dim()
        }
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        &self.0[i]
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiPoint, Point};
    use crate::types::Coord;
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ((8 4), (4 0))").ok().unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn basic_multipoint_zm() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ZM (0 0 4 3, 1 2 4 5)")
            .ok()
            .unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());

        assert_eq!(0.0, points[0].0.as_ref().unwrap().x);
        assert_eq!(0.0, points[0].0.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[0].0.as_ref().unwrap().z);
        assert_eq!(Some(3.0), points[0].0.as_ref().unwrap().m);

        assert_eq!(1.0, points[1].0.as_ref().unwrap().x);
        assert_eq!(2.0, points[1].0.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[1].0.as_ref().unwrap().z);
        assert_eq!(Some(5.0), points[1].0.as_ref().unwrap().m);
    }

    #[test]
    fn basic_multipoint_zm_extra_parents() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ZM ((0 0 4 3), (1 2 4 5))")
            .ok()
            .unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());

        assert_eq!(0.0, points[0].0.as_ref().unwrap().x);
        assert_eq!(0.0, points[0].0.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[0].0.as_ref().unwrap().z);
        assert_eq!(Some(3.0), points[0].0.as_ref().unwrap().m);

        assert_eq!(1.0, points[1].0.as_ref().unwrap().x);
        assert_eq!(2.0, points[1].0.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[1].0.as_ref().unwrap().z);
        assert_eq!(Some(5.0), points[1].0.as_ref().unwrap().m);
    }
    #[test]
    fn postgis_style_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT (8 4, 4 0)").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn mixed_parens_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT (8 4, (4 0))").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn empty_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT EMPTY").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(0, points.len());
    }

    #[test]
    fn write_empty_multipoint() {
        let multipoint: MultiPoint<f64> = MultiPoint(vec![]);

        assert_eq!("MULTIPOINT EMPTY", format!("{}", multipoint));
    }

    #[test]
    fn write_multipoint() {
        let multipoint = MultiPoint(vec![
            Point(Some(Coord {
                x: 10.1,
                y: 20.2,
                z: None,
                m: None,
            })),
            Point(Some(Coord {
                x: 30.3,
                y: 40.4,
                z: None,
                m: None,
            })),
        ]);

        assert_eq!(
            "MULTIPOINT((10.1 20.2),(30.3 40.4))",
            format!("{}", multipoint)
        );
    }
}
