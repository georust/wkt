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

use geo_traits::PointTrait;

use crate::tokenizer::{PeekableTokens, Token};
use crate::types::Dimension;
use crate::{FromTokens, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coord<T>
where
    T: WktNum,
{
    pub x: T,
    pub y: T,
    pub z: Option<T>,
    pub m: Option<T>,
}

impl<T> fmt::Display for Coord<T>
where
    T: WktNum + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} {}", self.x, self.y)?;
        if let Some(z) = self.z {
            write!(f, " {}", z)?;
        }
        if let Some(m) = self.m {
            write!(f, " {}", m)?;
        }
        Ok(())
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
}

impl<T: WktNum> PointTrait for Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits::Dimensions {
        match (self.z.is_some(), self.m.is_some()) {
            (true, true) => geo_traits::Dimensions::XYZM,
            (true, false) => geo_traits::Dimensions::XYZ,
            (false, true) => geo_traits::Dimensions::XYM,
            (false, false) => geo_traits::Dimensions::XYM,
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_unchecked(&self, n: usize) -> Self::T {
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

impl<T: WktNum> PointTrait for &Coord<T> {
    type T = T;

    fn dim(&self) -> geo_traits::Dimensions {
        match (self.z.is_some(), self.m.is_some()) {
            (true, true) => geo_traits::Dimensions::XYZM,
            (true, false) => geo_traits::Dimensions::XYZ,
            (false, true) => geo_traits::Dimensions::XYM,
            (false, false) => geo_traits::Dimensions::XYM,
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_unchecked(&self, n: usize) -> Self::T {
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

#[cfg(test)]
mod tests {
    use super::Coord;

    #[test]
    fn write_2d_coord() {
        let coord = Coord {
            x: 10.1,
            y: 20.2,
            z: None,
            m: None,
        };

        assert_eq!("10.1 20.2", format!("{}", coord));
    }

    #[test]
    fn write_3d_coord() {
        let coord = Coord {
            x: 10.1,
            y: 20.2,
            z: Some(-30.3),
            m: None,
        };

        assert_eq!("10.1 20.2 -30.3", format!("{}", coord));
    }

    #[test]
    fn write_2d_coord_with_linear_referencing_system() {
        let coord = Coord {
            x: 10.1,
            y: 20.2,
            z: None,
            m: Some(10.),
        };

        assert_eq!("10.1 20.2 10", format!("{}", coord));
    }

    #[test]
    fn write_3d_coord_with_linear_referencing_system() {
        let coord = Coord {
            x: 10.1,
            y: 20.2,
            z: Some(-30.3),
            m: Some(10.),
        };

        assert_eq!("10.1 20.2 -30.3 10", format!("{}", coord));
    }
}
