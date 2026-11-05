#[macro_use]
extern crate criterion;
extern crate wkt;

use std::str::FromStr;
use wkt::TryFromWkt;

#[path = "support/wkb.rs"]
mod wkb;

// Returns a MULTIPOLYGON of 400 closed rings with 111 coordinates each.
// The coordinates are longitude and latitude values at full f64 precision,
// from a fixed-seed xorshift generator so that each run uses the same input.
fn lonlat_wkt() -> String {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next_unit = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    let polygons: Vec<String> = (0..400)
        .map(|_| {
            let mut ring: Vec<String> = (0..110)
                .map(|_| {
                    format!(
                        "{} {}",
                        next_unit() * 360.0 - 180.0,
                        next_unit() * 180.0 - 90.0
                    )
                })
                .collect();
            ring.push(ring[0].clone());
            format!("(({}))", ring.join(", "))
        })
        .collect();
    format!("MULTIPOLYGON({})", polygons.join(", "))
}

fn bench_parse(c: &mut criterion::Criterion) {
    c.bench_function("parse small", |bencher| {
        let s = include_str!("./small.wkt");
        bencher.iter(|| {
            let _ = wkt::Wkt::<f64>::from_str(s).unwrap();
        });
    });

    c.bench_function("parse big", |bencher| {
        let s = include_str!("./big.wkt");
        bencher.iter(|| {
            let _ = wkt::Wkt::<f64>::from_str(s).unwrap();
        });
    });

    c.bench_function("parse lonlat", |bencher| {
        let s = lonlat_wkt();
        bencher.iter(|| {
            let _ = wkt::Wkt::<f64>::from_str(&s).unwrap();
        });
    });
}

fn bench_parse_to_geo(c: &mut criterion::Criterion) {
    c.bench_function("parse small to geo", |bencher| {
        let s = include_str!("./small.wkt");
        bencher.iter(|| {
            let _ = geo_types::Geometry::<f64>::try_from_wkt_str(s).unwrap();
        });
    });

    c.bench_function("parse big to geo", |bencher| {
        let s = include_str!("./big.wkt");
        bencher.iter(|| {
            let _ = geo_types::Geometry::<f64>::try_from_wkt_str(s).unwrap();
        });
    });

    c.bench_function("parse lonlat to geo", |bencher| {
        let s = lonlat_wkt();
        bencher.iter(|| {
            let _ = geo_types::Geometry::<f64>::try_from_wkt_str(&s).unwrap();
        });
    });
}

fn bench_validate(c: &mut criterion::Criterion) {
    c.bench_function("validate small", |bencher| {
        let s = include_str!("./small.wkt");
        bencher.iter(|| wkt::visitor::validate::<f64>(s).unwrap());
    });

    c.bench_function("validate big", |bencher| {
        let s = include_str!("./big.wkt");
        bencher.iter(|| wkt::visitor::validate::<f64>(s).unwrap());
    });

    c.bench_function("validate lonlat", |bencher| {
        let s = lonlat_wkt();
        bencher.iter(|| wkt::visitor::validate::<f64>(&s).unwrap());
    });
}

// Inputs on which the two WKB converters must write the same bytes.
const WKB_CASES: &[&str] = &[
    "POINT (1 2)",
    "POINT EMPTY",
    "POINT ZM (1 2 3 4)",
    "LINESTRING M (1 2 3, 4 5 6)",
    "LINESTRING EMPTY",
    "POLYGON ((0 0, 1 0, 1 1, 0 0), (0.1 0.1, 0.2 0.1, 0.1 0.1))",
    "POLYGON EMPTY",
    "MULTIPOINT (1 2, EMPTY, (3 4))",
    "MULTIPOINT Z EMPTY",
    "MULTILINESTRING ((0 0, 1 1), EMPTY)",
    "MULTIPOLYGON (((0 0, 1 0, 0 0)), EMPTY)",
    "GEOMETRYCOLLECTION (POINT Z (1 2 3), GEOMETRYCOLLECTION (LINESTRING (0 0, 1 1)), POINT EMPTY)",
    "GEOMETRYCOLLECTION EMPTY",
];

fn assert_same_wkb(input: &str) {
    let (mut a, mut b) = (Vec::new(), Vec::new());
    wkb::via_wkt(input, &mut a).unwrap();
    wkb::via_visitor(input, &mut b).unwrap();
    // NaN is not equal to NaN, so compare the bytes.
    assert_eq!(a, b, "input: {input}");
}

fn bench_to_wkb(c: &mut criterion::Criterion) {
    for input in WKB_CASES {
        assert_same_wkb(input);
    }
    let lonlat = lonlat_wkt();
    let inputs = [
        ("small", include_str!("./small.wkt")),
        ("big", include_str!("./big.wkt")),
        ("lonlat", lonlat.as_str()),
    ];
    for (name, s) in inputs {
        assert_same_wkb(s);
        c.bench_function(&format!("to wkb {name} via Wkt"), |bencher| {
            let mut buf = Vec::new();
            bencher.iter(|| {
                buf.clear();
                wkb::via_wkt(s, &mut buf).unwrap();
            });
        });
        c.bench_function(&format!("to wkb {name} via visitor"), |bencher| {
            let mut buf = Vec::new();
            bencher.iter(|| {
                buf.clear();
                wkb::via_visitor(s, &mut buf).unwrap();
            });
        });
    }
}

criterion_group!(
    benches,
    bench_parse,
    bench_parse_to_geo,
    bench_validate,
    bench_to_wkb
);
criterion_main!(benches);
