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

use geo_traits::LineStringTrait;

use crate::to_wkt::write_linestring;
use crate::tokenizer::PeekableTokens;
use crate::types::coord::Coord;
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LineString<T: WktNum> {
    pub(crate) dim: Dimension,
    pub(crate) coords: Vec<Coord<T>>,
}

impl<T: WktNum> LineString<T> {
    pub fn new(coords: Vec<Coord<T>>, dim: Dimension) -> Self {
        LineString { dim, coords }
    }

    /// Create a new empty LineString.
    pub fn empty(dim: Dimension) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new LineString from a non-empty sequence of [Coord].
    ///
    /// This will infer the dimension from the first coordinate, and will not validate that all
    /// coordinates have the same dimension.
    ///
    /// ## Panics
    ///
    /// If the input iterator is empty.
    pub fn from_coords(coords: impl IntoIterator<Item = Coord<T>>) -> Self {
        let coords = coords.into_iter().collect::<Vec<_>>();
        let dim = coords[0].dimension();
        Self::new(coords, dim)
    }
}

impl<T> From<LineString<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: LineString<T>) -> Self {
        Wkt::LineString(value)
    }
}

impl<T> FromTokens<T> for LineString<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(<Coord<T> as FromTokens<T>>::from_tokens, tokens, dim);
        result.map(LineString)
    }
}

impl<T> fmt::Display for LineString<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(write_linestring(f, self)?)
    }
}

impl<T: WktNum> LineStringTrait for LineString<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_coords(&self) -> usize {
        self.coords.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.coords.get_unchecked(i)
    }
}

impl<T: WktNum> LineStringTrait for &LineString<T> {
    type T = T;
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        self.dim.into()
    }

    fn num_coords(&self) -> usize {
        self.coords.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.coords.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{Coord, LineString};
    use crate::types::Dimension;
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_linestring() {
        let wkt: Wkt<f64> = Wkt::from_str("LINESTRING (10 -20, -0 -0.5)").ok().unwrap();
        let coords = match wkt {
            Wkt::LineString(LineString { coords, dim: _ }) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(10.0, coords[0].x);
        assert_eq!(-20.0, coords[0].y);
        assert_eq!(None, coords[0].z);
        assert_eq!(None, coords[0].m);

        assert_eq!(0.0, coords[1].x);
        assert_eq!(-0.5, coords[1].y);
        assert_eq!(None, coords[1].z);
        assert_eq!(None, coords[1].m);
    }

    #[test]
    fn basic_linestring_z() {
        let wkt = Wkt::from_str("LINESTRING Z (-117 33 2, -116 34 4)")
            .ok()
            .unwrap();
        let coords = match wkt {
            Wkt::LineString(LineString { coords, dim: _ }) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(-117.0, coords[0].x);
        assert_eq!(33.0, coords[0].y);
        assert_eq!(Some(2.0), coords[0].z);
        assert_eq!(None, coords[0].m);

        assert_eq!(-116.0, coords[1].x);
        assert_eq!(34.0, coords[1].y);
        assert_eq!(Some(4.0), coords[1].z);
        assert_eq!(None, coords[1].m);
    }

    #[test]
    fn basic_linestring_m() {
        let wkt = Wkt::from_str("LINESTRING M (-117 33 2, -116 34 4)")
            .ok()
            .unwrap();
        let coords = match wkt {
            Wkt::LineString(LineString { coords, dim: _ }) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(-117.0, coords[0].x);
        assert_eq!(33.0, coords[0].y);
        assert_eq!(None, coords[0].z);
        assert_eq!(Some(2.0), coords[0].m);

        assert_eq!(-116.0, coords[1].x);
        assert_eq!(34.0, coords[1].y);
        assert_eq!(None, coords[1].z);
        assert_eq!(Some(4.0), coords[1].m);
    }

    #[test]
    fn basic_linestring_zm() {
        let wkt = Wkt::from_str("LINESTRING ZM (-117 33 2 3, -116 34 4 5)")
            .ok()
            .unwrap();
        let coords = match wkt {
            Wkt::LineString(LineString { coords, dim: _ }) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(-117.0, coords[0].x);
        assert_eq!(33.0, coords[0].y);
        assert_eq!(Some(2.0), coords[0].z);
        assert_eq!(Some(3.0), coords[0].m);

        assert_eq!(-116.0, coords[1].x);
        assert_eq!(34.0, coords[1].y);
        assert_eq!(Some(4.0), coords[1].z);
        assert_eq!(Some(5.0), coords[1].m);
    }

    #[test]
    fn basic_linestring_zm_one_word() {
        let wkt = Wkt::from_str("LINESTRINGZM (-117 33 2 3, -116 34 4 5)")
            .ok()
            .unwrap();
        let coords = match wkt {
            Wkt::LineString(LineString { coords, dim: _ }) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(-117.0, coords[0].x);
        assert_eq!(33.0, coords[0].y);
        assert_eq!(Some(2.0), coords[0].z);
        assert_eq!(Some(3.0), coords[0].m);

        assert_eq!(-116.0, coords[1].x);
        assert_eq!(34.0, coords[1].y);
        assert_eq!(Some(4.0), coords[1].z);
        assert_eq!(Some(5.0), coords[1].m);
    }

    #[test]
    fn write_empty_linestring() {
        let linestring: LineString<f64> = LineString::empty(Dimension::XY);

        assert_eq!("LINESTRING EMPTY", format!("{}", linestring));
    }

    #[test]
    fn write_linestring() {
        let linestring = LineString::from_coords([
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
        ]);

        assert_eq!("LINESTRING(10.1 20.2,30.3 40.4)", format!("{}", linestring));
    }
}
