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
            use $crate::types::*;
            $crate::wkt_internal!($($wkt)+)
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! wkt_internal {
    (POINT $tt: tt) => {
        point!($tt)
    };
    (POINT Z $tt: tt) => {
        point_z!($tt)
    };
    (POINT M $tt: tt) => {
        point_m!($tt)
    };
    (POINT ZM $tt: tt) => {
        point_zm!($tt)
    };
    (LINESTRING $tt: tt) => {
        line_string!($tt)
    };
    (LINESTRING Z $tt: tt) => {
        line_string_z!($tt)
    };
    (LINESTRING M $tt: tt) => {
        line_string_m!($tt)
    };
    (LINESTRING ZM $tt: tt) => {
        line_string_zm!($tt)
    };
    (POLYGON $tt:tt) => {
        polygon!($tt)
    };
    (POLYGON Z $tt:tt) => {
        polygon_z!($tt)
    };
    (POLYGON M $tt:tt) => {
        polygon_m!($tt)
    };
    (POLYGON ZM $tt:tt) => {
        polygon_zm!($tt)
    };
    (MULTIPOINT $tt: tt) => {
        multi_point!($tt)
    };
    (MULTIPOINT Z $tt: tt) => {
        multi_point_z!($tt)
    };
    (MULTIPOINT M $tt: tt) => {
        multi_point_m!($tt)
    };
    (MULTIPOINT ZM $tt: tt) => {
        multi_point_zm!($tt)
    };
    (MULTILINESTRING $tt: tt) => {
        multi_line_string!($tt)
    };
    (MULTILINESTRING Z $tt: tt) => {
        multi_line_string_z!($tt)
    };
    (MULTILINESTRING M $tt: tt) => {
        multi_line_string_m!($tt)
    };
    (MULTILINESTRING ZM $tt: tt) => {
        multi_line_string_zm!($tt)
    };
    (MULTIPOLYGON $tt: tt) => {
        multi_polygon!($tt)
    };
    (MULTIPOLYGON Z $tt: tt) => {
        multi_polygon_z!($tt)
    };
    (MULTIPOLYGON M $tt: tt) => {
        multi_polygon_m!($tt)
    };
    (MULTIPOLYGON ZM $tt: tt) => {
        multi_polygon_zm!($tt)
    };
    (GEOMETRYCOLLECTION $tt: tt) => {
        geometry_collection!($tt)
    };
    (GEOMETRYCOLLECTION Z $tt: tt) => {
        geometry_collection_z!($tt)
    };
    (GEOMETRYCOLLECTION M $tt: tt) => {
        geometry_collection_m!($tt)
    };
    (GEOMETRYCOLLECTION ZM $tt: tt) => {
        geometry_collection_zm!($tt)
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

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_el_xy {
    (EMPTY) => {
        Point::empty(Dimension::XY)
    };
    ($x: literal $y: literal) => {
        Point::from_coord($crate::coord_xy!($x $y))
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_el_xyz {
    (EMPTY) => {
        Point::empty(Dimension::XYZ)
    };
    ($x: literal $y: literal $z:literal) => {
        Point::from_coord($crate::coord_xyz!($x $y $z))
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_el_xym {
    (EMPTY) => {
        Point::empty(Dimension::XYM)
    };
    ($x: literal $y: literal $m:literal) => {
        Point::from_coord($crate::coord_xym!($x $y $m))
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_el_xyzm {
    (EMPTY) => {
        Point::empty(Dimension::XYZM)
    };
    ($x: literal $y: literal $z:literal $m:literal) => {
        Point::from_coord($crate::coord_xyzm!($x $y $z $m))
    };
}

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
    (($($x: literal $y: literal $z:literal),*)) => {
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
    (($($x: literal $y: literal $m:literal),*)) => {
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

#[macro_export]
#[doc(hidden)]
macro_rules! polygon {
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    (EMPTY) => {
        Polygon::empty(Dimension::XY)
    };
    (( $($line_string_tt: tt),* )) => {
        Polygon::from_rings([
           $($crate::line_string![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! polygon_z {
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    (EMPTY) => {
        Polygon::empty(Dimension::XYZ)
    };
    (( $($line_string_tt: tt),* )) => {
        Polygon::from_rings([
           $($crate::line_string_z![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! polygon_m {
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    (EMPTY) => {
        Polygon::empty(Dimension::XYM)
    };
    (( $($line_string_tt: tt),* )) => {
        Polygon::from_rings([
           $($crate::line_string_m![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! polygon_zm {
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    (EMPTY) => {
        Polygon::empty(Dimension::XYZM)
    };
    (( $($line_string_tt: tt),* )) => {
        Polygon::from_rings([
           $($crate::line_string_zm![$line_string_tt]),*
        ])
    };
}

// Inspired by serde_json::json macro
#[macro_export]
#[doc(hidden)]
macro_rules! point_vec {
    (@points [$($el:expr),*]) => {
        // done
        vec![$($el),*]
    };
    (@points [$el:expr]) => {
        // done
        vec![$el]
    };

    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] EMPTY, $($rest:tt)*) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el_xy!(EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] $x:literal $y:literal, $($rest:tt)*) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el_xy!($x $y),] $($rest)*)
    };

    (@points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el_xy!(EMPTY)])
    };
    (@points [$($el:expr,)*] $x: literal $y:literal) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el_xy!($x $y)])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! point_vec_xyz {
    (@points [$($el:expr),*]) => {
        // done
        vec![$($el),*]
    };
    (@points [$el:expr]) => {
        // done
        vec![$el]
    };

    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] EMPTY, $($rest:tt)*) => {
        $crate::point_vec_xyz!(@points [$($el,)* $crate::point_el_xyz!(EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] $x:literal $y:literal $z:literal, $($rest:tt)*) => {
        $crate::point_vec_xyz!(@points [$($el,)* $crate::point_el_xyz!($x $y $z),] $($rest)*)
    };

    (@points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec_xyz!(@points [$($el,)* $crate::point_el_xyz!(EMPTY)])
    };
    (@points [$($el:expr,)*] $x: literal $y:literal $z:literal) => {
        $crate::point_vec_xyz!(@points [$($el,)* $crate::point_el_xyz!($x $y $z)])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! point_vec_xym {
    (@points [$($el:expr),*]) => {
        // done
        vec![$($el),*]
    };
    (@points [$el:expr]) => {
        // done
        vec![$el]
    };

    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] EMPTY, $($rest:tt)*) => {
        $crate::point_vec_xym!(@points [$($el,)* $crate::point_el_xym!(EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] $x:literal $y:literal $m:literal, $($rest:tt)*) => {
        $crate::point_vec_xym!(@points [$($el,)* $crate::point_el_xym!($x $y $m),] $($rest)*)
    };

    (@points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec_xym!(@points [$($el,)* $crate::point_el_xym!(EMPTY)])
    };
    (@points [$($el:expr,)*] $x: literal $y:literal $m:literal) => {
        $crate::point_vec_xym!(@points [$($el,)* $crate::point_el_xym!($x $y $m)])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! point_vec_xyzm {
    (@points [$($el:expr),*]) => {
        // done
        vec![$($el),*]
    };
    (@points [$el:expr]) => {
        // done
        vec![$el]
    };

    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] EMPTY, $($rest:tt)*) => {
        $crate::point_vec_xyzm!(@points [$($el,)* $crate::point_el_xyzm!(EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] $x:literal $y:literal $z:literal $m:literal, $($rest:tt)*) => {
        $crate::point_vec_xyzm!(@points [$($el,)* $crate::point_el_xyzm!($x $y $z $m),] $($rest)*)
    };

    (@points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec_xyzm!(@points [$($el,)* $crate::point_el_xyzm!(EMPTY)])
    };
    (@points [$($el:expr,)*] $x: literal $y:literal $z:literal $m:literal) => {
        $crate::point_vec_xyzm!(@points [$($el,)* $crate::point_el_xyzm!($x $y $z $m)])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point {
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    (EMPTY) => {
        MultiPoint::empty(Dimension::XY)
    };
    (($($tt: tt)*)) => {
        MultiPoint::from_points(
            point_vec!(@points [] $($tt)*)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point_z {
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    (EMPTY) => {
        MultiPoint::empty(Dimension::XYZ)
    };
    (($($tt: tt)*)) => {
        MultiPoint::from_points(
            point_vec_xyz!(@points [] $($tt)*)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point_m {
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    (EMPTY) => {
        MultiPoint::empty(Dimension::XYM)
    };
    (($($tt: tt)*)) => {
        MultiPoint::from_points(
            point_vec_xym!(@points [] $($tt)*)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point_zm {
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    (EMPTY) => {
        MultiPoint::empty(Dimension::XYZM)
    };
    (($($tt: tt)*)) => {
        MultiPoint::from_points(
            point_vec_xyzm!(@points [] $($tt)*)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string {
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    (EMPTY) => {
        MultiLineString::empty(Dimension::XY)
    };
    (( $($line_string_tt: tt),* )) => {
        MultiLineString::from_line_strings(vec![
           $($crate::line_string![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string_z {
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    (EMPTY) => {
        MultiLineString::empty(Dimension::XYZ)
    };
    (( $($line_string_tt: tt),* )) => {
        MultiLineString::from_line_strings(vec![
           $($crate::line_string_z![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string_m {
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    (EMPTY) => {
        MultiLineString::empty(Dimension::XYM)
    };
    (( $($line_string_tt: tt),* )) => {
        MultiLineString::from_line_strings(vec![
           $($crate::line_string_m![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string_zm {
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    (EMPTY) => {
        MultiLineString::empty(Dimension::XYZM)
    };
    (( $($line_string_tt: tt),* )) => {
        MultiLineString::from_line_strings(vec![
           $($crate::line_string_zm![$line_string_tt]),*
        ])
    };
}

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

    #[test]
    fn point() {
        let point = wkt! { POINT(1.0 2.0) };
        assert_eq!(point.coord.as_ref().unwrap().x, 1.0);
        assert_eq!(point.coord.as_ref().unwrap().y, 2.0);
        assert_eq!(point.dim, Dimension::XY);

        let point = wkt! { POINT(1.0   2.0) };
        assert_eq!(point.coord.as_ref().unwrap().x, 1.0);
        assert_eq!(point.coord.as_ref().unwrap().y, 2.0);
        assert_eq!(point.dim, Dimension::XY);

        let point = wkt! { POINT Z (1.0 2.0 3.0) };
        assert_eq!(point.coord.as_ref().unwrap().x, 1.0);
        assert_eq!(point.coord.as_ref().unwrap().y, 2.0);
        assert_eq!(point.coord.as_ref().unwrap().z, Some(3.0));
        assert_eq!(point.dim, Dimension::XYZ);

        let point = wkt! { POINT M (1.0 2.0 3.0) };
        assert_eq!(point.coord.as_ref().unwrap().x, 1.0);
        assert_eq!(point.coord.as_ref().unwrap().y, 2.0);
        assert_eq!(point.coord.as_ref().unwrap().m, Some(3.0));
        assert_eq!(point.dim, Dimension::XYM);

        let point = wkt! { POINT ZM (1.0 2.0 3.0 4.0) };
        assert_eq!(point.coord.as_ref().unwrap().x, 1.0);
        assert_eq!(point.coord.as_ref().unwrap().y, 2.0);
        assert_eq!(point.coord.as_ref().unwrap().z, Some(3.0));
        assert_eq!(point.coord.as_ref().unwrap().m, Some(4.0));
        assert_eq!(point.dim, Dimension::XYZM);
    }

    #[test]
    fn empty_point() {
        let point: Point<f64> = wkt! { POINT EMPTY };
        assert!(point.coord.is_none());
        assert_eq!(point.dim, Dimension::XY);

        let point: Point<f64> = wkt! { POINT Z EMPTY };
        assert!(point.coord.is_none());
        assert_eq!(point.dim, Dimension::XYZ);

        let point: Point<f64> = wkt! { POINT M EMPTY };
        assert!(point.coord.is_none());
        assert_eq!(point.dim, Dimension::XYM);

        let point: Point<f64> = wkt! { POINT ZM EMPTY };
        assert!(point.coord.is_none());
        assert_eq!(point.dim, Dimension::XYZM);
    }

    #[test]
    fn empty_line_string() {
        let line_string: LineString<f64> = wkt! { LINESTRING EMPTY };
        assert!(line_string.coords.is_empty());
        assert_eq!(line_string.dim, Dimension::XY);

        let line_string: LineString<f64> = wkt! { LINESTRING Z EMPTY };
        assert!(line_string.coords.is_empty());
        assert_eq!(line_string.dim, Dimension::XYZ);

        let line_string: LineString<f64> = wkt! { LINESTRING M EMPTY };
        assert!(line_string.coords.is_empty());
        assert_eq!(line_string.dim, Dimension::XYM);

        let line_string: LineString<f64> = wkt! { LINESTRING ZM EMPTY };
        assert!(line_string.coords.is_empty());
        assert_eq!(line_string.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { LINESTRING() }
    }

    #[test]
    fn line_string() {
        let line_string = wkt! { LINESTRING(1.0 2.0, 3.0 4.0) };
        assert_eq!(line_string.coords.len(), 2);
        assert_eq!(line_string.coords[0], coord_xy! { 1.0 2.0 });
        assert_eq!(line_string.dim, Dimension::XY);
    }

    #[test]
    fn empty_polygon() {
        let polygon: Polygon<f64> = wkt! { POLYGON EMPTY };
        assert!(polygon.rings.is_empty());
        assert_eq!(polygon.dim, Dimension::XY);

        let polygon: Polygon<f64> = wkt! { POLYGON Z EMPTY };
        assert!(polygon.rings.is_empty());
        assert_eq!(polygon.dim, Dimension::XYZ);

        let polygon: Polygon<f64> = wkt! { POLYGON M EMPTY };
        assert!(polygon.rings.is_empty());
        assert_eq!(polygon.dim, Dimension::XYM);

        let polygon: Polygon<f64> = wkt! { POLYGON ZM EMPTY };
        assert!(polygon.rings.is_empty());
        assert_eq!(polygon.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { POLYGON() }
    }

    #[test]
    fn polygon() {
        let polygon = wkt! { POLYGON((1.0 2.0)) };
        assert_eq!(polygon.rings.len(), 1);
        assert_eq!(polygon.rings[0].coords[0], coord_xy! { 1.0 2.0 });
        assert_eq!(polygon.dim, Dimension::XY);

        // let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0)) };
        // // Note: an extra coord is added to close the linestring
        // assert_eq!(polygon.exterior().0.len(), 3);
        // assert_eq!(polygon.exterior().0[0], coord! { x: 1.0, y: 2.0 });
        // assert_eq!(polygon.exterior().0[1], coord! { x: 3.0, y: 4.0 });
        // assert_eq!(polygon.exterior().0[2], coord! { x: 1.0, y: 2.0 });

        let polygon = wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
        assert_eq!(polygon.rings[0].coords.len(), 1);
        assert_eq!(polygon.rings[1].coords.len(), 1);
        assert_eq!(polygon.rings[0].coords[0], coord_xy! { 1.0 2.0 });
        assert_eq!(polygon.rings[1].coords[0], coord_xy! { 1.1 2.1 });

        // let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
        // assert_eq!(polygon.exterior().0.len(), 3);
        // assert_eq!(polygon.interiors().len(), 2);
        // assert_eq!(polygon.interiors()[1][1], coord! { x: 3.2, y: 4.2 });
    }

    #[test]
    fn empty_multi_point() {
        let multipoint: MultiPoint<f64> = wkt! { MULTIPOINT EMPTY };
        assert!(multipoint.points.is_empty());
        assert_eq!(multipoint.dim, Dimension::XY);

        let multipoint: MultiPoint<f64> = wkt! { MULTIPOINT Z EMPTY };
        assert!(multipoint.points.is_empty());
        assert_eq!(multipoint.dim, Dimension::XYZ);

        let multipoint: MultiPoint<f64> = wkt! { MULTIPOINT M EMPTY };
        assert!(multipoint.points.is_empty());
        assert_eq!(multipoint.dim, Dimension::XYM);

        let multipoint: MultiPoint<f64> = wkt! { MULTIPOINT ZM EMPTY };
        assert!(multipoint.points.is_empty());
        assert_eq!(multipoint.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { MULTIPOINT() }
    }

    #[test]
    fn multi_point() {
        let multi_point = wkt! { MULTIPOINT(1.0 2.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point! { (1.0 2.0) });
        assert_eq!(multi_point.dim, Dimension::XY);

        let multi_point = wkt! { MULTIPOINT(1.0 2.0,3.0 4.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point! { (1.0 2.0) });
        assert_eq!(multi_point.dim, Dimension::XY);

        let multi_point = wkt! { MULTIPOINT Z (1.0 2.0 3.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point_z! { (1.0 2.0 3.0) });
        assert_eq!(multi_point.dim, Dimension::XYZ);

        let multi_point = wkt! { MULTIPOINT Z (1.0 2.0 3.0, 4.0 5.0 6.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point_z! { (1.0 2.0 3.0) });
        assert_eq!(multi_point.points[1], point_z! { (4.0 5.0 6.0) });
        assert_eq!(multi_point.dim, Dimension::XYZ);

        let multi_point = wkt! { MULTIPOINT M (1.0 2.0 3.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point_m! { (1.0 2.0 3.0) });
        assert_eq!(multi_point.dim, Dimension::XYM);

        let multi_point = wkt! { MULTIPOINT M (1.0 2.0 3.0, 4.0 5.0 6.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point_m! { (1.0 2.0 3.0) });
        assert_eq!(multi_point.points[1], point_m! { (4.0 5.0 6.0) });
        assert_eq!(multi_point.dim, Dimension::XYM);

        let multi_point = wkt! { MULTIPOINT ZM (1.0 2.0 3.0 4.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point_zm! { (1.0 2.0 3.0 4.0) });
        assert_eq!(multi_point.dim, Dimension::XYZM);

        let multi_point = wkt! { MULTIPOINT ZM (1.0 2.0 3.0 4.0, 4.0 5.0 6.0 7.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point_zm! { (1.0 2.0 3.0 4.0) });
        assert_eq!(multi_point.points[1], point_zm! { (4.0 5.0 6.0 7.0) });
        assert_eq!(multi_point.dim, Dimension::XYZM);
    }

    #[test]
    fn empty_multi_line_string() {
        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING EMPTY };
        assert!(multi_line_string.line_strings.is_empty());
        assert_eq!(multi_line_string.dim, Dimension::XY);

        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING Z EMPTY };
        assert!(multi_line_string.line_strings.is_empty());
        assert_eq!(multi_line_string.dim, Dimension::XYZ);

        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING M EMPTY };
        assert!(multi_line_string.line_strings.is_empty());
        assert_eq!(multi_line_string.dim, Dimension::XYM);

        let multi_line_string: MultiLineString = wkt! { MULTILINESTRING ZM EMPTY };
        assert!(multi_line_string.line_strings.is_empty());
        assert_eq!(multi_line_string.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { MULTILINESTRING() }
    }

    #[test]
    fn multi_line_string() {
        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord_xy! { 3.0 4.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XY);

        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord_xy! { 7.0 8.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XY);

        let multi_line_string = wkt! { MULTILINESTRING Z ((1.0 2.0 3.0,3.0 4.0 5.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord_xyz! { 3.0 4.0 5.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZ);

        let multi_line_string =
            wkt! { MULTILINESTRING Z ((1.0 2.0 3.0,3.0 4.0 5.0),(5.0 6.0 7.0,7.0 8.0 9.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord_xyz! { 7.0 8.0 9.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZ);

        let multi_line_string = wkt! { MULTILINESTRING M ((1.0 2.0 3.0,3.0 4.0 5.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord_xym! { 3.0 4.0 5.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYM);

        let multi_line_string =
            wkt! { MULTILINESTRING M ((1.0 2.0 3.0,3.0 4.0 5.0),(5.0 6.0 7.0,7.0 8.0 9.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord_xym! { 7.0 8.0 9.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYM);

        let multi_line_string = wkt! { MULTILINESTRING ZM ((1.0 2.0 3.0 4.0,3.0 4.0 5.0 6.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord_xyzm! { 3.0 4.0 5.0 6.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZM);

        let multi_line_string = wkt! { MULTILINESTRING ZM ((1.0 2.0 3.0 4.0,3.0 4.0 5.0 6.0),(5.0 6.0 7.0 8.0,7.0 8.0 9.0 10.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord_xyzm! { 7.0 8.0 9.0 10.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZM);
    }

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
