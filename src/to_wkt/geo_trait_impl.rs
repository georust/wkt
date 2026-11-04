use std::fmt;
use std::fmt::Write;

use geo_traits::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    TriangleTrait,
};

use crate::error::Error;
use crate::types::Coord;
use crate::WktNum;

/// The physical size of the coordinate dimension
///
/// This is used so that we don't have to call `.dim()` on **every** coordinate. We infer it once
/// from the `geo_traits::Dimensions` and then pass it to each coordinate.
#[derive(Clone, Copy)]
enum PhysicalCoordinateDimension {
    Two,
    Three,
    Four,
}

impl TryFrom<geo_traits::Dimensions> for PhysicalCoordinateDimension {
    type Error = Error;

    fn try_from(value: geo_traits::Dimensions) -> Result<Self, Self::Error> {
        match value.size() {
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(Error::UnknownDimension),
        }
    }
}

/// Whether a geometry is written on its own, after its type tag, or as a member of another geometry.
#[derive(Clone, Copy)]
enum Nesting {
    TopLevel,
    Member,
}

impl Nesting {
    fn empty_keyword(self) -> &'static str {
        match self {
            Self::TopLevel => " EMPTY",
            Self::Member => "EMPTY",
        }
    }
}

/// Write an object implementing [`PointTrait`] to a WKT string.
pub fn write_point<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    g: &impl PointTrait<T = T>,
) -> Result<(), Error> {
    let dim = g.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => f.write_str("POINT"),
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => f.write_str("POINT Z"),
        geo_traits::Dimensions::Xym => f.write_str("POINT M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("POINT ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    write_point_contents(f, g, dim.try_into()?, Nesting::TopLevel)
}

/// Write an object implementing [`LineStringTrait`] to a WKT string.
pub fn write_linestring<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    linestring: &impl LineStringTrait<T = T>,
) -> Result<(), Error> {
    let dim = linestring.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("LINESTRING")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("LINESTRING Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("LINESTRING M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("LINESTRING ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    write_linestring_contents(f, linestring, dim.try_into()?, Nesting::TopLevel)
}

/// Write an object implementing [`PolygonTrait`] to a WKT string.
pub fn write_polygon<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    polygon: &impl PolygonTrait<T = T>,
) -> Result<(), Error> {
    let dim = polygon.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => f.write_str("POLYGON"),
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("POLYGON Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("POLYGON M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("POLYGON ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    write_polygon_contents(f, polygon, dim.try_into()?, Nesting::TopLevel)
}

/// Write an object implementing [`MultiPointTrait`] to a WKT string.
pub fn write_multi_point<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    multipoint: &impl MultiPointTrait<T = T>,
) -> Result<(), Error> {
    let dim = multipoint.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("MULTIPOINT")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("MULTIPOINT Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("MULTIPOINT M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("MULTIPOINT ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let size = dim.try_into()?;

    let mut points = multipoint.points();

    // Note: This is largely copied from `write_coord_sequence`, because `multipoint.points()`
    // yields a sequence of Point, not Coord.
    if let Some(first_point) = points.next() {
        f.write_char('(')?;

        write_point_contents(f, &first_point, size, Nesting::Member)?;

        for point in points {
            f.write_char(',')?;
            write_point_contents(f, &point, size, Nesting::Member)?;
        }

        f.write_char(')')?;
    } else {
        f.write_str(" EMPTY")?;
    }

    Ok(())
}

/// Write a Point without its type tag, as `EMPTY` if it has no coordinate.
fn write_point_contents<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    point: &impl PointTrait<T = T>,
    size: PhysicalCoordinateDimension,
    nesting: Nesting,
) -> Result<(), Error> {
    if let Some(coord) = point.coord() {
        f.write_char('(')?;
        write_coord(f, &coord, size)?;
        f.write_char(')')?;
        Ok(())
    } else {
        Ok(f.write_str(nesting.empty_keyword())?)
    }
}

/// Write an object implementing [`MultiLineStringTrait`] to a WKT string.
pub fn write_multi_linestring<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    multilinestring: &impl MultiLineStringTrait<T = T>,
) -> Result<(), Error> {
    let dim = multilinestring.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("MULTILINESTRING")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("MULTILINESTRING Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("MULTILINESTRING M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("MULTILINESTRING ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let size = dim.try_into()?;
    let mut line_strings = multilinestring.line_strings();
    if let Some(first_linestring) = line_strings.next() {
        f.write_str("(")?;
        write_linestring_contents(f, &first_linestring, size, Nesting::Member)?;

        for linestring in line_strings {
            f.write_char(',')?;
            write_linestring_contents(f, &linestring, size, Nesting::Member)?;
        }

        f.write_char(')')?;
    } else {
        f.write_str(" EMPTY")?;
    };

    Ok(())
}

/// Write a LineString without its type tag, as `EMPTY` if it has no coordinates.
fn write_linestring_contents<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    linestring: &impl LineStringTrait<T = T>,
    size: PhysicalCoordinateDimension,
    nesting: Nesting,
) -> Result<(), Error> {
    if linestring.num_coords() == 0 {
        Ok(f.write_str(nesting.empty_keyword())?)
    } else {
        write_coord_sequence(f, linestring.coords(), size)
    }
}

/// Write an object implementing [`MultiPolygonTrait`] to a WKT string.
pub fn write_multi_polygon<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    multipolygon: &impl MultiPolygonTrait<T = T>,
) -> Result<(), Error> {
    let dim = multipolygon.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("MULTIPOLYGON")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("MULTIPOLYGON Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("MULTIPOLYGON M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("MULTIPOLYGON ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let size = dim.try_into()?;

    let mut polygons = multipolygon.polygons();

    if let Some(first_polygon) = polygons.next() {
        f.write_char('(')?;

        write_polygon_contents(f, &first_polygon, size, Nesting::Member)?;

        for polygon in polygons {
            f.write_char(',')?;
            write_polygon_contents(f, &polygon, size, Nesting::Member)?;
        }

        f.write_char(')')?;
    } else {
        f.write_str(" EMPTY")?;
    };

    Ok(())
}

/// Write a Polygon without its type tag, as `EMPTY` if it has no exterior coordinates.
fn write_polygon_contents<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    polygon: &impl PolygonTrait<T = T>,
    size: PhysicalCoordinateDimension,
    nesting: Nesting,
) -> Result<(), Error> {
    let Some(exterior) = polygon.exterior().filter(|e| e.num_coords() != 0) else {
        return Ok(f.write_str(nesting.empty_keyword())?);
    };

    f.write_char('(')?;
    write_coord_sequence(f, exterior.coords(), size)?;
    for interior in polygon.interiors() {
        f.write_char(',')?;
        write_linestring_contents(f, &interior, size, Nesting::Member)?;
    }
    Ok(f.write_char(')')?)
}

/// Write an object implementing [`GeometryTrait`] to a WKT string.
pub fn write_geometry<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    geometry: &impl GeometryTrait<T = T>,
) -> Result<(), Error> {
    match geometry.as_type() {
        geo_traits::GeometryType::Point(point) => write_point(f, point),
        geo_traits::GeometryType::LineString(linestring) => write_linestring(f, linestring),
        geo_traits::GeometryType::Polygon(polygon) => write_polygon(f, polygon),
        geo_traits::GeometryType::MultiPoint(multi_point) => write_multi_point(f, multi_point),
        geo_traits::GeometryType::MultiLineString(mls) => write_multi_linestring(f, mls),
        geo_traits::GeometryType::MultiPolygon(multi_polygon) => {
            write_multi_polygon(f, multi_polygon)
        }
        geo_traits::GeometryType::GeometryCollection(gc) => write_geometry_collection(f, gc),
        geo_traits::GeometryType::Rect(rect) => write_rect(f, rect),
        geo_traits::GeometryType::Triangle(triangle) => write_triangle(f, triangle),
        geo_traits::GeometryType::Line(line) => write_line(f, line),
    }
}

/// Write an object implementing [`GeometryCollectionTrait`] to a WKT string.
pub fn write_geometry_collection<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    gc: &impl GeometryCollectionTrait<T = T>,
) -> Result<(), Error> {
    let dim = gc.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("GEOMETRYCOLLECTION")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("GEOMETRYCOLLECTION Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("GEOMETRYCOLLECTION M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("GEOMETRYCOLLECTION ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let mut geometries = gc.geometries();

    if let Some(first_geometry) = geometries.next() {
        f.write_str("(")?;

        write_geometry(f, &first_geometry)?;
        for geom in geometries {
            f.write_char(',')?;
            write_geometry(f, &geom)?;
        }

        f.write_char(')')?;
    } else {
        f.write_str(" EMPTY")?;
    }
    Ok(())
}

/// Write an object implementing [`RectTrait`] to a WKT string.
///
/// The Rect will written as a Polygon with one exterior ring.
///
/// Note that only 2D `Rect`s are supported, because it's unclear how to map a higher-dimensional
/// Rect to a Polygon. For higher dimensional `Rect`, transform your data to a Polygon and use
/// [`write_polygon`].
pub fn write_rect<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    rect: &impl RectTrait<T = T>,
) -> Result<(), Error> {
    // Write prefix and error if not 2D
    match rect.dim() {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => f.write_str("POLYGON"),
        _ => return Err(Error::RectUnsupportedDimension),
    }?;

    let min_coord = rect.min();
    let max_coord = rect.max();

    // We need to construct the five points of the rect that make up the exterior Polygon
    let coords = [
        Coord {
            x: min_coord.x(),
            y: min_coord.y(),
            z: None,
            m: None,
        },
        Coord {
            x: min_coord.x(),
            y: max_coord.y(),
            z: None,
            m: None,
        },
        Coord {
            x: max_coord.x(),
            y: max_coord.y(),
            z: None,
            m: None,
        },
        Coord {
            x: max_coord.x(),
            y: min_coord.y(),
            z: None,
            m: None,
        },
        Coord {
            x: min_coord.x(),
            y: min_coord.y(),
            z: None,
            m: None,
        },
    ];

    f.write_str("(")?;
    write_coord_sequence(f, coords.iter(), PhysicalCoordinateDimension::Two)?;
    Ok(f.write_char(')')?)
}

/// Write an object implementing [`TriangleTrait`] to a WKT string.
///
/// The Triangle will written as a Polygon with one exterior ring.
pub fn write_triangle<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    triangle: &impl TriangleTrait<T = T>,
) -> Result<(), Error> {
    let dim = triangle.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => f.write_str("POLYGON"),
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("POLYGON Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("POLYGON M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("POLYGON ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let size = dim.try_into()?;
    f.write_str("(")?;

    let coords_iter = triangle
        .coords()
        .into_iter()
        .chain(std::iter::once(triangle.first()));
    write_coord_sequence(f, coords_iter, size)?;

    Ok(f.write_char(')')?)
}

/// Write an object implementing [`LineTrait`] to a WKT string.
///
/// The Line will written as a LineString with two coordinates.
pub fn write_line<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    line: &impl LineTrait<T = T>,
) -> Result<(), Error> {
    let dim = line.dim();
    // Write prefix
    match dim {
        geo_traits::Dimensions::Xy | geo_traits::Dimensions::Unknown(2) => {
            f.write_str("LINESTRING")
        }
        geo_traits::Dimensions::Xyz | geo_traits::Dimensions::Unknown(3) => {
            f.write_str("LINESTRING Z")
        }
        geo_traits::Dimensions::Xym => f.write_str("LINESTRING M"),
        geo_traits::Dimensions::Xyzm | geo_traits::Dimensions::Unknown(4) => {
            f.write_str("LINESTRING ZM")
        }
        geo_traits::Dimensions::Unknown(_) => return Err(Error::UnknownDimension),
    }?;
    let size = dim.try_into()?;
    write_coord_sequence(f, line.coords().into_iter(), size)
}

/// Write a single coordinate to the writer.
///
/// Will not include any start or end `()` characters.
fn write_coord<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    coord: &impl CoordTrait<T = T>,
    size: PhysicalCoordinateDimension,
) -> Result<(), std::fmt::Error> {
    match size {
        PhysicalCoordinateDimension::Two => write!(f, "{} {}", coord.x(), coord.y()),
        PhysicalCoordinateDimension::Three => {
            // Safety:
            // We've validated that there are three dimensions
            write!(f, "{} {} {}", coord.x(), coord.y(), unsafe {
                coord.nth_unchecked(2)
            })
        }
        PhysicalCoordinateDimension::Four => {
            write!(
                f,
                "{} {} {} {}",
                coord.x(),
                coord.y(),
                // Safety:
                // We've validated that there are four dimensions
                unsafe { coord.nth_unchecked(2) },
                // Safety:
                // We've validated that there are four dimensions
                unsafe { coord.nth_unchecked(3) }
            )
        }
    }
}

/// Includes the `()` characters to start and end this sequence.
///
/// E.g. it will write:
/// ```notest
/// (1 2, 3 4, 5 6)
/// ```
/// for a coordinate sequence with three coordinates.
fn write_coord_sequence<T: WktNum + fmt::Display>(
    f: &mut impl Write,
    mut coords: impl Iterator<Item = impl CoordTrait<T = T>>,
    size: PhysicalCoordinateDimension,
) -> Result<(), Error> {
    f.write_char('(')?;

    if let Some(first_coord) = coords.next() {
        write_coord(f, &first_coord, size)?;

        for coord in coords {
            f.write_char(',')?;
            write_coord(f, &coord, size)?;
        }
    }

    f.write_char(')')?;
    Ok(())
}
