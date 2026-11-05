//! Visitors that build a [`Wkt`] value.
//!
//! Each builder owns the value that it builds, and gives the value to its destination when it
//! ends. The destination of the top-level geometry returns the value as the output of the visit.
//! The destination of a member pushes the value into its parent.

use std::marker::PhantomData;

use crate::error::ParseError;
use crate::types::{
    Coord, Dimension, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};
use crate::visitor::{
    stride, CoordsVisitor, GeometryCollectionVisitor, GeometryVisitor, MultiLineStringVisitor,
    MultiPointVisitor, MultiPolygonVisitor, PointVisitor, PolygonVisitor,
};
use crate::{Wkt, WktNum};

/// Receives the value that a builder builds.
pub(crate) trait Dest<X> {
    type Output;

    fn put(self, value: X) -> Self::Output;
}

/// The destination of the top-level geometry.
pub(crate) struct Top<T>(PhantomData<T>);

impl<T: WktNum> Top<T> {
    pub(crate) fn new() -> Self {
        Top(PhantomData)
    }
}

impl<T: WktNum, X: Into<Wkt<T>>> Dest<X> for Top<T> {
    type Output = Wkt<T>;

    fn put(self, value: X) -> Wkt<T> {
        value.into()
    }
}

/// The destination of a member of a geometry collection.
pub(crate) struct CollectionMember<'a, T: WktNum>(&'a mut Vec<Wkt<T>>);

impl<T: WktNum, X: Into<Wkt<T>>> Dest<X> for CollectionMember<'_, T> {
    type Output = ();

    fn put(self, value: X) {
        self.0.push(value.into());
    }
}

/// The destination of a member of a polygon or a multi-geometry.
impl<X> Dest<X> for &mut Vec<X> {
    type Output = ();

    fn put(self, value: X) {
        self.push(value);
    }
}

fn to_coord<T: WktNum>(values: &[T], dim: Dimension) -> Coord<T> {
    let (z, m) = match dim {
        Dimension::XY => (None, None),
        Dimension::XYZ => (Some(values[2]), None),
        Dimension::XYM => (None, Some(values[2])),
        Dimension::XYZM => (Some(values[2]), Some(values[3])),
    };
    Coord {
        x: values[0],
        y: values[1],
        z,
        m,
    }
}

/// Builds a geometry of any type, and gives it to `D`.
pub(crate) struct GeometryBuilder<D>(pub(crate) D);

impl<T, D, O> GeometryVisitor<T> for GeometryBuilder<D>
where
    T: WktNum,
    D: Dest<Point<T>, Output = O>
        + Dest<LineString<T>, Output = O>
        + Dest<Polygon<T>, Output = O>
        + Dest<MultiPoint<T>, Output = O>
        + Dest<MultiLineString<T>, Output = O>
        + Dest<MultiPolygon<T>, Output = O>
        + Dest<GeometryCollection<T>, Output = O>,
{
    type Error = ParseError;
    type Output = O;
    type Point = Builder<Point<T>, D>;
    type LineString = Builder<LineString<T>, D>;
    type Polygon = Builder<Polygon<T>, D>;
    type MultiPoint = Builder<MultiPoint<T>, D>;
    type MultiLineString = Builder<MultiLineString<T>, D>;
    type MultiPolygon = Builder<MultiPolygon<T>, D>;
    type GeometryCollection = Builder<GeometryCollection<T>, D>;

    fn point(self, dim: Dimension) -> Result<Self::Point, ParseError> {
        Ok(Builder::new(Point::empty(dim), self.0))
    }
    fn line_string(self, dim: Dimension) -> Result<Self::LineString, ParseError> {
        Ok(Builder::new(LineString::empty(dim), self.0))
    }
    fn polygon(self, dim: Dimension) -> Result<Self::Polygon, ParseError> {
        Ok(Builder::new(Polygon::empty(dim), self.0))
    }
    fn multi_point(self, dim: Dimension) -> Result<Self::MultiPoint, ParseError> {
        Ok(Builder::new(MultiPoint::empty(dim), self.0))
    }
    fn multi_line_string(self, dim: Dimension) -> Result<Self::MultiLineString, ParseError> {
        Ok(Builder::new(MultiLineString::empty(dim), self.0))
    }
    fn multi_polygon(self, dim: Dimension) -> Result<Self::MultiPolygon, ParseError> {
        Ok(Builder::new(MultiPolygon::empty(dim), self.0))
    }
    fn geometry_collection(self, dim: Dimension) -> Result<Self::GeometryCollection, ParseError> {
        Ok(Builder::new(GeometryCollection::empty(dim), self.0))
    }
}

/// Builds a value of type `X`, and gives it to `D` when it ends.
pub(crate) struct Builder<X, D> {
    value: X,
    dest: D,
}

impl<X, D: Dest<X>> Builder<X, D> {
    fn new(value: X, dest: D) -> Self {
        Builder { value, dest }
    }

    fn finish(self) -> Result<D::Output, ParseError> {
        Ok(self.dest.put(self.value))
    }
}

impl<T: WktNum, D: Dest<Point<T>>> PointVisitor<T, ParseError> for Builder<Point<T>, D> {
    type Output = D::Output;

    fn coord(mut self, values: &[T], dim: Dimension) -> Result<D::Output, ParseError> {
        self.value.coord = Some(to_coord(values, dim));
        self.finish()
    }

    fn empty(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<LineString<T>>> CoordsVisitor<T, ParseError> for Builder<LineString<T>, D> {
    type Output = D::Output;

    fn coords(&mut self, values: &[T], dim: Dimension) -> Result<(), ParseError> {
        self.value.coords.extend(
            values
                .chunks_exact(stride(dim))
                .map(|values| to_coord(values, dim)),
        );
        Ok(())
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<Polygon<T>>> PolygonVisitor<T, ParseError> for Builder<Polygon<T>, D> {
    type Output = D::Output;
    type Ring<'a>
        = Builder<LineString<T>, &'a mut Vec<LineString<T>>>
    where
        Self: 'a;

    fn ring(&mut self) -> Result<Self::Ring<'_>, ParseError> {
        let ring = LineString::empty(self.value.dim);
        Ok(Builder::new(ring, &mut self.value.rings))
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<MultiPoint<T>>> MultiPointVisitor<T, ParseError>
    for Builder<MultiPoint<T>, D>
{
    type Output = D::Output;
    type Point<'a>
        = Builder<Point<T>, &'a mut Vec<Point<T>>>
    where
        Self: 'a;

    fn point(&mut self) -> Result<Self::Point<'_>, ParseError> {
        let point = Point::empty(self.value.dim);
        Ok(Builder::new(point, &mut self.value.points))
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<MultiLineString<T>>> MultiLineStringVisitor<T, ParseError>
    for Builder<MultiLineString<T>, D>
{
    type Output = D::Output;
    type LineString<'a>
        = Builder<LineString<T>, &'a mut Vec<LineString<T>>>
    where
        Self: 'a;

    fn line_string(&mut self) -> Result<Self::LineString<'_>, ParseError> {
        let line_string = LineString::empty(self.value.dim);
        Ok(Builder::new(line_string, &mut self.value.line_strings))
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<MultiPolygon<T>>> MultiPolygonVisitor<T, ParseError>
    for Builder<MultiPolygon<T>, D>
{
    type Output = D::Output;
    type Polygon<'a>
        = Builder<Polygon<T>, &'a mut Vec<Polygon<T>>>
    where
        Self: 'a;

    fn polygon(&mut self) -> Result<Self::Polygon<'_>, ParseError> {
        let polygon = Polygon::empty(self.value.dim);
        Ok(Builder::new(polygon, &mut self.value.polygons))
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}

impl<T: WktNum, D: Dest<GeometryCollection<T>>> GeometryCollectionVisitor<T, ParseError>
    for Builder<GeometryCollection<T>, D>
{
    type Output = D::Output;
    type Geometry<'a>
        = GeometryBuilder<CollectionMember<'a, T>>
    where
        Self: 'a;

    fn geometry(&mut self) -> Result<Self::Geometry<'_>, ParseError> {
        Ok(GeometryBuilder(CollectionMember(&mut self.value.geoms)))
    }

    fn end(self) -> Result<D::Output, ParseError> {
        self.finish()
    }
}
