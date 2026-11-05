//! Two WKT to WKB converters for the conversion benchmarks. Both write ISO WKB in
//! little-endian byte order with the same functions. `via_wkt` builds a `Wkt` value and
//! writes it through `geo_traits`. `via_visitor` writes from `visit_wkt` and patches each
//! count when its part ends.

use geo_traits::{
    CoordTrait, Dimensions, GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait,
};
use std::str::FromStr;
use wkt::error::ParseError;
use wkt::types::Dimension;
use wkt::visitor::{
    visit_wkt, CoordsVisitor, GeometryCollectionVisitor, GeometryVisitor, MultiLineStringVisitor,
    MultiPointVisitor, MultiPolygonVisitor, PointVisitor, PolygonVisitor,
};
use wkt::Wkt;

const POINT: u32 = 1;
const LINE_STRING: u32 = 2;
const POLYGON: u32 = 3;
const MULTI_POINT: u32 = 4;
const MULTI_LINE_STRING: u32 = 5;
const MULTI_POLYGON: u32 = 6;
const GEOMETRY_COLLECTION: u32 = 7;

fn size(dim: Dimension) -> usize {
    match dim {
        Dimension::XY => 2,
        Dimension::XYZ | Dimension::XYM => 3,
        Dimension::XYZM => 4,
    }
}

fn header(buf: &mut Vec<u8>, base: u32, dim: Dimension) {
    let code = base
        + match dim {
            Dimension::XY => 0,
            Dimension::XYZ => 1000,
            Dimension::XYM => 2000,
            Dimension::XYZM => 3000,
        };
    buf.push(1);
    buf.extend_from_slice(&code.to_le_bytes());
}

fn count(buf: &mut Vec<u8>, n: usize) {
    buf.extend_from_slice(&(n as u32).to_le_bytes());
}

fn value(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

// WKB has no empty point. Write NaN for each value.
fn empty_point(buf: &mut Vec<u8>, dim: Dimension) {
    for _ in 0..size(dim) {
        value(buf, f64::NAN);
    }
}

/// Parses `input` into a `Wkt` value and writes the value as WKB.
pub fn via_wkt(input: &str, buf: &mut Vec<u8>) -> Result<(), ParseError> {
    write_geometry(buf, &Wkt::<f64>::from_str(input)?);
    Ok(())
}

fn to_dimension(dim: Dimensions) -> Dimension {
    match dim {
        Dimensions::Xy => Dimension::XY,
        Dimensions::Xyz => Dimension::XYZ,
        Dimensions::Xym => Dimension::XYM,
        Dimensions::Xyzm => Dimension::XYZM,
        Dimensions::Unknown(_) => unimplemented!("WKT has no unknown dimension"),
    }
}

fn write_geometry(buf: &mut Vec<u8>, g: &impl GeometryTrait<T = f64>) {
    let dim = to_dimension(g.dim());
    match g.as_type() {
        GeometryType::Point(p) => {
            header(buf, POINT, dim);
            write_point(buf, p, dim);
        }
        GeometryType::LineString(ls) => {
            header(buf, LINE_STRING, dim);
            write_coords(buf, ls, dim);
        }
        GeometryType::Polygon(p) => {
            header(buf, POLYGON, dim);
            write_rings(buf, p, dim);
        }
        GeometryType::MultiPoint(mp) => {
            header(buf, MULTI_POINT, dim);
            count(buf, mp.num_points());
            for p in mp.points() {
                header(buf, POINT, dim);
                write_point(buf, &p, dim);
            }
        }
        GeometryType::MultiLineString(mls) => {
            header(buf, MULTI_LINE_STRING, dim);
            count(buf, mls.num_line_strings());
            for ls in mls.line_strings() {
                header(buf, LINE_STRING, dim);
                write_coords(buf, &ls, dim);
            }
        }
        GeometryType::MultiPolygon(mp) => {
            header(buf, MULTI_POLYGON, dim);
            count(buf, mp.num_polygons());
            for p in mp.polygons() {
                header(buf, POLYGON, dim);
                write_rings(buf, &p, dim);
            }
        }
        GeometryType::GeometryCollection(gc) => {
            header(buf, GEOMETRY_COLLECTION, dim);
            count(buf, gc.num_geometries());
            for member in gc.geometries() {
                write_geometry(buf, &member);
            }
        }
        _ => unimplemented!("WKT parsing does not make this type"),
    }
}

fn write_coord(buf: &mut Vec<u8>, c: &impl CoordTrait<T = f64>, dim: Dimension) {
    for n in 0..size(dim) {
        value(buf, c.nth_or_panic(n));
    }
}

fn write_point(buf: &mut Vec<u8>, p: &impl PointTrait<T = f64>, dim: Dimension) {
    match p.coord() {
        Some(c) => write_coord(buf, &c, dim),
        None => empty_point(buf, dim),
    }
}

fn write_coords(buf: &mut Vec<u8>, ls: &impl LineStringTrait<T = f64>, dim: Dimension) {
    count(buf, ls.num_coords());
    for c in ls.coords() {
        write_coord(buf, &c, dim);
    }
}

fn write_rings(buf: &mut Vec<u8>, p: &impl PolygonTrait<T = f64>, dim: Dimension) {
    let exterior = p.exterior();
    count(buf, usize::from(exterior.is_some()) + p.num_interiors());
    for ring in exterior.into_iter().chain(p.interiors()) {
        write_coords(buf, &ring, dim);
    }
}

/// Parses `input` with a visitor that writes WKB.
pub fn via_visitor(input: &str, buf: &mut Vec<u8>) -> Result<(), ParseError> {
    visit_wkt(input, GeometryWriter { buf })
}

/// Writes the header of a geometry of any type.
struct GeometryWriter<'a> {
    buf: &'a mut Vec<u8>,
}

/// Writes the coordinate of a point.
struct PointWriter<'a> {
    buf: &'a mut Vec<u8>,
    dim: Dimension,
}

impl<'a> PointWriter<'a> {
    fn new(buf: &'a mut Vec<u8>, dim: Dimension) -> Self {
        header(buf, POINT, dim);
        PointWriter { buf, dim }
    }
}

/// Writes a part that has a count of elements: a line string, a ring, a polygon, a
/// multi-geometry or a geometry collection.
struct PartWriter<'a> {
    buf: &'a mut Vec<u8>,
    dim: Dimension,
    // The position of the count in `buf`.
    count_at: usize,
    count: u32,
}

impl<'a> PartWriter<'a> {
    // Writes a header if `base` is given, and reserves the count.
    fn new(buf: &'a mut Vec<u8>, base: Option<u32>, dim: Dimension) -> Self {
        if let Some(base) = base {
            header(buf, base, dim);
        }
        let count_at = buf.len();
        count(buf, 0);
        PartWriter {
            buf,
            dim,
            count_at,
            count: 0,
        }
    }

    // Counts a member and gives the buffer to its writer.
    fn member(&mut self) -> &mut Vec<u8> {
        self.count += 1;
        self.buf
    }

    fn finish(self) -> Result<(), ParseError> {
        self.buf[self.count_at..self.count_at + 4].copy_from_slice(&self.count.to_le_bytes());
        Ok(())
    }
}

impl<'a> GeometryVisitor<f64> for GeometryWriter<'a> {
    type Error = ParseError;
    type Output = ();
    type Point = PointWriter<'a>;
    type LineString = PartWriter<'a>;
    type Polygon = PartWriter<'a>;
    type MultiPoint = PartWriter<'a>;
    type MultiLineString = PartWriter<'a>;
    type MultiPolygon = PartWriter<'a>;
    type GeometryCollection = PartWriter<'a>;

    fn point(self, dim: Dimension) -> Result<PointWriter<'a>, ParseError> {
        Ok(PointWriter::new(self.buf, dim))
    }
    fn line_string(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(LINE_STRING), dim))
    }
    fn polygon(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(POLYGON), dim))
    }
    fn multi_point(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(MULTI_POINT), dim))
    }
    fn multi_line_string(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(MULTI_LINE_STRING), dim))
    }
    fn multi_polygon(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(MULTI_POLYGON), dim))
    }
    fn geometry_collection(self, dim: Dimension) -> Result<PartWriter<'a>, ParseError> {
        Ok(PartWriter::new(self.buf, Some(GEOMETRY_COLLECTION), dim))
    }
}

impl PointVisitor<f64, ParseError> for PointWriter<'_> {
    type Output = ();
    fn coord(self, values: &[f64], _: Dimension) -> Result<(), ParseError> {
        for &v in values {
            value(self.buf, v);
        }
        Ok(())
    }
    fn empty(self) -> Result<(), ParseError> {
        empty_point(self.buf, self.dim);
        Ok(())
    }
}

impl CoordsVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    fn coords(&mut self, values: &[f64], dim: Dimension) -> Result<(), ParseError> {
        self.count += (values.len() / size(dim)) as u32;
        for &v in values {
            value(self.buf, v);
        }
        Ok(())
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}

impl PolygonVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    type Ring<'b>
        = PartWriter<'b>
    where
        Self: 'b;
    fn ring(&mut self) -> Result<PartWriter<'_>, ParseError> {
        let dim = self.dim;
        Ok(PartWriter::new(self.member(), None, dim))
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}

impl MultiPointVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    type Point<'b>
        = PointWriter<'b>
    where
        Self: 'b;
    fn point(&mut self) -> Result<PointWriter<'_>, ParseError> {
        let dim = self.dim;
        Ok(PointWriter::new(self.member(), dim))
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}

impl MultiLineStringVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    type LineString<'b>
        = PartWriter<'b>
    where
        Self: 'b;
    fn line_string(&mut self) -> Result<PartWriter<'_>, ParseError> {
        let dim = self.dim;
        Ok(PartWriter::new(self.member(), Some(LINE_STRING), dim))
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}

impl MultiPolygonVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    type Polygon<'b>
        = PartWriter<'b>
    where
        Self: 'b;
    fn polygon(&mut self) -> Result<PartWriter<'_>, ParseError> {
        let dim = self.dim;
        Ok(PartWriter::new(self.member(), Some(POLYGON), dim))
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}

impl GeometryCollectionVisitor<f64, ParseError> for PartWriter<'_> {
    type Output = ();
    type Geometry<'b>
        = GeometryWriter<'b>
    where
        Self: 'b;
    fn geometry(&mut self) -> Result<GeometryWriter<'_>, ParseError> {
        Ok(GeometryWriter { buf: self.member() })
    }
    fn end(self) -> Result<(), ParseError> {
        self.finish()
    }
}
