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

use geo_traits::MultiPointTrait;

use crate::to_wkt::write_multi_point;
use crate::tokenizer::PeekableTokens;
use crate::types::point::Point;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiPoint<T: WktNum> {
    pub(crate) dim: Dimension,
    pub(crate) points: Vec<Point<T>>,
}

impl<T: WktNum> MultiPoint<T> {
    pub fn new(points: Vec<Point<T>>, dim: Dimension) -> Self {
        MultiPoint { dim, points }
    }

    /// Create a new empty MultiPoint.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new MultiPoint from a non-empty sequence of [Point].
    ///
    /// This will infer the dimension from the first point, and will not validate that all
    /// points have the same dimension.
    ///
    /// ## Panics
    ///
    /// If the input iterator is empty.
    pub fn from_points(points: impl IntoIterator<Item = Point<T>>) -> Self {
        let points = points.into_iter().collect::<Vec<_>>();
        let dim = points[0].dim;
        Self::new(points, dim)
    }
}

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
        Ok(write_multi_point(f, self)?)
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
        result.map(|points| MultiPoint { points, dim })
    }
}

impl<T: WktNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type PointType<'a>
        = &'a Point<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_points(&self) -> usize {
        self.points.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.points.get_unchecked(i)
    }
}

impl<T: WktNum> MultiPointTrait for &MultiPoint<T> {
    type T = T;
    type PointType<'a>
        = &'a Point<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_points(&self) -> usize {
        self.points.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.points.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiPoint, Point};
    use crate::types::{Coord, Dimension};
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ((8 4), (4 0))").ok().unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
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
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());

        assert_eq!(0.0, points[0].coord.as_ref().unwrap().x);
        assert_eq!(0.0, points[0].coord.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[0].coord.as_ref().unwrap().z);
        assert_eq!(Some(3.0), points[0].coord.as_ref().unwrap().m);

        assert_eq!(1.0, points[1].coord.as_ref().unwrap().x);
        assert_eq!(2.0, points[1].coord.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[1].coord.as_ref().unwrap().z);
        assert_eq!(Some(5.0), points[1].coord.as_ref().unwrap().m);
    }

    #[test]
    fn basic_multipoint_zm_extra_parents() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ZM ((0 0 4 3), (1 2 4 5))")
            .ok()
            .unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());

        assert_eq!(0.0, points[0].coord.as_ref().unwrap().x);
        assert_eq!(0.0, points[0].coord.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[0].coord.as_ref().unwrap().z);
        assert_eq!(Some(3.0), points[0].coord.as_ref().unwrap().m);

        assert_eq!(1.0, points[1].coord.as_ref().unwrap().x);
        assert_eq!(2.0, points[1].coord.as_ref().unwrap().y);
        assert_eq!(Some(4.0), points[1].coord.as_ref().unwrap().z);
        assert_eq!(Some(5.0), points[1].coord.as_ref().unwrap().m);
    }
    #[test]
    fn postgis_style_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT (8 4, 4 0)").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn mixed_parens_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT (8 4, (4 0))").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn empty_multipoint() {
        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT EMPTY").unwrap();
        let points = match wkt {
            Wkt::MultiPoint(MultiPoint { points, dim: _ }) => points,
            _ => unreachable!(),
        };
        assert_eq!(0, points.len());
    }

    #[test]
    fn write_empty_multipoint() {
        let multipoint: MultiPoint<f64> = MultiPoint::empty(Dimension::XY);
        assert_eq!("MULTIPOINT EMPTY", format!("{}", multipoint));

        let multipoint: MultiPoint<f64> = MultiPoint::empty(Dimension::XYZ);
        assert_eq!("MULTIPOINT Z EMPTY", format!("{}", multipoint));

        let multipoint: MultiPoint<f64> = MultiPoint::empty(Dimension::XYM);
        assert_eq!("MULTIPOINT M EMPTY", format!("{}", multipoint));

        let multipoint: MultiPoint<f64> = MultiPoint::empty(Dimension::XYZM);
        assert_eq!("MULTIPOINT ZM EMPTY", format!("{}", multipoint));
    }

    #[test]
    fn write_multipoint() {
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
        ]);

        assert_eq!(
            "MULTIPOINT((10.1 20.2),(30.3 40.4))",
            format!("{}", multipoint)
        );
    }
}
