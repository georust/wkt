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

use crate::tokenizer::{PeekableTokens, Token};
use crate::types::Dimension;
use crate::{FromTokens, WktNum};
use std::str::FromStr;

/// A parsed coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coord<T: WktNum = f64> {
    pub x: T,
    pub y: T,
    pub z: Option<T>,
    pub m: Option<T>,
}

impl<T: WktNum> Coord<T> {
    /// Return the [Dimension] of this coord.
    pub fn dimension(&self) -> Dimension {
        match (self.z.is_some(), self.m.is_some()) {
            (true, true) => Dimension::XYZM,
            (true, false) => Dimension::XYZ,
            (false, true) => Dimension::XYM,
            (false, false) => Dimension::XY,
        }
    }
}

impl<T> FromTokens<T> for Coord<T>
where
    T: WktNum + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>, dim: Dimension) -> Result<Self, &'static str> {
        let x = match tokens.next().transpose()? {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected a number for the X coordinate"),
        };
        let y = match tokens.next().transpose()? {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected a number for the Y coordinate"),
        };

        let mut z = None;
        let mut m = None;

        match dim {
            Dimension::XY => (),
            Dimension::XYZ => match tokens.next().transpose()? {
                Some(Token::Number(n)) => {
                    z = Some(n);
                }
                _ => return Err("Expected a number for the Z coordinate"),
            },
            Dimension::XYM => match tokens.next().transpose()? {
                Some(Token::Number(n)) => {
                    m = Some(n);
                }
                _ => return Err("Expected a number for the M coordinate"),
            },
            Dimension::XYZM => {
                match tokens.next().transpose()? {
                    Some(Token::Number(n)) => {
                        z = Some(n);
                    }
                    _ => return Err("Expected a number for the Z coordinate"),
                }
                match tokens.next().transpose()? {
                    Some(Token::Number(n)) => {
                        m = Some(n);
                    }
                    _ => return Err("Expected a number for the M coordinate"),
                }
            }
        }

        Ok(Coord { x, y, z, m })
    }

    fn new_empty(_dim: Dimension) -> Self {
        unreachable!("empty coord does not exist in WKT")
    }
}

#[cfg(feature = "geo-traits_0_2")]
impl<T: WktNum> geo_traits_0_2::CoordTrait for Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits_0_2::Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}

#[cfg(feature = "geo-traits_0_2")]
impl<T: WktNum> geo_traits_0_2::CoordTrait for &Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits_0_2::Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}

#[cfg(feature = "geo-traits_0_3")]
impl<T: WktNum> geo_traits_0_3::CoordTrait for Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits_0_3::Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}

#[cfg(feature = "geo-traits_0_3")]
impl<T: WktNum> geo_traits_0_3::CoordTrait for &Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits_0_3::Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}
