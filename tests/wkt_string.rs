//! Serialization compatibility and allocation regression for geo-types strings.
use std::io;
use wkt::{ToWkt, Wkt};

#[test]
fn external_implementations_keep_the_default_and_dimensions() {
    struct External(Wkt<f64>);
    impl ToWkt<f64> for External {
        fn to_wkt(&self) -> Wkt<f64> {
            self.0.clone()
        }

        fn write_wkt(&self, _: impl io::Write) -> io::Result<()> {
            Err(io::Error::other("custom io implementation"))
        }
    }

    for text in [
        "POINT Z(1 2 3)",
        "POINT M(1 2 4)",
        "POINT ZM(1 2 3 4)",
        "POINT Z EMPTY",
        "LINESTRING ZM EMPTY",
        "POLYGON M EMPTY",
        "MULTIPOINT Z((1 2 3),EMPTY)",
        "GEOMETRYCOLLECTION Z(POINT Z(1 2 3),LINESTRING Z EMPTY)",
    ] {
        let value = External(text.parse().unwrap());
        assert_eq!(value.wkt_string(), value.0.to_string());
        assert!(value.write_wkt(Vec::new()).is_err());
    }
}

#[cfg(feature = "geo-types")]
#[test]
fn geo_types_match_the_previous_string_for_every_kind() {
    use geo_types::{
        line_string, polygon, Coord, Geometry, GeometryCollection, Line, LineString,
        MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
    };

    macro_rules! check {
        ($kind:ident, $value:expr) => {{
            let value = $value;
            let previous = value.to_wkt().to_string();
            assert_eq!(value.wkt_string(), previous);
            let geometry: Geometry<_> = Geometry::$kind(value);
            assert_eq!(geometry.wkt_string(), previous);
        }};
    }

    check!(Point, Point::new(1i32, 2));
    check!(Point, Point::new(1.25f32, -2.5));
    check!(Point, Point::new(-0.0f64, f64::INFINITY));
    check!(Point, Point::new(f64::NAN, 2.0));
    check!(Line, Line::new((1., 2.), (3., 4.)));
    check!(LineString, line_string![(x: 1., y: 2.), (x: 3., y: 4.)]);
    check!(LineString, LineString::<f64>::new(vec![]));
    let outer = line_string![(x: 0., y: 0.), (x: 4., y: 0.), (x: 0., y: 4.), (x: 0., y: 0.)];
    let hole = line_string![(x: 1., y: 1.), (x: 2., y: 1.), (x: 1., y: 2.), (x: 1., y: 1.)];
    check!(Polygon, Polygon::new(outer.clone(), vec![hole.clone()]));
    check!(
        Polygon,
        Polygon::<f64>::new(LineString::new(vec![]), vec![])
    );
    check!(Polygon, Polygon::new(outer, vec![LineString::new(vec![])]));
    let unusual = Polygon::new(LineString::new(vec![]), vec![hole]);
    check!(Polygon, unusual.clone());
    check!(MultiPoint, MultiPoint::new(vec![Point::new(1., 2.)]));
    check!(MultiPoint, MultiPoint::<f64>::new(vec![]));
    check!(
        MultiLineString,
        MultiLineString::new(vec![LineString::new(vec![]), line_string![(x: 1., y: 2.)]])
    );
    check!(MultiLineString, MultiLineString::<f64>::new(vec![]));
    check!(
        MultiPolygon,
        MultiPolygon::new(vec![Polygon::new(LineString::new(vec![]), vec![]), unusual])
    );
    check!(MultiPolygon, MultiPolygon::<f64>::new(vec![]));
    check!(
        Polygon,
        polygon![(x: 0., y: 0.), (x: 4., y: 0.), (x: 0., y: 4.)]
    );
    check!(
        Triangle,
        Triangle::new(
            Coord { x: 0., y: 0. },
            Coord { x: 4., y: 0. },
            Coord { x: 0., y: 4. }
        )
    );
    check!(
        Triangle,
        Triangle::new(
            Coord { x: 0., y: 0. },
            Coord { x: 0., y: 0. },
            Coord { x: 0., y: 0. }
        )
    );
    check!(Rect, Rect::new((4., 4.), (8., 8.)));
    check!(Rect, Rect::new((0., 0.), (0., 0.)));

    // A NaN endpoint compares unequal to itself when geo-types closes a ring.
    let nan_rect = Rect::new((f64::NAN, 0.), (f64::NAN, 1.));
    let nan_triangle = Triangle::new(
        Coord { x: f64::NAN, y: 0. },
        Coord { x: 1., y: 0. },
        Coord { x: 0., y: 1. },
    );
    check!(Rect, nan_rect);
    check!(Triangle, nan_triangle);
    check!(
        GeometryCollection,
        GeometryCollection::new_from(vec![
            Geometry::Rect(nan_rect),
            Geometry::Triangle(nan_triangle),
        ])
    );
    check!(
        GeometryCollection,
        GeometryCollection::<f64>::new_from(vec![])
    );
    check!(
        GeometryCollection,
        GeometryCollection::new_from(vec![
            Geometry::Rect(Rect::new((4., 4.), (8., 8.))),
            Geometry::GeometryCollection(GeometryCollection::new_from(vec![
                Geometry::Rect(Rect::new((0., 0.), (0., 0.))),
                Geometry::MultiPoint(MultiPoint::new(vec![])),
            ])),
        ])
    );
}

#[cfg(feature = "geo-types")]
mod allocations {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
        static BYTES: Cell<usize> = const { Cell::new(0) };
        static CALLS: Cell<usize> = const { Cell::new(0) };
    }

    struct CountingAllocator;
    fn record(bytes: usize) {
        let _ = ENABLED.try_with(|enabled| {
            if enabled.get() {
                BYTES.with(|total| total.set(total.get() + bytes));
                CALLS.with(|total| total.set(total.get() + 1));
            }
        });
    }

    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record(layout.size());
            System.alloc(layout)
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            record(layout.size());
            System.alloc_zeroed(layout)
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout)
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            record(size);
            System.realloc(ptr, layout, size)
        }
    }

    #[global_allocator]
    static ALLOCATOR: CountingAllocator = CountingAllocator;

    fn measure(f: impl FnOnce() -> String) -> (String, usize, usize) {
        BYTES.with(|n| n.set(0));
        CALLS.with(|n| n.set(0));
        ENABLED.with(|enabled| enabled.set(true));
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                ENABLED.with(|enabled| enabled.set(false));
            }
        }
        let reset = Reset;
        let result = f();
        drop(reset);
        (result, BYTES.with(Cell::get), CALLS.with(Cell::get))
    }

    #[test]
    fn avoids_intermediate_polygon_allocation() {
        use geo_types::{Geometry, GeometryCollection, LineString, Polygon};
        let ring = LineString::from(
            (0..1024)
                .map(|i| (i as f64, (i % 7) as f64))
                .collect::<Vec<_>>(),
        );
        let polygon = Polygon::new(ring.clone(), vec![ring]);
        let collection = GeometryCollection::new_from(vec![Geometry::Polygon(polygon); 4]);
        let previous = measure(|| collection.to_wkt().to_string());
        let direct = measure(|| collection.wkt_string());
        assert_eq!(previous.0, direct.0);
        println!(
            "previous allocated bytes={}, calls={}; direct bytes={}, calls={}",
            previous.1, previous.2, direct.1, direct.2
        );
        assert!(
            direct.1 < previous.1,
            "intermediate allocation was not removed"
        );
        assert!(
            direct.2 < previous.2,
            "intermediate allocation count was not reduced"
        );
    }
}
