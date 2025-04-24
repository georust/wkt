/// Creates a geometry from a [WKT] literal.
///
/// This is evaluated at compile time, so you don't need to worry about runtime errors from invalid
/// WKT syntax.
///
/// Notes:
///
/// - Creates a concrete type. So `wkt! { POINT(1.0 2.0) }` will create a
///   [`Point`][crate::types::Point], not a [`Wkt`][crate::Wkt].
/// - Empty geometries, including `POINT EMPTY` **are** supported.
/// - All dimensions, including `Z`, `M`, and `ZM` are supported.
/// - Extended geometry types like `Curve`, `PolyhedralSurface`, or `CircularString` are **not**
///   supported.
///
/// ```
/// use wkt::wkt;
/// use wkt::types::Dimension;
/// use geo_traits::{PointTrait, CoordTrait, LineStringTrait, GeometryCollectionTrait};
///
/// let point = wkt! { POINT(1.0 2.0) };
/// assert_eq!(point.coord().unwrap().x(), 1.0);
/// assert_eq!(point.coord().unwrap().y(), 2.0);
///
/// let line_string = wkt! { LINESTRING ZM (1.0 2.0 3.0 4.0, 3.0 4.0 5.0 6.0) };
/// assert_eq!(line_string.num_coords(), 2);
/// assert_eq!(line_string.coord(0).unwrap().nth_or_panic(0), 1.0);
/// assert_eq!(line_string.coord(0).unwrap().nth_or_panic(1), 2.0);
/// assert_eq!(line_string.coord(0).unwrap().nth_or_panic(2), 3.0);
/// assert_eq!(line_string.coord(0).unwrap().nth_or_panic(3), 4.0);
/// assert_eq!(line_string.dimension(), Dimension::XYZM);
///
/// let geometry_collection = wkt! {
///     GEOMETRYCOLLECTION(
///         POINT(1.0 2.0),
///         LINESTRING EMPTY,
///         POLYGON((0.0 0.0,1.0 0.0,1.0 1.0,0.0 0.0))
///     )
/// };
/// assert_eq!(geometry_collection.geometries().len(), 3);
/// ```
///
/// [WKT]: https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry
#[macro_export]
macro_rules! wkt {
    (POINT $($tt: tt)+) => {
        $crate::point!($($tt)+)
    };
    (LINESTRING $($tt: tt)+) => {
        $crate::line_string!($($tt)+)
    };
    (POLYGON $($tt:tt)+) => {
       $crate::polygon!($($tt)+)
    };
    (MULTIPOINT $($tt: tt)+) => {
        $crate::multi_point!($($tt)+)
    };
    (MULTILINESTRING $($tt: tt)+) => {
        $crate::multi_line_string!($($tt)+)
    };
    (MULTIPOLYGON $($tt: tt)+) => {
        $crate::multi_polygon!($($tt)+)
    };
    (GEOMETRYCOLLECTION $($tt: tt)+) => {
        $crate::geometry_collection!($($tt)+)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! dim {
    (Z) => {
        $crate::types::Dimension::XYZ
    };
    (M) => {
        $crate::types::Dimension::XYM
    };
    (ZM) => {
        $crate::types::Dimension::XYZM
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord {
    ($x: literal $y: literal) => {
        $crate::types::Coord {
            x: $x,
            y: $y,
            z: None,
            m: None,
        }
    };
    (Z $x: literal $y: literal $z: literal) => {
        $crate::types::Coord {
            x: $x,
            y: $y,
            z: Some($z),
            m: None,
        }
    };
    (M $x: literal $y: literal $m: literal) => {
        $crate::types::Coord {
            x: $x,
            y: $y,
            z: None,
            m: Some($m),
        }
    };
    (ZM $x: literal $y: literal $z: literal $m: literal) => {
        $crate::types::Coord {
            x: $x,
            y: $y,
            z: Some($z),
            m: Some($m),
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point {
    ($($dim: ident)? ($($scalar: literal)+)) => {
        $crate::types::Point::from_coord($crate::coord!($($dim)? $($scalar)+))
    };
    (EMPTY) => {
        $crate::types::Point::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::Point::empty($crate::dim!($dim))
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `LINESTRING ", stringify!($dim), " EMPTY` for a Polygon with no coordinates"))
    };
    (EMPTY) => {
       $crate::types::LineString::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::LineString::empty($crate::dim!($dim))
    };
    (($($($scalar: literal)+),*)) => {
        $crate::types::LineString::from_coords(
            [$($crate::coord!($($scalar)+)),*]
        ).unwrap()
    };
    ($dim: ident ($($($scalar: literal)+),*)) => {
        $crate::types::LineString::from_coords(
            [$($crate::coord!($dim $($scalar)+)),*]
        ).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! polygon {
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `POLYGON ", stringify!($dim), " EMPTY` for a Polygon with no coordinates"))
    };
    (EMPTY) => {
        $crate::types::Polygon::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::Polygon::empty($crate::dim!($dim))
    };
    (( $($line_string_tt: tt),* )) => {
        $crate::types::Polygon::from_rings([
           $($crate::line_string![$line_string_tt]),*
        ]).unwrap()
    };
    ($dim: ident ( $($line_string_tt: tt),* )) => {
        $crate::types::Polygon::from_rings([
           $($crate::line_string![$dim $line_string_tt]),*
        ]).unwrap()
    };
}

// Inspired by serde_json::json macro
#[macro_export]
#[doc(hidden)]
macro_rules! point_vec {
    ($($dim: ident)? @points [$($el:expr),*]) => {
        // done
        vec![$($el),*]
    };
    ($($dim: ident)? @points [$el:expr]) => {
        // done
        vec![$el]
    };

    // Next element is an expression followed by comma.
    ($($dim: ident)? @points [$($el:expr,)*] EMPTY, $($rest:tt)*) => {
        $crate::point_vec!($($dim)? @points [$($el,)* $crate::wkt!(POINT $($dim)? EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    ($($dim: ident)? @points [$($el:expr,)*] $($scalar:literal)+, $($rest:tt)*) => {
        $crate::point_vec!($($dim)? @points [$($el,)* $crate::wkt!(POINT $($dim)? ($($scalar)+)),] $($rest)*)
    };

    ($($dim: ident)? @points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec!($($dim)? @points [$($el,)* $crate::wkt!(POINT $($dim)? EMPTY)])
    };
    ($($dim: ident)? @points [$($el:expr,)*] $($scalar:literal)+) => {
        $crate::point_vec!($($dim)? @points [$($el,)* $crate::wkt!(POINT $($dim)? ($($scalar)+))])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point {
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `MULTIPOINT ", stringify!($dim), " EMPTY` for a MultiPoint with no coordinates"))
    };
    (EMPTY) => {
       $crate::types::MultiPoint::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::MultiPoint::empty($crate::dim!($dim))
    };
    ($($dim: ident)? ($($tt: tt)*)) => {
        $crate::types::MultiPoint::from_points(
            $crate::point_vec!($($dim)? @points [] $($tt)*)
        ).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string {
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `MULTILINESTRING ", stringify!($dim), " EMPTY` for a MultiLineString with no coordinates"))
    };
    (EMPTY) => {
        $crate::types::MultiLineString::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::MultiLineString::empty($crate::dim!($dim))
    };
    (( $($line_string_tt: tt),* )) => {
        $crate::types::MultiLineString::from_line_strings(vec![
           $($crate::line_string![$line_string_tt]),*
        ]).unwrap()
    };
    ($dim: ident ( $($line_string_tt: tt),* )) => {
        $crate::types::MultiLineString::from_line_strings(vec![
           $($crate::line_string![$dim $line_string_tt]),*
        ]).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_polygon {
    (()) => {
        compile_error!("use `MULTIPOLYGON EMPTY` for a MultiPolygon with no coordinates")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `MULTIPOLYGON ", stringify!($dim), " EMPTY` for a MultiPolygon with no coordinates"))
    };
    (EMPTY) => {
        $crate::types::MultiPolygon::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::MultiPolygon::empty($crate::dim!($dim))
    };
    (( $($polygon_tt: tt),* )) => {
        $crate::types::MultiPolygon::from_polygons(vec![
           $($crate::polygon![$polygon_tt]),*
        ]).unwrap()
    };
    ($dim: ident ( $($polygon_tt: tt),* )) => {
        $crate::types::MultiPolygon::from_polygons(vec![
           $($crate::polygon![$dim $polygon_tt]),*
        ]).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! geometry_collection {
    (EMPTY) => {
        $crate::types::GeometryCollection::empty($crate::types::Dimension::XY)
    };
    ($dim: ident EMPTY) => {
        $crate::types::GeometryCollection::empty($crate::dim!($dim))
    };
    (()) => {
        compile_error!("use `GEOMETRYCOLLECTION EMPTY` for an empty collection")
    };
    ($dim: ident ()) => {
        compile_error!(concat!("use `GEOMETRYCOLLECTION ", stringify!($dim), " EMPTY` for an empty collection"))
    };
    (( $($el_type:tt $el_tt: tt),* )) => {
        $crate::types::GeometryCollection::from_geometries(vec![
           $($crate::wkt!($el_type $el_tt).into()),*
        ]).unwrap()
    };
    ($dim: ident ( $($el_type:tt $dim2: ident $el_tt: tt),* )) => {
        $crate::types::GeometryCollection::from_geometries(vec![
            $({
                const _: () = assert!(
                    matches!(
                        ($crate::dim!($dim), $crate::dim!($dim2)),
                        ($crate::types::Dimension::XY, $crate::types::Dimension::XY)
                            | ($crate::types::Dimension::XYZ, $crate::types::Dimension::XYZ)
                            | ($crate::types::Dimension::XYM, $crate::types::Dimension::XYM)
                            | ($crate::types::Dimension::XYZM, $crate::types::Dimension::XYZM)
                    ),
                    concat!("Cannot add member with ", stringify!($dim2), " dimension to GEOMETRYCOLLECTION ", stringify!($dim))
                );
                $crate::wkt!($el_type $dim $el_tt).into()
           }),*
        ]).unwrap()
    };
}

#[cfg(test)]
mod test {
    use crate::types::*;
    use crate::Wkt;

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
        assert_eq!(line_string.coords[0], coord! { 1.0 2.0 });
        assert_eq!(line_string.dim, Dimension::XY);

        let line_string = wkt! { LINESTRING Z (1.0 2.0 3.0, 3.0 4.0 5.0) };
        assert_eq!(line_string.coords.len(), 2);
        assert_eq!(line_string.coords[0], coord! { Z 1.0 2.0 3.0 });
        assert_eq!(line_string.dim, Dimension::XYZ);

        let line_string = wkt! { LINESTRING M (1.0 2.0 3.0, 3.0 4.0 5.0) };
        assert_eq!(line_string.coords.len(), 2);
        assert_eq!(line_string.coords[0], coord! { M 1.0 2.0 3.0 });
        assert_eq!(line_string.dim, Dimension::XYM);

        let line_string = wkt! { LINESTRING ZM (1.0 2.0 3.0 4.0, 3.0 4.0 5.0 6.0) };
        assert_eq!(line_string.coords.len(), 2);
        assert_eq!(line_string.coords[0], coord! { ZM 1.0 2.0 3.0 4.0 });
        assert_eq!(line_string.dim, Dimension::XYZM);
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
        assert_eq!(polygon.rings[0].coords[0], coord! { 1.0 2.0 });
        assert_eq!(polygon.dim, Dimension::XY);

        let polygon = wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
        assert_eq!(polygon.rings[0].coords.len(), 1);
        assert_eq!(polygon.rings[1].coords.len(), 1);
        assert_eq!(polygon.rings[0].coords[0], coord! { 1.0 2.0 });
        assert_eq!(polygon.rings[1].coords[0], coord! { 1.1 2.1 });

        let polygon = wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
        assert_eq!(polygon.rings.len(), 3);
        assert_eq!(polygon.rings[0].coords.len(), 2);
        assert_eq!(polygon.rings[2].coords[1], coord! { 3.2 4.2 });
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
        assert_eq!(multi_point.points[0], point! { Z (1.0 2.0 3.0) });
        assert_eq!(multi_point.dim, Dimension::XYZ);

        let multi_point = wkt! { MULTIPOINT Z (1.0 2.0 3.0, 4.0 5.0 6.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point! { Z (1.0 2.0 3.0) });
        assert_eq!(multi_point.points[1], point! { Z (4.0 5.0 6.0) });
        assert_eq!(multi_point.dim, Dimension::XYZ);

        let multi_point = wkt! { MULTIPOINT M (1.0 2.0 3.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point! { M (1.0 2.0 3.0) });
        assert_eq!(multi_point.dim, Dimension::XYM);

        let multi_point = wkt! { MULTIPOINT M (1.0 2.0 3.0, 4.0 5.0 6.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point! { M (1.0 2.0 3.0) });
        assert_eq!(multi_point.points[1], point! { M (4.0 5.0 6.0) });
        assert_eq!(multi_point.dim, Dimension::XYM);

        let multi_point = wkt! { MULTIPOINT ZM (1.0 2.0 3.0 4.0) };
        assert_eq!(multi_point.points.len(), 1);
        assert_eq!(multi_point.points[0], point! { ZM (1.0 2.0 3.0 4.0) });
        assert_eq!(multi_point.dim, Dimension::XYZM);

        let multi_point = wkt! { MULTIPOINT ZM (1.0 2.0 3.0 4.0, 4.0 5.0 6.0 7.0) };
        assert_eq!(multi_point.points.len(), 2);
        assert_eq!(multi_point.points[0], point! { ZM (1.0 2.0 3.0 4.0) });
        assert_eq!(multi_point.points[1], point! { ZM (4.0 5.0 6.0 7.0) });
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
            coord! { 3.0 4.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XY);

        let multi_line_string = wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord! { 7.0 8.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XY);

        let multi_line_string = wkt! { MULTILINESTRING Z ((1.0 2.0 3.0,3.0 4.0 5.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord! { Z 3.0 4.0 5.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZ);

        let multi_line_string =
            wkt! { MULTILINESTRING Z ((1.0 2.0 3.0,3.0 4.0 5.0),(5.0 6.0 7.0,7.0 8.0 9.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord! { Z 7.0 8.0 9.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZ);

        let multi_line_string = wkt! { MULTILINESTRING M ((1.0 2.0 3.0,3.0 4.0 5.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord! { M 3.0 4.0 5.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYM);

        let multi_line_string =
            wkt! { MULTILINESTRING M ((1.0 2.0 3.0,3.0 4.0 5.0),(5.0 6.0 7.0,7.0 8.0 9.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord! { M 7.0 8.0 9.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYM);

        let multi_line_string = wkt! { MULTILINESTRING ZM ((1.0 2.0 3.0 4.0,3.0 4.0 5.0 6.0)) };
        assert_eq!(multi_line_string.line_strings.len(), 1);
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[0].coords[1],
            coord! { ZM 3.0 4.0 5.0 6.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZM);

        let multi_line_string = wkt! { MULTILINESTRING ZM ((1.0 2.0 3.0 4.0,3.0 4.0 5.0 6.0),(5.0 6.0 7.0 8.0,7.0 8.0 9.0 10.0)) };
        assert_eq!(multi_line_string.line_strings[0].coords.len(), 2);
        assert_eq!(
            multi_line_string.line_strings[1].coords[1],
            coord! { ZM 7.0 8.0 9.0 10.0 }
        );
        assert_eq!(multi_line_string.dim, Dimension::XYZM);
    }

    #[test]
    fn empty_multi_polygon() {
        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON EMPTY };
        assert!(multi_polygon.polygons.is_empty());
        assert_eq!(multi_polygon.dim, Dimension::XY);

        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON Z EMPTY };
        assert!(multi_polygon.polygons.is_empty());
        assert_eq!(multi_polygon.dim, Dimension::XYZ);

        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON M EMPTY };
        assert!(multi_polygon.polygons.is_empty());
        assert_eq!(multi_polygon.dim, Dimension::XYM);

        let multi_polygon: MultiPolygon = wkt! { MULTIPOLYGON ZM EMPTY };
        assert!(multi_polygon.polygons.is_empty());
        assert_eq!(multi_polygon.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { MULTIPOLYGON() }
    }

    #[test]
    fn multi_line_polygon() {
        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0))) };
        assert_eq!(multi_polygon.polygons.len(), 1);
        assert_eq!(
            multi_polygon.polygons[0].rings[0].coords[0],
            coord! { 1.0 2.0}
        );
        assert_eq!(multi_polygon.dim, Dimension::XY);

        let multi_polygon = wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)),((1.0 2.0))) };
        assert_eq!(multi_polygon.polygons.len(), 2);
        assert_eq!(
            multi_polygon.polygons[0].rings[2].coords[0],
            coord! { 1.2 2.2}
        );
        assert_eq!(multi_polygon.dim, Dimension::XY);

        let multi_polygon = wkt! { MULTIPOLYGON Z (((1.0 2.0 3.0))) };
        assert_eq!(multi_polygon.polygons.len(), 1);
        assert_eq!(
            multi_polygon.polygons[0].rings[0].coords[0],
            coord! { Z 1.0 2.0 3.0 }
        );
        assert_eq!(multi_polygon.dim, Dimension::XYZ);

        let multi_polygon = wkt! { MULTIPOLYGON Z (((1.0 2.0 3.0,3.0 4.0 5.0), (1.1 2.1 3.1,3.1 4.1 5.1), (1.2 2.2 3.2,3.2 4.2 5.2)),((1.0 2.0 3.0))) };
        assert_eq!(multi_polygon.polygons.len(), 2);
        assert_eq!(
            multi_polygon.polygons[0].rings[2].coords[0],
            coord! { Z 1.2 2.2 3.2}
        );
        assert_eq!(multi_polygon.dim, Dimension::XYZ);

        let multi_polygon = wkt! { MULTIPOLYGON M (((1.0 2.0 3.0))) };
        assert_eq!(multi_polygon.polygons.len(), 1);
        assert_eq!(
            multi_polygon.polygons[0].rings[0].coords[0],
            coord! { M 1.0 2.0 3.0 }
        );
        assert_eq!(multi_polygon.dim, Dimension::XYM);

        let multi_polygon = wkt! { MULTIPOLYGON M (((1.0 2.0 3.0,3.0 4.0 5.0), (1.1 2.1 3.1,3.1 4.1 5.1), (1.2 2.2 3.2,3.2 4.2 5.2)),((1.0 2.0 3.0))) };
        assert_eq!(multi_polygon.polygons.len(), 2);
        assert_eq!(
            multi_polygon.polygons[0].rings[2].coords[0],
            coord! { M 1.2 2.2 3.2}
        );
        assert_eq!(multi_polygon.dim, Dimension::XYM);

        let multi_polygon = wkt! { MULTIPOLYGON ZM (((1.0 2.0 3.0 4.0))) };
        assert_eq!(multi_polygon.polygons.len(), 1);
        assert_eq!(
            multi_polygon.polygons[0].rings[0].coords[0],
            coord! { ZM 1.0 2.0 3.0 4.0 }
        );
        assert_eq!(multi_polygon.dim, Dimension::XYZM);

        let multi_polygon = wkt! { MULTIPOLYGON ZM (((1.0 2.0 3.0 4.0,3.0 4.0 5.0 6.0), (1.1 2.1 3.1 4.1,3.1 4.1 5.1 6.1), (1.2 2.2 3.2 4.2,3.2 4.2 5.2 6.2)),((1.0 2.0 3.0 4.0))) };
        assert_eq!(multi_polygon.polygons.len(), 2);
        assert_eq!(
            multi_polygon.polygons[0].rings[2].coords[0],
            coord! { ZM 1.2 2.2 3.2 4.2}
        );
        assert_eq!(multi_polygon.dim, Dimension::XYZM);
    }

    #[test]
    fn empty_geometry_collection() {
        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION EMPTY };
        assert!(geometry_collection.geoms.is_empty());
        assert_eq!(geometry_collection.dim, Dimension::XY);

        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION Z EMPTY };
        assert!(geometry_collection.geoms.is_empty());
        assert_eq!(geometry_collection.dim, Dimension::XYZ);

        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION M EMPTY };
        assert!(geometry_collection.geoms.is_empty());
        assert_eq!(geometry_collection.dim, Dimension::XYM);

        let geometry_collection: GeometryCollection = wkt! { GEOMETRYCOLLECTION ZM EMPTY };
        assert!(geometry_collection.geoms.is_empty());
        assert_eq!(geometry_collection.dim, Dimension::XYZM);

        // This (rightfully) fails to compile because its invalid wkt
        // wkt! { GEOMETRYCOLLECTION() }
    }

    #[test]
    fn geometry_collection() {
        let geometry_collection: GeometryCollection = wkt! {
            GEOMETRYCOLLECTION (
                POINT (40.0 10.0),
                LINESTRING (10.0 10.0, 20.0 20.0, 10.0 40.0),
                POLYGON ((40.0 40.0, 20.0 45.0, 45.0 30.0, 40.0 40.0))
            )
        };
        assert_eq!(geometry_collection.geoms.len(), 3);
        assert_eq!(geometry_collection.dim, Dimension::XY);

        let line_string = match &geometry_collection.geometries()[1] {
            Wkt::LineString(line_string) => line_string,
            _ => unreachable!(),
        };
        assert_eq!(line_string.coords[1], coord! { 20.0 20.0 });

        let geometry_collection: GeometryCollection = wkt! {
            GEOMETRYCOLLECTION Z (
                POINT Z (40.0 10.0 50.0),
                LINESTRING Z (10.0 10.0 20.0, 20.0 20.0 30.0, 10.0 40.0 50.0),
                POLYGON Z ((40.0 40.0 50.0, 20.0 45.0 55.0, 45.0 30.0 60.0, 40.0 40.0 50.0))
            )
        };
        assert_eq!(geometry_collection.geoms.len(), 3);
        assert_eq!(geometry_collection.dim, Dimension::XYZ);

        let geometry_collection: GeometryCollection = wkt! {
            GEOMETRYCOLLECTION M (
                POINT M (40.0 10.0 50.0),
                LINESTRING M (10.0 10.0 20.0, 20.0 20.0 30.0, 10.0 40.0 50.0),
                POLYGON M ((40.0 40.0 50.0, 20.0 45.0 55.0, 45.0 30.0 60.0, 40.0 40.0 50.0))
            )
        };
        assert_eq!(geometry_collection.geoms.len(), 3);
        assert_eq!(geometry_collection.dim, Dimension::XYM);

        let geometry_collection: GeometryCollection = wkt! {
            GEOMETRYCOLLECTION ZM (
                POINT ZM (40.0 10.0 50.0 60.0),
                LINESTRING ZM (10.0 10.0 20.0 30.0, 20.0 20.0 30.0 40.0, 10.0 40.0 50.0 60.0),
                POLYGON ZM ((40.0 40.0 50.0 60.0, 20.0 45.0 55.0 65.0, 45.0 30.0 60.0 70.0, 40.0 40.0 50.0 60.0))
            )
        };
        assert_eq!(geometry_collection.geoms.len(), 3);
        assert_eq!(geometry_collection.dim, Dimension::XYZM);
    }

    #[test]
    fn other_numeric_types() {
        let point: Point<i32> = wkt!(POINT(1 2));
        assert_eq!(point.coord.unwrap().x, 1i32);
        assert_eq!(point.coord.unwrap().y, 2i32);

        let point: Point<u64> = wkt!(POINT(1 2));
        assert_eq!(point.coord.unwrap().x, 1u64);
        assert_eq!(point.coord.unwrap().y, 2u64);

        let point: Point<f32> = wkt!(POINT(1.0 2.0));
        assert_eq!(point.coord.unwrap().x, 1f32);
        assert_eq!(point.coord.unwrap().y, 2f32);
    }
}
