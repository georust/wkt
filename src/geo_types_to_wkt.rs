use geo_types::CoordNum;

use crate::types::{
    Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};
use crate::{Geometry, ToWkt, Wkt};

/// # Examples
/// ```
/// use geo_types::{point, Geometry};
/// use wkt::ToWkt;
///
/// let geometry: Geometry<f64> = Geometry::Point(point!(x: 1., y: 2.));
///
/// assert_eq!(geometry.wkt_string(), "POINT(1 2)");
/// ```
impl<T> ToWkt<T> for geo_types::Geometry<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        match self {
            geo_types::Geometry::Point(g) => g.to_wkt(),
            geo_types::Geometry::Line(g) => g.to_wkt(),
            geo_types::Geometry::LineString(g) => g.to_wkt(),
            geo_types::Geometry::Polygon(g) => g.to_wkt(),
            geo_types::Geometry::MultiPoint(g) => g.to_wkt(),
            geo_types::Geometry::MultiLineString(g) => g.to_wkt(),
            geo_types::Geometry::MultiPolygon(g) => g.to_wkt(),
            geo_types::Geometry::GeometryCollection(g) => g.to_wkt(),
            geo_types::Geometry::Rect(g) => g.to_wkt(),
            geo_types::Geometry::Triangle(g) => g.to_wkt(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{point, Point};
/// use wkt::ToWkt;
///
/// let point: Point<f64> = point!(x: 1., y: 2.);
///
/// assert_eq!(point.wkt_string(), "POINT(1 2)");
/// ```
impl<T> ToWkt<T> for geo_types::Point<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_point_to_w_point(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{coord, Line};
/// use wkt::ToWkt;
///
/// let line = Line::<f64>::new(coord!(x: 1., y: 2.), coord!(x: 3., y: 4.));
///
/// assert_eq!(line.wkt_string(), "LINESTRING(1 2,3 4)");
/// ```
impl<T> ToWkt<T> for geo_types::Line<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_line_to_w_linestring(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{line_string, LineString};
/// use wkt::ToWkt;
///
/// let line_string: LineString<f64> = line_string![(x: 1., y: 2.), (x: 3., y: 4.), (x: 5., y: 6.)];
///
/// assert_eq!(line_string.wkt_string(), "LINESTRING(1 2,3 4,5 6)");
/// ```
impl<T> ToWkt<T> for geo_types::LineString<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_linestring_to_w_linestring(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{polygon, Polygon};
/// use wkt::ToWkt;
///
/// let polygon: Polygon<f64> = polygon![(x: 0., y: 0.), (x: 4., y: 0.), (x: 2., y: 4.), (x: 0., y: 0.)];
///
/// assert_eq!(polygon.wkt_string(), "POLYGON((0 0,4 0,2 4,0 0))");
/// ```
impl<T> ToWkt<T> for geo_types::Polygon<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_polygon_to_w_polygon(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{point, MultiPoint};
/// use wkt::ToWkt;
///
/// let multi_point: MultiPoint<f64> = MultiPoint::new(vec![point!(x: 0., y: 0.), point!(x: 4., y: 0.), point!(x: 2., y: 4.)]);
///
/// assert_eq!(multi_point.wkt_string(), "MULTIPOINT((0 0),(4 0),(2 4))");
/// ```
impl<T> ToWkt<T> for geo_types::MultiPoint<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_mpoint_to_w_mpoint(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{line_string, LineString, MultiLineString};
/// use wkt::ToWkt;
///
/// let line_string_1: LineString<f64> = line_string![(x: 1., y: 2.), (x: 3., y: 4.), (x: 5., y: 6.)];
/// let line_string_2: LineString<f64> = line_string![(x: 7., y: 8.), (x: 9., y: 0.)];
/// let multi_line_string: MultiLineString<f64> = MultiLineString::new(vec![line_string_1, line_string_2]);
///
/// assert_eq!(multi_line_string.wkt_string(), "MULTILINESTRING((1 2,3 4,5 6),(7 8,9 0))");
/// ```
impl<T> ToWkt<T> for geo_types::MultiLineString<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_mline_to_w_mline(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{polygon, Polygon, MultiPolygon};
/// use wkt::ToWkt;
///
/// // triangle
/// let polygon_1: Polygon<f64> = polygon![(x: 0., y: 0.), (x: 4., y: 0.), (x: 2., y: 4.), (x: 0., y: 0.)];
/// // square
/// let polygon_2: Polygon<f64> = polygon![(x: 4., y: 4.), (x: 8., y: 4.), (x: 8., y: 8.), (x: 4., y: 8.), (x: 4., y: 4.)];
/// let multi_polygon: MultiPolygon<f64> = MultiPolygon::new(vec![polygon_1, polygon_2]);
///
/// assert_eq!(multi_polygon.wkt_string(), "MULTIPOLYGON(((0 0,4 0,2 4,0 0)),((4 4,8 4,8 8,4 8,4 4)))");
/// ```
impl<T> ToWkt<T> for geo_types::MultiPolygon<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_mpolygon_to_w_mpolygon(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{line_string, LineString, polygon, Polygon, GeometryCollection};
/// use wkt::ToWkt;
///
/// let polygon: Polygon<f64> = polygon![(x: 0., y: 0.), (x: 4., y: 0.), (x: 2., y: 4.), (x: 0., y: 0.)];
/// let line_string: LineString<f64> = line_string![(x: 1., y: 2.), (x: 3., y: 4.), (x: 5., y: 6.)];
/// let geometry_collection: GeometryCollection<f64> = GeometryCollection::new_from(vec![polygon.into(), line_string.into()]);
///
/// assert_eq!(geometry_collection.wkt_string(), "GEOMETRYCOLLECTION(POLYGON((0 0,4 0,2 4,0 0)),LINESTRING(1 2,3 4,5 6))");
/// ```
impl<T> ToWkt<T> for geo_types::GeometryCollection<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_geocol_to_w_geocol(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{coord, Rect};
/// use wkt::ToWkt;
///
/// let rect: Rect<f64> = Rect::new(coord!(x: 4., y: 4.), coord!(x: 8., y: 8.));
///
/// assert_eq!(rect.wkt_string(), "POLYGON((4 4,4 8,8 8,8 4,4 4))");
/// ```
impl<T> ToWkt<T> for geo_types::Rect<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_rect_to_w_polygon(self).as_item(),
        }
    }
}

/// # Examples
/// ```
/// use geo_types::{coord, Triangle};
/// use wkt::ToWkt;
///
/// let triangle: Triangle<f64> = Triangle::new(coord!(x: 0., y: 0.), coord!(x: 4., y: 0.), coord!(x: 2., y: 4.));
///
/// assert_eq!(triangle.wkt_string(), "POLYGON((0 0,4 0,2 4,0 0))");
/// ```
impl<T> ToWkt<T> for geo_types::Triangle<T>
where
    T: CoordNum + std::fmt::Display,
{
    fn to_wkt(&self) -> Wkt<T> {
        Wkt {
            item: g_triangle_to_w_polygon(self).as_item(),
        }
    }
}

fn g_point_to_w_coord<T>(g_point: &geo_types::Coordinate<T>) -> Coord<T>
where
    T: CoordNum,
{
    Coord {
        x: g_point.x,
        y: g_point.y,
        z: None,
        m: None,
    }
}

fn g_point_to_w_point<T>(g_point: &geo_types::Point<T>) -> Point<T>
where
    T: CoordNum,
{
    let coord = g_point_to_w_coord(&g_point.0);
    Point(Some(coord))
}

fn g_points_to_w_coords<T>(g_points: &[geo_types::Coordinate<T>]) -> Vec<Coord<T>>
where
    T: CoordNum,
{
    g_points.iter().map(g_point_to_w_coord).collect()
}

fn g_points_to_w_points<T>(g_points: &[geo_types::Point<T>]) -> Vec<Point<T>>
where
    T: CoordNum,
{
    g_points
        .iter()
        .map(|p| &p.0)
        .map(g_point_to_w_coord)
        .map(|c| Point(Some(c)))
        .collect()
}

fn g_line_to_w_linestring<T>(g_line: &geo_types::Line<T>) -> LineString<T>
where
    T: CoordNum,
{
    g_points_to_w_linestring(&[g_line.start, g_line.end])
}

fn g_linestring_to_w_linestring<T>(g_linestring: &geo_types::LineString<T>) -> LineString<T>
where
    T: CoordNum,
{
    let &geo_types::LineString(ref g_points) = g_linestring;
    g_points_to_w_linestring(g_points)
}

fn g_points_to_w_linestring<T>(g_coords: &[geo_types::Coordinate<T>]) -> LineString<T>
where
    T: CoordNum,
{
    let w_coords = g_points_to_w_coords(g_coords);
    LineString(w_coords)
}

fn g_lines_to_w_lines<T>(g_lines: &[geo_types::LineString<T>]) -> Vec<LineString<T>>
where
    T: CoordNum,
{
    let mut w_lines = vec![];
    for g_line in g_lines {
        let &geo_types::LineString(ref g_points) = g_line;
        w_lines.push(g_points_to_w_linestring(g_points));
    }
    w_lines
}

fn g_triangle_to_w_polygon<T>(g_triangle: &geo_types::Triangle<T>) -> Polygon<T>
where
    T: CoordNum,
{
    let polygon = g_triangle.to_polygon();
    g_polygon_to_w_polygon(&polygon)
}

fn g_rect_to_w_polygon<T>(g_rect: &geo_types::Rect<T>) -> Polygon<T>
where
    T: CoordNum,
{
    let polygon = g_rect.to_polygon();
    g_polygon_to_w_polygon(&polygon)
}

fn g_polygon_to_w_polygon<T>(g_polygon: &geo_types::Polygon<T>) -> Polygon<T>
where
    T: CoordNum,
{
    let outer_line = g_polygon.exterior();
    let inner_lines = g_polygon.interiors();
    let mut poly_lines = vec![];

    // Outer
    let &geo_types::LineString(ref outer_points) = outer_line;
    if !outer_points.is_empty() {
        poly_lines.push(g_points_to_w_linestring(outer_points));
    }

    // Inner
    let inner = g_lines_to_w_lines(inner_lines);
    poly_lines.extend(inner.into_iter());

    Polygon(poly_lines)
}

fn g_mpoint_to_w_mpoint<T>(g_mpoint: &geo_types::MultiPoint<T>) -> MultiPoint<T>
where
    T: CoordNum,
{
    let &geo_types::MultiPoint(ref g_points) = g_mpoint;
    let w_points = g_points_to_w_points(g_points);
    MultiPoint(w_points)
}

fn g_mline_to_w_mline<T>(g_mline: &geo_types::MultiLineString<T>) -> MultiLineString<T>
where
    T: CoordNum,
{
    let &geo_types::MultiLineString(ref g_lines) = g_mline;
    let w_lines = g_lines_to_w_lines(g_lines);
    MultiLineString(w_lines)
}

fn g_polygons_to_w_polygons<T>(g_polygons: &[geo_types::Polygon<T>]) -> Vec<Polygon<T>>
where
    T: CoordNum,
{
    let mut w_polygons = vec![];
    for g_polygon in g_polygons {
        w_polygons.push(g_polygon_to_w_polygon(g_polygon));
    }
    w_polygons
}

fn g_mpolygon_to_w_mpolygon<T>(g_mpolygon: &geo_types::MultiPolygon<T>) -> MultiPolygon<T>
where
    T: CoordNum,
{
    let &geo_types::MultiPolygon(ref g_polygons) = g_mpolygon;
    let w_polygons = g_polygons_to_w_polygons(g_polygons);
    MultiPolygon(w_polygons)
}

fn g_geocol_to_w_geocol<T>(g_geocol: &geo_types::GeometryCollection<T>) -> GeometryCollection<T>
where
    T: CoordNum,
{
    let &geo_types::GeometryCollection(ref g_geoms) = g_geocol;
    let mut w_geoms = vec![];
    for g_geom in g_geoms {
        let w_geom = g_geom_to_w_geom(g_geom);
        w_geoms.push(w_geom);
    }
    GeometryCollection(w_geoms)
}

fn g_geom_to_w_geom<T>(g_geom: &geo_types::Geometry<T>) -> Geometry<T>
where
    T: CoordNum,
{
    match *g_geom {
        geo_types::Geometry::Point(ref g_point) => g_point_to_w_point(g_point).as_item(),

        geo_types::Geometry::Line(ref g_line) => g_line_to_w_linestring(g_line).as_item(),

        geo_types::Geometry::LineString(ref g_line) => {
            g_linestring_to_w_linestring(g_line).as_item()
        }

        geo_types::Geometry::Triangle(ref g_triangle) => {
            g_triangle_to_w_polygon(g_triangle).as_item()
        }

        geo_types::Geometry::Rect(ref g_rect) => g_rect_to_w_polygon(g_rect).as_item(),

        geo_types::Geometry::Polygon(ref g_polygon) => g_polygon_to_w_polygon(g_polygon).as_item(),

        geo_types::Geometry::MultiPoint(ref g_mpoint) => g_mpoint_to_w_mpoint(g_mpoint).as_item(),

        geo_types::Geometry::MultiLineString(ref g_mline) => g_mline_to_w_mline(g_mline).as_item(),

        geo_types::Geometry::MultiPolygon(ref g_mpolygon) => {
            g_mpolygon_to_w_mpolygon(g_mpolygon).as_item()
        }

        geo_types::Geometry::GeometryCollection(ref g_geocol) => {
            g_geocol_to_w_geocol(g_geocol).as_item()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ToWkt;

    #[test]
    fn integer_geom() {
        let point = geo_types::Point::new(1i32, 2i32);
        assert_eq!("POINT(1 2)", &point.wkt_string());
    }

    #[test]
    fn float_geom() {
        let point = geo_types::Point::new(1f32, 2f32);
        assert_eq!("POINT(1 2)", &point.wkt_string());

        let point = geo_types::Point::new(1.1f32, 2.9f32);
        assert_eq!("POINT(1.1 2.9)", &point.wkt_string());
    }
}
