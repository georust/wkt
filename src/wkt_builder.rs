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

// Properties of `Wkt::from_str` and `validate` on generated input.
#[cfg(test)]
mod tests {
    use crate::visitor::validate;
    use crate::Wkt;
    use hegel::generators as gs;
    use hegel::TestCase;
    use std::str::FromStr;

    const DIMS: [&str; 8] = ["", " Z", " M", " ZM", "Z", "M", "ZM", " zm"];

    fn values_per_coord(dim: &str) -> usize {
        match dim.trim().to_ascii_uppercase().as_str() {
            "" => 2,
            "ZM" => 4,
            _ => 3,
        }
    }

    // A comma-separated list of 1 to `max` items.
    fn list(tc: &TestCase, max: usize, mut item: impl FnMut(&TestCase) -> String) -> String {
        let n = tc.draw(gs::integers::<usize>().min_value(1).max_value(max));
        (0..n).map(|_| item(tc)).collect::<Vec<_>>().join(", ")
    }

    fn coord(tc: &TestCase, n: usize) -> String {
        (0..n)
            .map(|_| {
                tc.draw(gs::sampled_from(vec![
                    "0",
                    "-1",
                    "+2",
                    "1.5",
                    "-.5",
                    "1e3",
                    "180",
                    "-90.123456789",
                    "4294967296",
                ]))
                .to_string()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    // A parenthesised coordinate sequence. The writer writes a polygon with an empty first
    // ring as `EMPTY`, so a polygon ring is not `EMPTY`.
    fn ring(tc: &TestCase, n: usize) -> String {
        format!("({})", list(tc, 4, |tc| coord(tc, n)))
    }

    // A parenthesised coordinate sequence, or EMPTY.
    fn sequence(tc: &TestCase, n: usize) -> String {
        if tc.draw(gs::weighted_booleans(0.2)) {
            "EMPTY".into()
        } else {
            format!("({})", list(tc, 4, |tc| coord(tc, n)))
        }
    }

    // A valid WKT geometry, nested at most `depth` more levels.
    fn geometry(tc: &TestCase, depth: u32) -> String {
        let names = [
            "POINT",
            "LINESTRING",
            "POLYGON",
            "MULTIPOINT",
            "MULTILINESTRING",
            "MULTIPOLYGON",
            "LINEARRING",
            "GEOMETRYCOLLECTION",
        ];
        let max = if depth == 0 { 6 } else { 7 };
        let kind = tc.draw(gs::integers::<usize>().min_value(0).max_value(max));
        let mut name = names[kind].to_string();
        if tc.draw(gs::booleans()) {
            name = name.to_ascii_lowercase();
        }
        let dim = if names[kind] == "LINEARRING" {
            ""
        } else {
            tc.draw(gs::sampled_from(DIMS.to_vec()))
        };
        if tc.draw(gs::weighted_booleans(0.15)) {
            return format!("{name}{dim} EMPTY");
        }
        let n = values_per_coord(dim);
        let body = match names[kind] {
            "POINT" => coord(tc, n),
            "LINESTRING" | "LINEARRING" => list(tc, 4, |tc| coord(tc, n)),
            "POLYGON" => list(tc, 3, |tc| ring(tc, n)),
            "MULTILINESTRING" => list(tc, 3, |tc| sequence(tc, n)),
            "MULTIPOINT" => list(tc, 4, |tc| {
                match tc.draw(gs::integers::<u8>().min_value(0).max_value(2)) {
                    0 => coord(tc, n),
                    1 => format!("({})", coord(tc, n)),
                    _ => "EMPTY".into(),
                }
            }),
            "MULTIPOLYGON" => list(tc, 3, |tc| {
                if tc.draw(gs::weighted_booleans(0.2)) {
                    "EMPTY".into()
                } else {
                    format!("({})", list(tc, 3, |tc| ring(tc, n)))
                }
            }),
            _ => list(tc, 3, |tc| geometry(tc, depth - 1)),
        };
        format!("{name}{dim}({body})")
    }

    const VOCABULARY: [&str; 14] = [
        "POINT",
        "MULTIPOINT",
        "POLYGON",
        "GEOMETRYCOLLECTION",
        "Z",
        "ZM",
        "EMPTY",
        "(",
        ")",
        ",",
        "1",
        "-2.5",
        "x",
        "\u{be}",
    ];

    // Deletes a range of bytes or inserts a token at a random position.
    fn mutate(tc: &TestCase, mut input: String) -> String {
        let at = tc.draw(gs::integers::<usize>().min_value(0).max_value(input.len()));
        if tc.draw(gs::booleans()) {
            let len = tc.draw(gs::integers::<usize>().min_value(1).max_value(4));
            input.replace_range(at..(at + len).min(input.len()), "");
        } else {
            input.insert_str(at, tc.draw(gs::sampled_from(VOCABULARY.to_vec())));
        }
        input
    }

    // Valid input parses, and the written WKT parses to the same value.
    #[hegel::test(test_cases = 3000)]
    fn valid_input_round_trips(tc: TestCase) {
        let input = geometry(&tc, 3);
        let wkt = Wkt::<f64>::from_str(&input).unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let written = wkt.to_string();
        assert_eq!(
            Wkt::from_str(&written),
            Ok(wkt),
            "input: {input:?}, written: {written:?}"
        );
    }

    // The builder and the validator give the same result for each input.
    fn assert_validate_agrees(input: &str) {
        assert_eq!(
            validate::<f64>(input),
            Wkt::<f64>::from_str(input).map(|_| ()),
            "input: {input:?}"
        );
    }

    #[hegel::test(test_cases = 3000)]
    fn validate_agrees_on_mutated_input(tc: TestCase) {
        let input = geometry(&tc, 3);
        assert_validate_agrees(&mutate(&tc, input));
    }

    #[hegel::test(test_cases = 3000)]
    fn validate_agrees_on_token_sequences(tc: TestCase) {
        let tokens = tc.draw(gs::vecs(gs::sampled_from(VOCABULARY.to_vec())).max_size(12));
        assert_validate_agrees(&tokens.join(tc.draw(gs::sampled_from(vec![" ", ""]))));
    }

    #[hegel::test(test_cases = 2000)]
    fn validate_agrees_on_any_text(tc: TestCase) {
        assert_validate_agrees(&tc.draw(gs::text()));
    }
}
