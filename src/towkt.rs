use crate::types::{
    Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};
use crate::Geometry;
use crate::{Wkt, WktFloat};

use geo_types::CoordFloat;

fn g_point_to_w_coord<T>(g_point: geo_types::Coordinate<T>) -> Coord<T>
where
    T: CoordFloat,
{
    Coord {
        x: g_point.x,
        y: g_point.y,
        z: None,
        m: None,
    }
}

fn g_point_to_w_point<T>(g_point: geo_types::Point<T>) -> Point<T>
where
    T: CoordFloat,
{
    let coord = g_point_to_w_coord(g_point.0);
    Point(Some(coord))
}

fn g_points_to_w_coords<T>(g_points: Vec<geo_types::Coordinate<T>>) -> Vec<Coord<T>>
where
    T: CoordFloat,
{
    g_points.into_iter().map(g_point_to_w_coord).collect()
}

fn g_points_to_w_points<T>(g_points: Vec<geo_types::Point<T>>) -> Vec<Point<T>>
where
    T: CoordFloat,
{
    g_points
        .into_iter()
        .map(|p| p.0)
        .map(g_point_to_w_coord)
        .map(|c| Point(Some(c)))
        .collect()
}

fn g_line_to_w_linestring<T>(g_line: geo_types::Line<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    g_points_to_w_linestring(vec![g_line.start, g_line.end]).as_item()
}

fn g_linestring_to_w_linestring<T>(g_linestring: geo_types::LineString<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let geo_types::LineString(g_points) = g_linestring;
    g_points_to_w_linestring(g_points).as_item()
}

fn g_points_to_w_linestring<T>(g_coords: Vec<geo_types::Coordinate<T>>) -> LineString<T>
where
    T: CoordFloat,
{
    let w_coords = g_points_to_w_coords(g_coords);
    LineString(w_coords)
}

fn g_lines_to_w_lines<T>(g_lines: Vec<geo_types::LineString<T>>) -> Vec<LineString<T>>
where
    T: CoordFloat,
{
    g_lines
        .into_iter()
        .map(|g_line| {
            let geo_types::LineString(g_points) = g_line;

            g_points_to_w_linestring(g_points)
        })
        .collect()
}

fn g_triangle_to_w_polygon<T>(g_triangle: geo_types::Triangle<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let polygon = g_triangle.to_polygon();
    g_polygon_to_w_polygon(polygon).as_item()
}

fn g_rect_to_w_polygon<T>(g_rect: geo_types::Rect<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let polygon = g_rect.to_polygon();
    g_polygon_to_w_polygon(polygon).as_item()
}

fn g_polygon_to_w_polygon<T>(g_polygon: geo_types::Polygon<T>) -> Polygon<T>
where
    T: CoordFloat,
{
    let (outer_line, inner_lines) = g_polygon.into_inner();
    let mut poly_lines = Vec::with_capacity(inner_lines.len() + 1);

    // Outer
    let geo_types::LineString(outer_points) = outer_line;
    if !outer_points.is_empty() {
        poly_lines.push(g_points_to_w_linestring(outer_points));
    }

    // Inner
    let inner = g_lines_to_w_lines(inner_lines);
    poly_lines.extend(inner.into_iter());

    Polygon(poly_lines)
}

fn g_polygon_to_w_polygon_geom<T>(g_polygon: geo_types::Polygon<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    g_polygon_to_w_polygon(g_polygon).as_item()
}

fn g_mpoint_to_w_mpoint<T>(g_mpoint: geo_types::MultiPoint<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let geo_types::MultiPoint(g_points) = g_mpoint;
    let w_points = g_points_to_w_points(g_points);
    MultiPoint(w_points).as_item()
}

fn g_mline_to_w_mline<T>(g_mline: geo_types::MultiLineString<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let geo_types::MultiLineString(g_lines) = g_mline;
    let w_lines = g_lines_to_w_lines(g_lines);
    MultiLineString(w_lines).as_item()
}

fn g_polygons_to_w_polygons<T>(g_polygons: Vec<geo_types::Polygon<T>>) -> Vec<Polygon<T>>
where
    T: CoordFloat,
{
    g_polygons.into_iter().map(g_polygon_to_w_polygon).collect()
}

fn g_mpolygon_to_w_mpolygon<T>(g_mpolygon: geo_types::MultiPolygon<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let geo_types::MultiPolygon(g_polygons) = g_mpolygon;
    let w_polygons = g_polygons_to_w_polygons(g_polygons);
    MultiPolygon(w_polygons).as_item()
}

fn g_geocol_to_w_geocol<T>(g_geocol: geo_types::GeometryCollection<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    let geo_types::GeometryCollection(g_geoms) = g_geocol;

    GeometryCollection(g_geoms.into_iter().map(g_geom_to_w_geom).collect()).as_item()
}

fn g_geom_to_w_geom<T>(g_geom: geo_types::Geometry<T>) -> Geometry<T>
where
    T: CoordFloat,
{
    match g_geom {
        geo_types::Geometry::Point(g_point) => g_point_to_w_point(g_point).as_item(),

        geo_types::Geometry::Line(g_line) => g_line_to_w_linestring(g_line),

        geo_types::Geometry::LineString(g_line) => g_linestring_to_w_linestring(g_line),

        geo_types::Geometry::Triangle(g_triangle) => g_triangle_to_w_polygon(g_triangle),

        geo_types::Geometry::Rect(g_rect) => g_rect_to_w_polygon(g_rect),

        geo_types::Geometry::Polygon(g_polygon) => g_polygon_to_w_polygon(g_polygon).as_item(),

        geo_types::Geometry::MultiPoint(g_mpoint) => g_mpoint_to_w_mpoint(g_mpoint),

        geo_types::Geometry::MultiLineString(g_mline) => g_mline_to_w_mline(g_mline),

        geo_types::Geometry::MultiPolygon(g_mpolygon) => g_mpolygon_to_w_mpolygon(g_mpolygon),

        geo_types::Geometry::GeometryCollection(g_geocol) => g_geocol_to_w_geocol(g_geocol),
    }
}

impl<T> From<geo_types::Point<T>> for Wkt<T>
where
    T: WktFloat,
{
    fn from(p: geo_types::Point<T>) -> Wkt<T> {
        Wkt {
            item: g_point_to_w_point(p).as_item(),
        }
    }
}

/// This macro allows easy implementation of From<SomeType> for Wkt
macro_rules! impl_from_for_wkt {
    ($from:ty, $func:ident) => {
        impl<T> From<$from> for Wkt<T>
        where
            T: WktFloat,
        {
            fn from(t: $from) -> Wkt<T> {
                Wkt { item: $func(t) }
            }
        }
    };
}

impl_from_for_wkt!(geo_types::Geometry<T>, g_geom_to_w_geom);
impl_from_for_wkt!(geo_types::Line<T>, g_line_to_w_linestring);
impl_from_for_wkt!(geo_types::LineString<T>, g_linestring_to_w_linestring);
impl_from_for_wkt!(geo_types::Triangle<T>, g_triangle_to_w_polygon);
impl_from_for_wkt!(geo_types::Rect<T>, g_rect_to_w_polygon);
impl_from_for_wkt!(geo_types::MultiPoint<T>, g_mpoint_to_w_mpoint);
impl_from_for_wkt!(geo_types::MultiLineString<T>, g_mline_to_w_mline);
impl_from_for_wkt!(geo_types::MultiPolygon<T>, g_mpolygon_to_w_mpolygon);
impl_from_for_wkt!(geo_types::GeometryCollection<T>, g_geocol_to_w_geocol);
impl_from_for_wkt!(geo_types::Polygon<T>, g_polygon_to_w_polygon_geom);

#[cfg(test)]
mod tests {
    use geo_types::Coordinate;

    use super::*;

    #[test]
    fn individual_geotypes_can_be_converted_to_wkt() {
        let c = Coordinate {
            x: 10.0_f64,
            y: 20.,
        };

        let _: Wkt<f64> = geo_types::Point(c).into();
        let _: Wkt<f64> = geo_types::Line { start: c, end: c }.into();
        let _: Wkt<f64> = geo_types::LineString(vec![]).into();
        let _: Wkt<f64> = geo_types::Triangle(c, c, c).into();
        let _: Wkt<f64> = geo_types::Rect::new(c, c).into();
        let _: Wkt<f64> = geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]).into();
        let _: Wkt<f64> = geo_types::MultiPoint(vec![]).into();
        let _: Wkt<f64> = geo_types::MultiLineString(vec![]).into();
        let _: Wkt<f64> = geo_types::MultiPolygon(vec![]).into();
        let _: Wkt<f64> = geo_types::GeometryCollection(vec![]).into();
    }

    #[test]
    fn geptypes_geometry_can_be_converted_to_wkt() {
        let c = Coordinate {
            x: 10.0_f64,
            y: 20.,
        };

        let _: Wkt<f64> = geo_types::Geometry::Point(geo_types::Point(c)).into();
        let _: Wkt<f64> = geo_types::Geometry::Line(geo_types::Line { start: c, end: c }).into();
        let _: Wkt<f64> = geo_types::Geometry::LineString(geo_types::LineString(vec![])).into();
        let _: Wkt<f64> = geo_types::Geometry::Triangle(geo_types::Triangle(c, c, c)).into();
        let _: Wkt<f64> = geo_types::Geometry::Rect(geo_types::Rect::new(c, c)).into();
        let _: Wkt<f64> = geo_types::Geometry::Polygon(geo_types::Polygon::new(
            geo_types::LineString(vec![]),
            vec![],
        ))
        .into();
        let _: Wkt<f64> = geo_types::Geometry::MultiPoint(geo_types::MultiPoint(vec![])).into();
        let _: Wkt<f64> =
            geo_types::Geometry::MultiLineString(geo_types::MultiLineString(vec![])).into();
        let _: Wkt<f64> = geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(vec![])).into();
        let _: Wkt<f64> =
            geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(vec![])).into();
    }
}
