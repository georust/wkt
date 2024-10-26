use core::fmt;
use std::fmt::Write;

use geo_traits::{
    CoordTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait,
    PointTrait, PolygonTrait,
};
use geo_types::CoordNum;

use crate::WktNum;

pub fn coord_to_wkt<T: CoordNum + WktNum + fmt::Display, G: CoordTrait<T = T>, W: Write>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    match g.dim() {
        geo_traits::Dimensions::Xy => {
            write!(f, "{} {}", g.x(), g.y())?;
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Xym => {
            write!(f, "{} {} {}", g.x(), g.y(), g.nth_unchecked(2))?;
        }
        geo_traits::Dimensions::Xyzm => {
            write!(
                f,
                "{} {} {} {}",
                g.x(),
                g.y(),
                g.nth_unchecked(2),
                g.nth_unchecked(3)
            )?;
        }
        geo_traits::Dimensions::Unknown(_) => todo!(),
    };

    Ok(())
}

pub fn point_to_wkt<T: CoordNum + WktNum + fmt::Display, G: PointTrait<T = T>, W: Write>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy => f.write_str("POINT"),
        geo_traits::Dimensions::Xyz => f.write_str("POINT Z"),
        geo_traits::Dimensions::Xym => f.write_str("POINT M"),
        geo_traits::Dimensions::Xyzm => f.write_str("POINT ZM"),
        geo_traits::Dimensions::Unknown(_) => todo!(),
    }?;
    if let Some(coord) = g.coord() {
        f.write_char('(')?;
        coord_to_wkt(&coord, f)?;
        f.write_char(')')?;
        Ok(())
    } else {
        f.write_str(" EMPTY")
    }
}

pub fn linestring_to_wkt<
    T: CoordNum + WktNum + fmt::Display,
    G: LineStringTrait<T = T>,
    W: Write,
>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy => f.write_str("LINESTRING"),
        geo_traits::Dimensions::Xyz => f.write_str("LINESTRING Z"),
        geo_traits::Dimensions::Xym => f.write_str("LINESTRING M"),
        geo_traits::Dimensions::Xyzm => f.write_str("LINESTRING ZM"),
        geo_traits::Dimensions::Unknown(_) => todo!(),
    }?;
    if g.num_coords() == 0 {
        f.write_str(" EMPTY")
    } else {
        // add_coords(linestring.coords(), f)?;
        let strings = g
            .coords()
            .map(|c| {
                let mut s = String::new();
                coord_to_wkt(&c, &mut s)?;
                Ok(s)
            })
            .collect::<Result<Vec<_>, std::fmt::Error>>()?
            .join(",");
        write!(f, "({})", strings)
    }
}

pub fn polygon_to_wkt<T: CoordNum + WktNum + fmt::Display, G: PolygonTrait<T = T>, W: Write>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy => f.write_str("POLYGON"),
        geo_traits::Dimensions::Xyz => f.write_str("POLYGON Z"),
        geo_traits::Dimensions::Xym => f.write_str("POLYGON M"),
        geo_traits::Dimensions::Xyzm => f.write_str("POLYGON ZM"),
        geo_traits::Dimensions::Unknown(_) => todo!(),
    }?;
    if let Some(exterior) = g.exterior() {
        let exterior_string = exterior
            .coords()
            .map(|c| {
                let mut s = String::new();
                coord_to_wkt(&c, &mut s)?;
                Ok(s)
            })
            .collect::<Result<Vec<_>, std::fmt::Error>>()?
            .join(",");

        if g.num_interiors() == 0 {
            write!(f, "({})", exterior_string)
        } else {
            let interior_string = g
                .interiors()
                .map(|ring| {
                    let s = ring
                        .coords()
                        .map(|c| {
                            let mut s = String::new();
                            coord_to_wkt(&c, &mut s)?;
                            Ok(s)
                        })
                        .collect::<Result<Vec<_>, std::fmt::Error>>()?
                        .join(",");
                    Ok(s)
                })
                .collect::<Result<Vec<_>, std::fmt::Error>>()?
                .join("),(");
            write!(f, "({},({}))", exterior_string, interior_string)
        }
    } else {
        f.write_str(" EMPTY")
    }
}

pub fn multipoint_to_wkt<
    T: CoordNum + WktNum + fmt::Display,
    G: MultiPointTrait<T = T>,
    W: Write,
>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy => f.write_str("MULTIPOINT"),
        geo_traits::Dimensions::Xyz => f.write_str("MULTIPOINT Z"),
        geo_traits::Dimensions::Xym => f.write_str("MULTIPOINT M"),
        geo_traits::Dimensions::Xyzm => f.write_str("MULTIPOINT ZM"),
        geo_traits::Dimensions::Unknown(_) => todo!(),
    }?;
    if g.num_points() == 0 {
        f.write_str(" EMPTY")
    } else {
        let strings = g
            .points()
            .map(|c| {
                let mut s = String::new();
                // Assume no empty points within this MultiPoint
                coord_to_wkt(&c.coord().unwrap(), &mut s)?;
                Ok(s)
            })
            .collect::<Result<Vec<_>, std::fmt::Error>>()?
            .join(",");
        write!(f, "({})", strings)
    }
}

pub fn multilinestring_to_wkt<
    T: CoordNum + WktNum + fmt::Display,
    G: MultiLineStringTrait<T = T>,
    W: Write,
>(
    g: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy => f.write_str("MULTILINESTRING"),
        geo_traits::Dimensions::Xyz => f.write_str("MULTILINESTRING Z"),
        geo_traits::Dimensions::Xym => f.write_str("MULTILINESTRING M"),
        geo_traits::Dimensions::Xyzm => f.write_str("MULTILINESTRING ZM"),
        geo_traits::Dimensions::Unknown(_) => todo!(),
    }?;

    if g.num_line_strings() == 0 {
        f.write_str(" EMPTY")
    } else {
        let strings = g
            .line_strings()
            .map(|ring| {
                let s = ring
                    .coords()
                    .map(|c| {
                        let mut s = String::new();
                        coord_to_wkt(&c, &mut s)?;
                        Ok(s)
                    })
                    .collect::<Result<Vec<_>, std::fmt::Error>>()?
                    .join(",");
                Ok(s)
            })
            .collect::<Result<Vec<_>, std::fmt::Error>>()?
            .join("),(");
        write!(f, "({})", strings)
    }
}

// pub fn multipolygon_to_wkt<
//     T: CoordNum + WktNum + fmt::Display,
//     G: MultiPolygonTrait<T = T>,
//     W: Write,
// >(
//     g: &G,
//     f: &mut W,
// ) -> Result<(), std::fmt::Error> {
//     let dim = g.dim();
//     // Write prefix
//     match dim {
//         geo_traits::Dimensions::Xy => f.write_str("MULTIPOLYGON"),
//         geo_traits::Dimensions::Xyz => f.write_str("MULTIPOLYGON Z"),
//         geo_traits::Dimensions::Xym => f.write_str("MULTIPOLYGON M"),
//         geo_traits::Dimensions::Xyzm => f.write_str("MULTIPOLYGON ZM"),
//         geo_traits::Dimensions::Unknown(_) => todo!(),
//     }?;

//     if g.num_polygons() == 0 {
//         f.write_str(" EMPTY")
//     } else {
//         let strings = g
//             .polygons()
//             .map(|polygon| {
//                 let exterior = polygon.exterior().unwrap();
//                 let exterior_string = exterior
//                     .coords()
//                     .map(|c| {
//                         let mut s = String::new();
//                         coord_to_wkt(&c, &mut s)?;
//                         Ok(s)
//                     })
//                     .collect::<Result<Vec<_>, std::fmt::Error>>()?
//                     .join(",");

//                 if polygon.num_interiors() == 0 {
//                     write!(f, "({})", exterior_string)
//                 } else {
//                     let interior_string = polygon
//                         .interiors()
//                         .map(|ring| {
//                             let s = ring
//                                 .coords()
//                                 .map(|c| {
//                                     let mut s = String::new();
//                                     coord_to_wkt(&c, &mut s)?;
//                                     Ok(s)
//                                 })
//                                 .collect::<Result<Vec<_>, std::fmt::Error>>()?
//                                 .join(",");
//                             Ok(s)
//                         })
//                         .collect::<Result<Vec<_>, std::fmt::Error>>()?
//                         .join("),(");
//                     write!(f, "({},({}))", exterior_string, interior_string)
//                 };
//                 Ok(s)
//             })
//             .collect::<Result<Vec<_>, std::fmt::Error>>()?
//             .join("),(");
//         write!(f, "({})", strings)
//     }
// }

fn add_coord<T: CoordNum + WktNum + fmt::Display, G: CoordTrait<T = T>, W: Write>(
    coord: &G,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    match coord.dim().size() {
        2 => write!(f, "{} {}", coord.x(), coord.y()),
        3 => {
            write!(f, "{} {} {}", coord.x(), coord.y(), coord.nth_unchecked(2))
        }
        4 => {
            write!(
                f,
                "{} {} {} {}",
                coord.x(),
                coord.y(),
                coord.nth_unchecked(2),
                coord.nth_unchecked(3)
            )
        }
        size => panic!("Unexpected dimension for coordinate: {}", size),
    }
}

fn add_coords<T: CoordNum + WktNum + fmt::Display, W: Write, C: CoordTrait<T = T>>(
    mut coords: impl ExactSizeIterator<Item = C>,
    f: &mut W,
) -> Result<(), std::fmt::Error> {
    f.write_char('(')?;

    if let Some(first_coord) = coords.next() {
        add_coord(&first_coord, f)?;

        for coord in coords {
            f.write_char(',')?;
            add_coord(&coord, f)?;
        }
    }

    f.write_char(')')?;

    Ok(())
}
