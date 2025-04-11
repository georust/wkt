/// ```
/// use wkt::wkt;
/// let point = wkt::wkt!(POINT(1.0 2.0));
/// println!("point is: {point}");
/// ```
#[macro_export(local_inner_macros)]
macro_rules! wkt {
    // Hide distracting implementation details from the generated rustdoc.
    ($($wkt:tt)+) => {
        {
            use $crate::{types::*, Wkt};
            wkt_internal!($($wkt)+)
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! wkt_internal {
    (POINT $tt: tt) => {
        Wkt::Point(point!($tt))
    };
    (LINESTRING $tt: tt) => {
        Wkt::LineString(line_string!($tt))
    };
    (POLYGON $tt:tt) => {
        Wkt::Polygon(polygon!($tt))
    };
    (MULTIPOINT $tt: tt) => {
        Wkt::MultiPoint(multi_point!($tt))
    };
    (MULTILINESTRING $tt: tt) => {
        Wkt::MultiLineString(multi_line_string!($tt))
    };
    (MULTIPOLYGON $tt: tt) => {
        Wkt::MultiPolygon(multi_polygon!($tt))
    };
    (GEOMETRYCOLLECTION $tt: tt) => {
        Wkt::GeometryCollection(geometry_collection!($tt))
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
        $crate::point_vec!(@points [$($el,)* $crate::point_el!(EMPTY),] $($rest)*)
    };
    // Next element is an expression followed by comma.
    (@points [$($el:expr,)*] $x:literal $y:literal, $($rest:tt)*) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el!($x $y),] $($rest)*)
    };

    (@points [$($el:expr,)*] EMPTY) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el!(EMPTY)])
    };
    (@points [$($el:expr,)*] $x: literal $y:literal) => {
        $crate::point_vec!(@points [$($el,)* $crate::point_el!($x $y)])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! coord {
    ($x: literal $y: literal) => {
        Coord {
            x: $x,
            y: $y,
            z: None,
            m: None,
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point_el {
    (EMPTY) => {
        Point(None)
    };
    ($x: literal $y: literal) => {
        Point(Some(coord!($x $y)))
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! point {
    (EMPTY) => {
        point_el!(EMPTY)
    };
    (($x: literal $y: literal)) => {
        point_el!($x $y)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! line_string {
    (()) => {
        compile_error!("use `LINESTRING EMPTY` for a LineString with no coordinates")
    };
    (EMPTY) => {
        LineString(vec![])
    };
    (($($x: literal $y: literal),*)) => {
        LineString(
            vec![$($crate::coord!($x $y)),*]
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! polygon {
    (EMPTY) => {
        Polygon(vec![])
    };
    (()) => {
        compile_error!("use `POLYGON EMPTY` for a Polygon with no coordinates")
    };
    (( $($line_string_tt: tt),* )) => {
        Polygon(vec![
           $($crate::line_string![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_point {
    (EMPTY) => {
        MultiPoint(vec![])
    };
    (()) => {
        compile_error!("use `MULTIPOINT EMPTY` for a MultiPoint with no coordinates")
    };
    (($($tt: tt)*)) => {
        MultiPoint(
            point_vec!(@points [] $($tt)*)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_line_string {
    (EMPTY) => {
        MultiLineString(vec![])
    };
    (()) => {
        compile_error!("use `MULTILINESTRING EMPTY` for a MultiLineString with no coordinates")
    };
    (( $($line_string_tt: tt),* )) => {
        MultiLineString(vec![
           $($crate::line_string![$line_string_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! multi_polygon{
    (EMPTY) => {
        MultiPolygon(vec![])
    };
    (()) => {
        compile_error!("use `MULTIPOLYGON EMPTY` for a MultiPolygon with no coordinates")
    };
    (( $($polygon_tt: tt),* )) => {
        MultiPolygon(vec![
           $($crate::polygon![$polygon_tt]),*
        ])
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! geometry_collection {
    (EMPTY) => {
        GeometryCollection(vec![])
    };
    (()) => {
        compile_error!("use `GEOMETRYCOLLECTION EMPTY` for an empty collection")
    };
    (( $($el_type:tt $el_tt: tt),* )) => {
        GeometryCollection(vec![
           $($crate::wkt_internal!($el_type $el_tt)),*
        ])
    };
}

#[cfg(test)]
mod test {
    use crate::Wkt;

    use std::str::FromStr;

    macro_rules! assert_wkt {
        ($($wkt_tokens: tt)*) => {
            let wkt_from_str: Wkt<f64> = Wkt::from_str(stringify!($($wkt_tokens)*)).unwrap();
            assert_eq!(wkt_from_str, wkt!($($wkt_tokens)*));
        }
    }

    #[test]
    fn point() {
        assert_wkt! { POINT(1.0 2.0) };
        assert_wkt! { POINT(1.0   2.0) };
    }

    #[test]
    fn empty_point() {
        assert_wkt! { POINT EMPTY };
    }

    #[test]
    fn line_string() {
        assert_wkt! { LINESTRING(1.0 2.0,3.0 4.0) };
        assert_wkt! { LINESTRING(1.0 2.0, 3.0 4.0) };
        assert_wkt! { LINESTRING(1.0 2.0) };
        assert_wkt! { LINESTRING EMPTY };
        // This fails to compile (as it should)
        // assert_wkt! { LINESTRING () };
    }

    #[test]
    fn empty_polygon() {
        assert_wkt! { POLYGON EMPTY };
    }

    #[test]
    fn polygon() {
        assert_wkt! { POLYGON((1.0 2.0)) };
        assert_wkt! { POLYGON((1.0 2.0,3.0 4.0)) };
        assert_wkt! { POLYGON((1.0 2.0), (1.1 2.1)) };
        assert_wkt! { POLYGON((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)) };
        // This fails to compile (as it should)
        // assert_wkt! { POLYGON(()) }
    }

    #[test]
    fn empty_multi_point() {
        assert_wkt! { MULTIPOINT EMPTY };
        // This fails to compile (as it should)
        // assert_wkt! { MULTIPOINT() }
    }

    #[test]
    fn multi_point_with_some_empty() {
        let wkt = wkt!(MULTIPOINT(1.0 2.0, EMPTY));
        let Wkt::MultiPoint(mp) = wkt else {
            panic!("expected multipoint")
        };
        assert!(mp.0[0].0.is_some());
        assert!(mp.0[1].0.is_none());

        // This currently fails because the from_str impl doesn't handle a mix of empty points
        // while the macro behaves correctly. See https://github.com/georust/wkt/issues/111
        //
        // assert_wkt! { MULTIPOINT(1.0 2.0, EMPTY) };
    }
    #[test]
    fn multi_point() {
        assert_wkt! { MULTIPOINT(1.0 2.0) };
        assert_wkt! { MULTIPOINT(1.0 2.0,3.0 4.0) };
    }

    #[test]
    fn multi_line_string() {
        assert_wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0)) }
        assert_wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),(5.0 6.0,7.0 8.0)) }
        assert_wkt! { MULTILINESTRING EMPTY }
        assert_wkt! { MULTILINESTRING ((1.0 2.0,3.0 4.0),EMPTY) }
    }

    #[test]
    fn multi_line_polygon() {
        assert_wkt! { MULTIPOLYGON EMPTY }
        assert_wkt! { MULTIPOLYGON (((1.0 2.0))) }
        assert_wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)),((1.0 2.0))) }
        assert_wkt! { MULTIPOLYGON (((1.0 2.0,3.0 4.0), (1.1 2.1,3.1 4.1), (1.2 2.2,3.2 4.2)), EMPTY) }
    }

    #[test]
    fn geometry_collectio() {
        assert_wkt! { GEOMETRYCOLLECTION EMPTY }
        assert_wkt! { GEOMETRYCOLLECTION (POINT (40.0 10.0), LINESTRING (10.0 10.0, 20.0 20.0, 10.0 40.0), POLYGON ((40.0 40.0, 20.0 45.0, 45.0 30.0, 40.0 40.0))) }
    }

    #[test]
    fn other_numeric_types() {
        let wkt: Wkt<i32> = wkt!(POINT(1 2));
        let Wkt::Point(point) = wkt else {
            panic!("unexpected wkt");
        };
        assert_eq!(point.0.clone().unwrap().x, 1i32);
        assert_eq!(point.0.clone().unwrap().y, 2i32);

        let wkt: Wkt<u64> = wkt!(POINT(1 2));
        let Wkt::Point(point) = wkt else {
            panic!("unexpected wkt");
        };
        assert_eq!(point.0.clone().unwrap().x, 1u64);
        assert_eq!(point.0.clone().unwrap().y, 2u64);

        let wkt: Wkt<f32> = wkt!(POINT(1.0 2.0));
        let Wkt::Point(point) = wkt else {
            panic!("unexpected wkt");
        };
        assert_eq!(point.0.clone().unwrap().x, 1.0f32);
        assert_eq!(point.0.clone().unwrap().y, 2.0f32);
    }
}
