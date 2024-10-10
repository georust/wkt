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

use crate::tokenizer::{PeekableTokens, Token};
use crate::types::Dimension;
use crate::{FromTokens, Wkt, WktNum};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryCollection<T: WktNum>(pub Vec<Wkt<T>>);

impl<T> From<GeometryCollection<T>> for Wkt<T>
where
    T: WktNum,
{
    fn from(value: GeometryCollection<T>) -> Self {
        Wkt::GeometryCollection(value)
    }
}

macro_rules! impl_display {
    ($t: ident) => {
        impl fmt::Display for GeometryCollection<$t> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                if self.0.is_empty() {
                    f.write_str("GEOMETRYCOLLECTION EMPTY")
                } else {
                    let strings = self
                        .0
                        .iter()
                        .map(|geometry| format!("{}", geometry))
                        .collect::<Vec<_>>()
                        .join(",");

                    write!(f, "GEOMETRYCOLLECTION({})", strings)
                }
            }
        }
    };
}

impl_display!(f32);
impl_display!(f64);
impl_display!(u8);
impl_display!(u16);
impl_display!(u32);
impl_display!(u64);
impl_display!(usize);
impl_display!(i8);
impl_display!(i16);
impl_display!(i32);
impl_display!(i64);
impl_display!(isize);

impl<T> FromTokens<T> for GeometryCollection<T>
where
    T: WktNum + FromStr + Default,
{
    // Unsure if the dimension should be used in parsing GeometryCollection; is it
    // GEOMETRYCOLLECTION ( POINT Z (...) , POINT ZM (...))
    // or does a geometry collection have a known dimension?
    fn from_tokens(tokens: &mut PeekableTokens<T>, _dim: Dimension) -> Result<Self, &'static str> {
        let mut items = Vec::new();

        let word = match tokens.next().transpose()? {
            Some(Token::Word(w)) => w,
            _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
        };

        let item = Wkt::from_word_and_tokens(&word, tokens)?;
        items.push(item);

        while let Some(&Ok(Token::Comma)) = tokens.peek() {
            tokens.next(); // throw away comma

            let word = match tokens.next().transpose()? {
                Some(Token::Word(w)) => w,
                _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
            };

            let item = Wkt::from_word_and_tokens(&word, tokens)?;
            items.push(item);
        }

        Ok(GeometryCollection(items))
    }
}

#[cfg(test)]
mod tests {
    use super::GeometryCollection;
    use crate::types::*;
    use crate::Wkt;
    use std::str::FromStr;

    #[test]
    fn basic_geometrycollection() {
        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4)))")
            .ok()
            .unwrap();
        let items = match wkt {
            Wkt::GeometryCollection(GeometryCollection(items)) => items,
            _ => unreachable!(),
        };
        assert_eq!(1, items.len());
    }

    #[test]
    fn complex_geometrycollection() {
        let wkt: Wkt<f64> = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4),LINESTRING(4 6,7 10)))")
            .ok()
            .unwrap();
        let items = match wkt {
            Wkt::GeometryCollection(GeometryCollection(items)) => items,
            _ => unreachable!(),
        };
        assert_eq!(2, items.len());
    }

    #[test]
    fn write_empty_geometry_collection() {
        let geometry_collection: GeometryCollection<f64> = GeometryCollection(vec![]);

        assert_eq!(
            "GEOMETRYCOLLECTION EMPTY",
            format!("{}", geometry_collection)
        );
    }

    #[test]
    fn write_geometry_collection() {
        let point = Wkt::Point(Point(Some(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        })));

        let multipoint = Wkt::MultiPoint(MultiPoint(vec![
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
        ]));

        let linestring = Wkt::LineString(LineString(vec![
            Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            },
            Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            },
        ]));

        let polygon = Wkt::Polygon(Polygon(vec![LineString(vec![
            Coord {
                x: 0.,
                y: 0.,
                z: None,
                m: None,
            },
            Coord {
                x: 20.,
                y: 40.,
                z: None,
                m: None,
            },
            Coord {
                x: 40.,
                y: 0.,
                z: None,
                m: None,
            },
            Coord {
                x: 0.,
                y: 0.,
                z: None,
                m: None,
            },
        ])]));

        let multilinestring = Wkt::MultiLineString(MultiLineString(vec![
            LineString(vec![
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
            ]),
            LineString(vec![
                Coord {
                    x: 50.5,
                    y: 60.6,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.7,
                    y: 80.8,
                    z: None,
                    m: None,
                },
            ]),
        ]));

        let multipolygon = Wkt::MultiPolygon(MultiPolygon(vec![
            Polygon(vec![LineString(vec![
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
            ])]),
            Polygon(vec![LineString(vec![
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 45.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 45.,
                    y: 30.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
            ])]),
        ]));

        let geometrycollection = GeometryCollection(vec![
            point,
            multipoint,
            linestring,
            polygon,
            multilinestring,
            multipolygon,
        ]);

        assert_eq!(
            "GEOMETRYCOLLECTION(\
             POINT(10.0 20.0),\
             MULTIPOINT((10.1 20.2),(30.3 40.4)),\
             LINESTRING(10.0 20.0,30.0 40.0),\
             POLYGON((0 0,20 40,40 0,0 0)),\
             MULTILINESTRING((10.1 20.2,30.3 40.4),(50.5 60.6,70.7 80.8)),\
             MULTIPOLYGON(((0 0,20 40,40 0,0 0)),((40 40,20 45,45 30,40 40)))\
             )",
            format!("{}", geometrycollection)
        );
    }
}
