#[macro_use]
extern crate criterion;
extern crate wkt;

use std::str::FromStr;

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
}

fn bench_parse_to_geo(c: &mut criterion::Criterion) {
    c.bench_function("parse small to geo", |bencher| {
        let s = include_str!("./small.wkt");
        bencher.iter(|| {
            let _ = geo_types::Geometry::try_from(wkt::Wkt::<f64>::from_str(s).unwrap());
        });
    });

    c.bench_function("parse big to geo", |bencher| {
        let s = include_str!("./big.wkt");
        bencher.iter(|| {
            let _ = geo_types::Geometry::try_from(wkt::Wkt::<f64>::from_str(s).unwrap());
        });
    });
}

criterion_group!(benches, bench_parse, bench_parse_to_geo);
criterion_main!(benches);
