#[macro_use]
extern crate criterion;

extern crate wkt;
use wkt::ToWkt;

use std::str::FromStr;

fn wkt_to_string(c: &mut criterion::Criterion) {
    c.bench_function("to_string small wkt", |bencher| {
        let s = include_str!("./small.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        bencher.iter(|| {
            let _ = w.to_string();
        });
    });

    c.bench_function("to_string big wkt", |bencher| {
        let s = include_str!("./big.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        bencher.iter(|| {
            let _ = w.to_string();
        });
    });
}

fn geo_to_wkt_string(c: &mut criterion::Criterion) {
    c.bench_function("geo: serialize small wkt string", |bencher| {
        let s = include_str!("./small.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            let _ = g.wkt_string();
        });
    });

    c.bench_function("geo: serialize big wkt string", |bencher| {
        let s = include_str!("./big.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            let _ = g.wkt_string();
        });
    });
}

fn geo_write_wkt(c: &mut criterion::Criterion) {
    c.bench_function("geo: write small wkt", |bencher| {
        let s = include_str!("./small.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            let _ = g.write_wkt(std::io::sink());
        });
    });

    c.bench_function("geo: write big wkt", |bencher| {
        let s = include_str!("./big.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            let _ = g.write_wkt(std::io::sink());
        });
    });
}

fn geo_write_wkt_as_trait(c: &mut criterion::Criterion) {
    c.bench_function("geo: write small wkt using trait", |bencher| {
        let s = include_str!("./small.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            wkt::to_wkt::write_geometry(&g, &mut String::new()).unwrap();
        });
    });

    c.bench_function("geo: write big wkt using trait", |bencher| {
        let s = include_str!("./big.wkt");
        let w = wkt::Wkt::<f64>::from_str(s).unwrap();
        let g = geo_types::Geometry::try_from(w).unwrap();
        bencher.iter(|| {
            wkt::to_wkt::write_geometry(&g, &mut String::new()).unwrap();
        });
    });
}

criterion_group!(
    benches,
    wkt_to_string,
    geo_to_wkt_string,
    geo_write_wkt,
    geo_write_wkt_as_trait
);
criterion_main!(benches);
