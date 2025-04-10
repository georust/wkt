/// Creates a [`crate::geometry`] from a
/// [WKT](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry) literal.
///
/// This is evaluated at compile time, so you don't need to worry about runtime errors from invalid
/// WKT syntax.
///
/// Note that `POINT EMPTY` is not accepted because it is not representable as a `geo_types::Point`.
///
/// ```
/// use geo_types::wkt;
/// let point = wkt! { POINT(1.0 2.0) };
/// assert_eq!(point.x(), 1.0);
/// assert_eq!(point.y(), 2.0);
///
/// let geometry_collection = wkt! {
///     GEOMETRYCOLLECTION(
///         POINT(1.0 2.0),
///         LINESTRING EMPTY,
///         POLYGON((0.0 0.0,1.0 0.0,1.0 1.0,0.0 0.0))
///     )
/// };
/// assert_eq!(geometry_collection.len(), 3);
/// ```
#[macro_export]
macro_rules! wkt {
    // Hide distracting implementation details from the generated rustdoc.
    ($($wkt:tt)+) => {
        {
            use $crate::{types::*, Wkt};
            $crate::wkt_internal!($($wkt)+)
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! wkt_internal {
    (POINT $tt: tt) => {
        Wkt::<f64>::Point(point!($tt))
    };
    (POINT Z $tt: tt) => {
        Wkt::<f64>::Point(point_z!($tt))
    };
    (POINT M $tt: tt) => {
        Wkt::<f64>::Point(point_m!($tt))
    };
    (POINT ZM $tt: tt) => {
        Wkt::<f64>::Point(point_zm!($tt))
    };
    (LINESTRING $tt: tt) => {
        Wkt::<f64>::LineString(line_string!($tt))
    };
    (LINESTRING Z $tt: tt) => {
        Wkt::<f64>::LineString(line_string_z!($tt))
    };
    (LINESTRING M $tt: tt) => {
        Wkt::<f64>::LineString(line_string_m!($tt))
    };
    (LINESTRING ZM $tt: tt) => {
        Wkt::<f64>::LineString(line_string_zm!($tt))
    };
    (POLYGON $tt:tt) => {
        Wkt::<f64>::Polygon(polygon!($tt))
    };
    (MULTIPOINT $tt: tt) => {
        Wkt::<f64>::MultiPoint(multi_point!($tt))
    };
    (MULTILINESTRING $tt: tt) => {
        Wkt::<f64>::MultiLineString(multi_line_string!($tt))
    };
    (MULTIPOLYGON $tt: tt) => {
        Wkt::<f64>::MultiPolygon(multi_polygon!($tt))
    };
    (GEOMETRYCOLLECTION $tt: tt) => {
        Wkt::<f64>::GeometryCollection(geometry_collection!($tt))
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord_xy {
    ($x: literal $y: literal) => {
        Coord {
            x: $x,
            y: $y,
            z: None,
            m: None,
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord_xyz {
    ($x: literal $y: literal $z: literal) => {
        Coord {
            x: $x,
            y: $y,
            z: Some($z),
            m: None,
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord_xym {
    ($x: literal $y: literal $m: literal) => {
        Coord {
            x: $x,
            y: $y,
            z: None,
            m: Some($m),
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord_xyzm {
    ($x: literal $y: literal $z: literal $m: literal) => {
        Coord {
            x: $x,
            y: $y,
            z: Some($z),
            m: Some($m),
        }
    };
}

// #[macro_export(local_inner_macros)]
// #[doc(hidden)]
// macro_rules! point_el {
//     (EMPTY) => {
//         Point::empty(Dimension::XY)
//     };
//     (Z EMPTY) => {
//         Point::empty(Dimension::XYZ)
//     };
//     (M EMPTY) => {
//         Point::empty(Dimension::XYM)
//     };
//     (ZM EMPTY) => {
//         Point::empty(Dimension::XYZM)
//     };
//     ($x: literal $y: literal) => {
//         Point(Some(coord!($x $y)))
//     };
// }

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point {
    (($x: literal $y: literal)) => {
        Point::from_coord($crate::coord_xy!($x $y))
    };
    (EMPTY) => {
        Point::empty(Dimension::XY)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_z {
    (($x: literal $y: literal $z: literal)) => {
        Point::from_coord($crate::coord_xyz!($x $y $z))
    };
    (EMPTY) => {
        Point::empty(Dimension::XYZ)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_m {
    (($x: literal $y: literal $m: literal)) => {
        Point::from_coord($crate::coord_xym!($x $y $m))
    };
    (EMPTY) => {
        Point::empty(Dimension::XYM)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_zm {
    (($x: literal $y: literal $z: literal $m: literal)) => {
        Point::from_coord($crate::coord_xyzm!($x $y $z $m))
    };
    (EMPTY) => {
        Point::empty(Dimension::XYZM)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    (EMPTY) => {
        LineString::empty(Dimension::XY)
    };
    (($($x: literal $y: literal),*)) => {
        LineString::from_coords(
            [$($crate::coord_xy!($x $y)),*]
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string_z {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    (EMPTY) => {
        LineString::empty(Dimension::XYZ)
    };
    (($($x: literal $y: literal),*)) => {
        LineString::from_coords(
            [$($crate::coord_xyz!($x $y $z)),*]
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string_m {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    (EMPTY) => {
        LineString::empty(Dimension::XYM)
    };
    (($($x: literal $y: literal),*)) => {
        LineString::from_coords(
            [$($crate::coord_xym!($x $y $m)),*]
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string_zm {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    (EMPTY) => {
        LineString::empty(Dimension::XYZM)
    };
    (($($x: literal $y: literal $z: literal $m: literal),*)) => {
        LineString::from_coords(
            [$($crate::coord_xyzm!($x $y $z $m)),*]
        )
    };
}

// #[macro_export]
// #[doc(hidden)]
// macro_rules! polygon {
//     (()) => {
//         compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
//     };
//     (EMPTY) => {
//         Polygon::empty(Dimension::XY)
//     };
//     (Z EMPTY) => {
//         Polygon::empty(Dimension::XYZ)
//     };
//     (M EMPTY) => {
//         Polygon::empty(Dimension::XYM)
//     };
//     (ZM EMPTY) => {
//         Polygon::empty(Dimension::XYZM)
//     };
//     (( $($line_string_tt: tt),* )) => {
//         Polygon::from_rings([
//            $($crate::line_string![$line_string_tt]),*
//         ])
//     };
//     (Z ( $($line_string_tt: tt),* )) => {
//         Polygon::from_rings([
//            $($crate::line_string![$line_string_tt]),*
//         ])
//     };
//     (M ( $($line_string_tt: tt),* )) => {
//         Polygon::from_rings([
//            $($crate::line_string![$line_string_tt]),*
//         ])
//     };
//     (ZM ( $($line_string_tt: tt),* )) => {
//         Polygon::from_rings([
//            $($crate::line_string![$line_string_tt]),*
//         ])
//     };
// }

// #[macro_export]
// #[doc(hidden)]
// macro_rules! wkt_internal {
//     (POINT EMPTY) => {
//         $crate::types::Point::empty($crate::types::Dimension::XY)
//     };
//     (POINT Z EMPTY) => {
//         $crate::types::Point::empty($crate::types::Dimension::XYZ)
//     };
//     (POINT M EMPTY) => {
//         $crate::types::Point::empty($crate::types::Dimension::XYM)
//     };
//     (POINT ZM EMPTY) => {
//         $crate::types::Point::empty($crate::types::Dimension::XYZM)
//     };
//     (POINT($x: literal $y: literal)) => {

//         $crate::point!(x: $x, y: $y)
//     };
//     (POINT $($tail: tt)*) => {
//         compile_error!("Invalid POINT wkt");
//     };
//     (LINESTRING EMPTY) => {
//         $crate::types::LineString::empty($crate::types::Dimension::XY)
//     };
//     (LINESTRING Z EMPTY) => {
//         $crate::types::LineString::empty($crate::types::Dimension::XYZ)
//     };
//     (LINESTRING M EMPTY) => {
//         $crate::types::LineString::empty($crate::types::Dimension::XYM)
//     };
//     (LINESTRING ZM EMPTY) => {
//         $crate::types::LineString::empty($crate::types::Dimension::XYZM)
//     };
//     (LINESTRING ($($x: literal $y: literal),+)) => {
//         $crate::types::LineString::from_coords([
//             $($crate::coord!(x: $x, y: $y)),*
//         ])
//     };
//     (LINESTRING ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (LINESTRING $($tail: tt)*) => {
//         compile_error!("Invalid LINESTRING wkt");
//     };
//     (POLYGON EMPTY) => {
//         $crate::types::Polygon::empty($crate::types::Dimension::XY)
//     };
//     (POLYGON Z EMPTY) => {
//         $crate::types::Polygon::empty($crate::types::Dimension::XYZ)
//     };
//     (POLYGON M EMPTY) => {
//         $crate::types::Polygon::empty($crate::types::Dimension::XYM)
//     };
//     (POLYGON ZM EMPTY) => {
//         $crate::types::Polygon::empty($crate::types::Dimension::XYZM)
//     };
//     // TODO: trying to collapse the multiple polygon definitions into a single one,
//     // because unlike geo we don't have to split off the first exterior ring.
//     // (MULTILINESTRING ( $($line_string_tt: tt),* )) => {
//     (POLYGON ( $($rings: tt),* )) => {
//         $crate::types::Polygon::from_rings([
//            $($crate::wkt!(LINESTRING $line_string_tt)),*
//         ])
//     };
//     // (POLYGON ( $exterior_tt: tt )) => {
//     //     $crate::Polygon::new($crate::wkt!(LINESTRING $exterior_tt), $crate::_alloc::vec![])
//     // };
//     // (POLYGON( $exterior_tt: tt, $($interiors_tt: tt),+ )) => {
//     //     $crate::Polygon::from_rings(
//     //         $crate::wkt!(LINESTRING $exterior_tt),
//     //         $crate::_alloc::vec![
//     //            $($crate::wkt!(LINESTRING $interiors_tt)),*
//     //         ]
//     //     )
//     // };
//     (POLYGON ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (POLYGON $($tail: tt)*) => {
//         compile_error!("Invalid POLYGON wkt");
//     };
//     (MULTIPOINT EMPTY) => {
//         $crate::types::MultiPoint::empty($crate::types::Dimension::XY)
//     };
//     (MULTIPOINT Z EMPTY) => {
//         $crate::types::MultiPoint::empty($crate::types::Dimension::XYZ)
//     };
//     (MULTIPOINT M EMPTY) => {
//         $crate::types::MultiPoint::empty($crate::types::Dimension::XYM)
//     };
//     (MULTIPOINT ZM EMPTY) => {
//         $crate::types::MultiPoint::empty($crate::types::Dimension::XYZM)
//     };
//     (MULTIPOINT ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (MULTIPOINT ($($x: literal $y: literal),* )) => {
//         $crate::types::MultiPoint::from_points(
//             [$($crate::point!(x: $x, y: $y)),*]
//         )
//     };
//     (MULTIPOINT $($tail: tt)*) => {
//         compile_error!("Invalid MULTIPOINT wkt");
//     };
//     (MULTILINESTRING EMPTY) => {
//         $crate::types::MultiLineString::empty($crate::types::Dimension::XY)
//     };
//     (MULTILINESTRING Z EMPTY) => {
//         $crate::types::MultiLineString::empty($crate::types::Dimension::XYZ)
//     };
//     (MULTILINESTRING M EMPTY) => {
//         $crate::types::MultiLineString::empty($crate::types::Dimension::XYM)
//     };
//     (MULTILINESTRING ZM EMPTY) => {
//         $crate::types::MultiLineString::empty($crate::types::Dimension::XYZM)
//     };
//     (MULTILINESTRING ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (MULTILINESTRING ( $($line_string_tt: tt),* )) => {
//         $crate::types::MultiLineString::from_line_strings([
//            $($crate::wkt!(LINESTRING $line_string_tt)),*
//         ])
//     };
//     (MULTILINESTRING $($tail: tt)*) => {
//         compile_error!("Invalid MULTILINESTRING wkt");
//     };
//     (MULTIPOLYGON EMPTY) => {
//         $crate::types::MultiPolygon::empty($crate::types::Dimension::XY)
//     };
//     (MULTIPOLYGON Z EMPTY) => {
//         $crate::types::MultiPolygon::empty($crate::types::Dimension::XYZ)
//     };
//     (MULTIPOLYGON M EMPTY) => {
//         $crate::types::MultiPolygon::empty($crate::types::Dimension::XYM)
//     };
//     (MULTIPOLYGON ZM EMPTY) => {
//         $crate::types::MultiPolygon::empty($crate::types::Dimension::XYZM)
//     };
//     (MULTIPOLYGON ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (MULTIPOLYGON ( $($polygon_tt: tt),* )) => {
//         $crate::types::MultiPolygon::from_polygons([
//            $($crate::wkt!(POLYGON $polygon_tt)),*
//         ])
//     };
//     (MULTIPOLYGON $($tail: tt)*) => {
//         compile_error!("Invalid MULTIPOLYGON wkt");
//     };
//     (GEOMETRYCOLLECTION EMPTY) => {
//         $crate::types::GeometryCollection::empty($crate::types::Dimension::XY)
//     };
//     (GEOMETRYCOLLECTION Z EMPTY) => {
//         $crate::types::GeometryCollection::empty($crate::types::Dimension::XYZ)
//     };
//     (GEOMETRYCOLLECTION M EMPTY) => {
//         $crate::types::GeometryCollection::empty($crate::types::Dimension::XYM)
//     };
//     (GEOMETRYCOLLECTION ZM EMPTY) => {
//         $crate::types::GeometryCollection::empty($crate::types::Dimension::XYZM)
//     };
//     (GEOMETRYCOLLECTION ()) => {
//         compile_error!("use `EMPTY` instead of () for an empty collection")
//     };
//     (GEOMETRYCOLLECTION ( $($el_type:tt $el_tt: tt),* )) => {
//         $crate::types::GeometryCollection::from_geometries([
//            $($crate::Geometry::from($crate::wkt!($el_type $el_tt))),*
//         ])
//     };
//     (GEOMETRYCOLLECTION $($tail: tt)*) => {
//         compile_error!("Invalid GEOMETRYCOLLECTION wkt");
//     };
//     ($name: ident ($($tail: tt)*)) => {
//         compile_error!("Unknown type. Must be one of POINT, LINESTRING, POLYGON, MULTIPOINT, MULTILINESTRING, MULTIPOLYGON, or GEOMETRYCOLLECTION");
//     };
// }

#[cfg(test)]
mod test {
    use crate::types::*;
    use crate::Wkt;

    #[test]
    fn point() {
        let point = wkt! { POINT(1.0 2.0) };
        match point {
            Wkt::Point(p) => {
                assert_eq!(p.coord.as_ref().unwrap().x, 1.0);
                assert_eq!(p.coord.as_ref().unwrap().y, 2.0);
                assert_eq!(p.dim, Dimension::XY);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT(1.0   2.0) };
        match point {
            Wkt::Point(p) => {
                assert_eq!(p.coord.as_ref().unwrap().x, 1.0);
                assert_eq!(p.coord.as_ref().unwrap().y, 2.0);
                assert_eq!(p.dim, Dimension::XY);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT Z (1.0 2.0 3.0) };
        match point {
            Wkt::Point(p) => {
                assert_eq!(p.coord.as_ref().unwrap().x, 1.0);
                assert_eq!(p.coord.as_ref().unwrap().y, 2.0);
                assert_eq!(p.coord.as_ref().unwrap().z, Some(3.0));
                assert_eq!(p.dim, Dimension::XYZ);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT M (1.0 2.0 3.0) };
        match point {
            Wkt::Point(p) => {
                assert_eq!(p.coord.as_ref().unwrap().x, 1.0);
                assert_eq!(p.coord.as_ref().unwrap().y, 2.0);
                assert_eq!(p.coord.as_ref().unwrap().m, Some(3.0));
                assert_eq!(p.dim, Dimension::XYM);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT ZM (1.0 2.0 3.0 4.0) };
        match point {
            Wkt::Point(p) => {
                assert_eq!(p.coord.as_ref().unwrap().x, 1.0);
                assert_eq!(p.coord.as_ref().unwrap().y, 2.0);
                assert_eq!(p.coord.as_ref().unwrap().z, Some(3.0));
                assert_eq!(p.coord.as_ref().unwrap().m, Some(4.0));
                assert_eq!(p.dim, Dimension::XYZM);
            }
            _ => panic!("Expected a Point"),
        }
    }

    #[test]
    fn empty_point() {
        let point = wkt! { POINT EMPTY };
        match point {
            Wkt::Point(p) => {
                assert!(p.coord.is_none());
                assert_eq!(p.dim, Dimension::XY);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT Z EMPTY };
        match point {
            Wkt::Point(p) => {
                assert!(p.coord.is_none());
                assert_eq!(p.dim, Dimension::XYZ);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT M EMPTY };
        match point {
            Wkt::Point(p) => {
                assert!(p.coord.is_none());
                assert_eq!(p.dim, Dimension::XYM);
            }
            _ => panic!("Expected a Point"),
        }

        let point = wkt! { POINT ZM EMPTY };
        match point {
            Wkt::Point(p) => {
                assert!(p.coord.is_none());
                assert_eq!(p.dim, Dimension::XYZM);
            }
            _ => panic!("Expected a Point"),
        }
    }

    #[test]
    fn empty_line_string() {
        let line_string = wkt! { LINESTRING EMPTY };
        match line_string {
            Wkt::LineString(l) => {
                assert!(l.coords.is_empty());
                assert_eq!(l.dim, Dimension::XY);
            }
            _ => panic!("Expected a LineString"),
        }

        let line_string = wkt! { LINESTRING Z EMPTY };
        match line_string {
            Wkt::LineString(l) => {
                assert!(l.coords.is_empty());
                assert_eq!(l.dim, Dimension::XYZ);
            }
            _ => panic!("Expected a LineString"),
        }

        let line_string = wkt! { LINESTRING M EMPTY };
        match line_string {
            Wkt::LineString(l) => {
                assert!(l.coords.is_empty());
                assert_eq!(l.dim, Dimension::XYM);
            }
            _ => panic!("Expected a LineString"),
        }

        let line_string = wkt! { LINESTRING ZM EMPTY };
        match line_string {
            Wkt::LineString(l) => {
                assert!(l.coords.is_empty());
                assert_eq!(l.dim, Dimension::XYZM);
            }
            _ => panic!("Expected a LineString"),
        }

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { LINESTRING() }
    }

    #[test]
    fn line_string() {
        let line_string = wkt! { LINESTRING(1.0 2.0,3.0 4.0) };
        match line_string {
            Wkt::LineString(l) => {
                assert_eq!(l.coords.len(), 2);
                assert_eq!(l.coords[0], coord_xy! { 1.0 2.0 });
                assert_eq!(l.dim, Dimension::XY);
            }
            _ => panic!("Expected a Point"),
        }
    }

    //     #[test]
    //     fn empty_polygon() {
    //         let polygon: Polygon = wkt! { POLYGON EMPTY };
    //         assert_eq!(polygon.exterior().0.len(), 0);
    //         assert_eq!(polygon.interiors().len(), 0);

    //         // This (rightfully) fails to compile because its invalid wkt
    //         // wkt! { POLYGON() }
    //     }

    //     #[test]
    //     fn polygon() {
    //         let polygon = wkt! { POLYGON((1.0 2.0)) };
    //         assert_eq!(polygon.exterior().0.len(), 1);
    //         assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });

    //         let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0)) };
    //         // Note: an extra coord is added to close the linestring
    //         assert_eq!(polygon.exterior().0.len(), 3);
    //         assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
    //         assert_eq!(polygon.exterior().0[1], coord! { x: 3.0, y: 4.0 });
    //         assert_eq!(polygon.exterior().0[2], coord! { x: 1.0, y: 2.0 });

    //         let polygon = wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
    //         assert_eq!(polygon.exterior().0.len(), 1);
    //         assert_eq!(polygon.interiors().len(), 1);

    //         assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
    //         assert_eq!(polygon.interiors()[0].0[0], coord! { x: 1.1, y: 2.1 });

    //         let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
    //         assert_eq!(polygon.exterior().0.len(), 3);
    //         assert_eq!(polygon.interiors().len(), 2);
    //         assert_eq!(polygon.interiors()[1][1], coord! { x: 3.2, y: 4.2 });
    //     }

    //     #[test]
    //     fn empty_multi_point() {
    //         let multipoint: MultiPoint = wkt! { MULTIPOINT EMPTY };
    //         assert!(multipoint.0.is_empty());
    //         // This (rightfully) fails to compile because its invalid wkt
    //         // wkt! { MULTIPOINT() }
    //     }

    //     #[test]
    //     fn multi_point() {
    //         let multi_point = wkt! { MULTIPOINT(1.0 2.0) };
    //         assert_eq!(multi_point.0, vec![point! { x: 1.0, y: 2.0}]);

    //         let multi_point = wkt! { MULTIPOINT(1.0 2.0,3.0 4.0) };
    //         assert_eq!(
    //             multi_point.0,
    //             vec![point! { x: 1.0, y: 2.0}, point! { x: 3.0, y: 4.0}]
    //         );
    //     }

    //     #[test]
    //     fn empty_multi_line_string() {
    //         let multi_line_string: MultiLineString = wkt! { MULTILINESTRING EMPTY };
    //         assert_eq!(multi_line_string.0, vec![]);
    //         // This (rightfully) fails to compile because its invalid wkt
    //         // wkt! { MULTILINESTRING() }
    //     }
    //     #[test]
    //     fn multi_line_string() {
    //         let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0)) };
    //         assert_eq!(multi_line_string.0.len(), 1);
    //         assert_eq!(multi_line_string.0[0].0[1], coord! { x: 3.0, y: 4.0 });
    //         let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) };
    //         assert_eq!(multi_line_string.0.len(), 2);
    //         assert_eq!(multi_line_string.0[1].0[1], coord! { x: 7.0, y: 8.0 });

    //         let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),EMPTY) };
    //         assert_eq!(multi_line_string.0.len(), 2);
    //         assert_eq!(multi_line_string.0[1].0.len(), 0);
    //     }

    //     #[test]
    //     fn empty_multi_polygon() {
    //         let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON EMPTY };
    //         assert!(multi_polygon.0.is_empty());

    //         // This (rightfully) fails to compile because its invalid wkt
    //         // wkt! { MULTIPOLYGON() }
    //     }

    //     #[test]
    //     fn multi_line_polygon() {
    //         let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0))) };
    //         assert_eq!(multi_polygon.0.len(), 1);
    //         assert_eq!(multi_polygon.0[0].exterior().0[0], coord! { x: 1.0, y: 2.0});

    //         let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)),((1.0 2.0))) };
    //         assert_eq!(multi_polygon.0.len(), 2);
    //         assert_eq!(
    //             multi_polygon.0[0].interiors()[1].0[0],
    //             coord! { x: 1.2, y: 2.2}
    //         );

    //         let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)), EMPTY) };
    //         assert_eq!(multi_polygon.0.len(), 2);
    //         assert_eq!(
    //             multi_polygon.0[0].interiors()[1].0[0],
    //             coord! { x: 1.2, y: 2.2}
    //         );
    //         assert!(multi_polygon.0[1].exterior().0.is_empty());
    //     }

    //     #[test]
    //     fn empty_geometry_collection() {
    //         let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION EMPTY };
    //         assert!(geometry_collection.is_empty());

    //         // This (rightfully) fails to compile because its invalid wkt
    //         // wkt! { MULTIPOLYGON() }
    //     }

    //     #[test]
    //     fn geometry_collection() {
    //         let geometry_collection = wkt! {
    //             GEOMETRYCOLLECTION (
    //                 POINT (40.0 10.0),
    //                 LINESTRING (10.0 10.0, 20.0 20.0, 10.0 40.0),
    //                 POLYGON ((40.0 40.0, 20.0 45.0, 45.0 30.0, 40.0 40.0))
    //             )
    //         };
    //         assert_eq!(geometry_collection.len(), 3);

    //         let line_string = match &geometry_collection[1] {
    //             Geometry::LineString(line_string) => line_string,
    //             _ => panic!(
    //                 "unexpected geometry: {geometry:?}",
    //                 geometry = geometry_collection[1]
    //             ),
    //         };
    //         assert_eq!(line_string.0[1], coord! {x: 20.0, y: 20.0 });
    //     }

    //     #[test]
    //     fn other_numeric_types() {
    //         let point: Point<i32> = wkt!(POINT(1 2));
    //         assert_eq!(point.x(), 1i32);
    //         assert_eq!(point.y(), 2i32);

    //         let point: Point<u64> = wkt!(POINT(1 2));
    //         assert_eq!(point.x(), 1u64);
    //         assert_eq!(point.y(), 2u64);

    //         let point: Point<f32> = wkt!(POINT(1.0 2.0));
    //         assert_eq!(point.x(), 1.0f32);
    //         assert_eq!(point.y(), 2.0f32);
    //     }
}
