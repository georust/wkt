#[macro_use]
extern crate criterion;
extern crate wkt;

use std::str::FromStr;
use wkt::TryFromWkt;

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

criterion_group!(benches, bench_parse, bench_parse_to_geo);
criterion_main!(benches);
