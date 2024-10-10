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

macro_rules! impl_display_for_float {
    ($t: ident) => {
        impl fmt::Display for Coord<$t> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                let mut buffer = ryu::Buffer::new();
                let x = buffer.format(self.x);

                let mut buffer = ryu::Buffer::new();
                let y = buffer.format(self.y);

                write!(f, "{} {}", x, y)?;
                if let Some(z) = self.z {
                    let mut buffer = ryu::Buffer::new();
                    let z = buffer.format(z);
                    write!(f, " {}", z)?;
                }
                if let Some(m) = self.m {
                    let mut buffer = ryu::Buffer::new();
                    let m = buffer.format(m);
                    write!(f, " {}", m)?;
                }
                Ok(())
            }
        }
    };
}

macro_rules! impl_display_for_int {
    ($t: ident) => {
        impl fmt::Display for Coord<$t> {
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
    };
}

impl_display_for_float!(f32);
impl_display_for_float!(f64);

impl_display_for_int!(u8);
impl_display_for_int!(u16);
impl_display_for_int!(u32);
impl_display_for_int!(u64);
impl_display_for_int!(usize);
impl_display_for_int!(i8);
impl_display_for_int!(i16);
impl_display_for_int!(i32);
impl_display_for_int!(i64);
impl_display_for_int!(isize);

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

        assert_eq!("10.1 20.2 10.0", format!("{}", coord));
    }

    #[test]
    fn write_3d_coord_with_linear_referencing_system() {
        let coord = Coord {
            x: 10.1,
            y: 20.2,
            z: Some(-30.3),
            m: Some(10.),
        };

        assert_eq!("10.1 20.2 -30.3 10.0", format!("{}", coord));
    }
}
