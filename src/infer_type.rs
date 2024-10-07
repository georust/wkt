use crate::types::{Dimension, GeometryType};

const POINT: &str = "POINT";
const LINESTRING: &str = "LINESTRING";
const POLYGON: &str = "POLYGON";
const MULTIPOINT: &str = "MULTIPOINT";
const MULTILINESTRING: &str = "MULTILINESTRING";
const MULTIPOLYGON: &str = "MULTIPOLYGON";
const GEOMETRYCOLLECTION: &str = "GEOMETRYCOLLECTION";

/// Infer the geometry type and dimension from an input WKT string slice.
///
/// An `EMPTY` WKT object will return `None` in place of the dimension.
///
/// ```
/// use wkt::infer_type;
/// use wkt::types::{Dimension, GeometryType};
///
/// assert_eq!(
///     infer_type("POINT (10 20.1)").unwrap(),
///     (GeometryType::Point, Some(Dimension::XY))
/// );
///
/// assert_eq!(
///     infer_type("POINT EMPTY").unwrap(),
///     (GeometryType::Point, None)
/// );
/// ```
pub fn infer_type(input: &str) -> Result<(GeometryType, Option<Dimension>), String> {
    let input = input.trim_start();

    if let Some((prefix, _suffix)) = input.split_once("(") {
        let prefix = prefix.to_uppercase();

        let (geom_type, dim_str) = if let Some(dim_str) = prefix.strip_prefix(POINT) {
            (GeometryType::Point, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(LINESTRING) {
            (GeometryType::LineString, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(POLYGON) {
            (GeometryType::Polygon, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(MULTIPOINT) {
            (GeometryType::MultiPoint, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(MULTILINESTRING) {
            (GeometryType::MultiLineString, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(MULTIPOLYGON) {
            (GeometryType::MultiPolygon, dim_str)
        } else if let Some(dim_str) = prefix.strip_prefix(GEOMETRYCOLLECTION) {
            (GeometryType::GeometryCollection, dim_str)
        } else {
            return Err(format!("Unsupported WKT prefix {}", prefix));
        };

        let dim = if dim_str.contains("ZM") {
            Dimension::XYZM
        } else if dim_str.contains("Z") {
            Dimension::XYZ
        } else if dim_str.contains("M") {
            Dimension::XYM
        } else {
            Dimension::XY
        };

        Ok((geom_type, Some(dim)))
    } else {
        let input = input.to_uppercase();
        if !input.contains("EMPTY") {
            return Err("Invalid WKT; no '(' character and not EMPTY".to_string());
        }

        if input.starts_with(POINT) {
            Ok((GeometryType::Point, None))
        } else if input.starts_with(LINESTRING) {
            Ok((GeometryType::LineString, None))
        } else if input.starts_with(POLYGON) {
            Ok((GeometryType::Polygon, None))
        } else if input.starts_with(MULTIPOINT) {
            Ok((GeometryType::MultiPoint, None))
        } else if input.starts_with(MULTILINESTRING) {
            Ok((GeometryType::MultiLineString, None))
        } else if input.starts_with(MULTIPOLYGON) {
            Ok((GeometryType::MultiPolygon, None))
        } else if input.starts_with(GEOMETRYCOLLECTION) {
            Ok((GeometryType::GeometryCollection, None))
        } else {
            return Err(format!("Unsupported WKT prefix {}", input));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_points() {
        assert_eq!(
            infer_type("POINT (10 20.1)").unwrap(),
            (GeometryType::Point, Some(Dimension::XY))
        );
        assert_eq!(
            infer_type("POINT Z (10 20.1 5)").unwrap(),
            (GeometryType::Point, Some(Dimension::XYZ))
        );
        assert_eq!(
            infer_type("POINT M (10 20.1 80)").unwrap(),
            (GeometryType::Point, Some(Dimension::XYM))
        );
        assert_eq!(
            infer_type("POINT ZM (10 20.1 5 80)").unwrap(),
            (GeometryType::Point, Some(Dimension::XYZM))
        );
    }

    #[test]
    fn lowercase_point() {
        assert_eq!(
            infer_type("point EMPTY").unwrap(),
            (GeometryType::Point, None)
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(
            infer_type("POINT EMPTY").unwrap(),
            (GeometryType::Point, None)
        );
        assert_eq!(
            infer_type("MULTIPOLYGON EMPTY").unwrap(),
            (GeometryType::MultiPolygon, None)
        );
    }
}
