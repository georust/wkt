//! This module deserialises to WKT using [`serde`].
//!
//! You can deserialise to [`geo_types`] or any other implementor of [`TryFromWkt`], using
//! [`deserialize_wkt`]. Or you can store this crates internal primitives [`Wkt`]
//! or [`Geometry`] in your struct fields.

use crate::{Geometry, TryFromWkt, Wkt, WktNum};
use serde::de::{Deserializer, Error, Visitor};
use std::{
    default::Default,
    fmt::{self, Debug},
    marker::PhantomData,
    str::FromStr,
};

#[cfg(feature = "geo-types")]
pub mod geo_types;

/// Deserializes a WKT String into any type which implements `TryFromWkt`.
///
/// This is useful when you have a struct which has a structured geometry field, (like a [`geo`](https://docs.rs/geo) or
/// [`geo-types`] geometry) stored as WKT.
///
#[cfg_attr(feature = "geo-types", doc = "```")]
#[cfg_attr(not(feature = "geo-types"), doc = "```ignore")]
/// // This example relies on enabling this crates `serde` and `geo-types` features
/// extern crate geo_types;
/// extern crate serde;
/// extern crate serde_json;
///
/// // If the WKT could be one of several types, deserialize to a `Geometry` enum.
/// let json = r#"[
///   { "geometry": "POINT (3.14 42)", "name": "bob's house" },
///   { "geometry": "LINESTRING (0 0, 1 1, 3.14 42)", "name": "bob's route home" }
/// ]"#;
///
/// #[derive(serde::Deserialize)]
/// struct MyGeomRecord {
///     #[serde(deserialize_with = "wkt::deserialize_wkt")]
///     pub geometry: geo_types::Geometry<f64>,
///     pub name: String,
/// }
/// let my_type: Vec<MyGeomRecord> = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type[0].geometry, geo_types::Geometry::Point(_)));
/// assert!(matches!(my_type[1].geometry, geo_types::Geometry::LineString(_)));
///
/// // If all your records have the same geometry type, deserialize directly to that type.
/// // For example, if I know all the geometry fields will be a POINT, I can do something like:
/// let json = r#"[
///   { "geometry": "POINT (3.14 42)", "name": "bob's house" },
///   { "geometry": "POINT (8.02 23)", "name": "alice's house" }
/// ]"#;
///
/// #[derive(serde::Deserialize)]
/// struct MyPointRecord {
///     #[serde(deserialize_with = "wkt::deserialize_wkt")]
///     pub geometry: geo_types::Point<f64>,
///     pub name: String,
/// }
///
/// let my_type: Vec<MyPointRecord> = serde_json::from_str(json).unwrap();
/// assert_eq!(my_type[0].geometry.x(), 3.14);
/// assert_eq!(my_type[1].geometry.y(), 23.0);
/// ```
pub fn deserialize_wkt<'de, D, G, T>(deserializer: D) -> Result<G, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default + WktNum,
    G: crate::TryFromWkt<T>,
    <G as TryFromWkt<T>>::Error: std::fmt::Display,
{
    deserializer.deserialize_str(TryFromWktVisitor::default())
}

struct TryFromWktVisitor<T, G: TryFromWkt<T>> {
    _marker_t: PhantomData<T>,
    _marker_g: PhantomData<G>,
}

impl<T, G: TryFromWkt<T>> Default for TryFromWktVisitor<T, G> {
    fn default() -> Self {
        Self {
            _marker_t: PhantomData::default(),
            _marker_g: PhantomData::default(),
        }
    }
}

impl<'de, T, G> Visitor<'de> for TryFromWktVisitor<T, G>
where
    T: FromStr + Default + WktNum,
    G: TryFromWkt<T>,
    <G as TryFromWkt<T>>::Error: std::fmt::Display,
{
    type Value = G;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a valid WKT format")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        G::try_from_wkt_str(s).map_err(|e| serde::de::Error::custom(e))
    }
}

struct WktVisitor<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for WktVisitor<T> {
    fn default() -> Self {
        WktVisitor {
            _marker: PhantomData::default(),
        }
    }
}

impl<'de, T> Visitor<'de> for WktVisitor<T>
where
    T: FromStr + Default + Debug + WktNum,
{
    type Value = Wkt<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a valid WKT format")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Wkt::from_str(s).map_err(|e| serde::de::Error::custom(e))
    }
}

impl<'de, T> serde::Deserialize<'de> for Wkt<T>
where
    T: FromStr + Default + Debug + WktNum,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(WktVisitor::default())
    }
}

struct GeometryVisitor<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for GeometryVisitor<T> {
    fn default() -> Self {
        GeometryVisitor {
            _marker: PhantomData::default(),
        }
    }
}

impl<'de, T> Visitor<'de> for GeometryVisitor<T>
where
    T: FromStr + Default + WktNum,
{
    type Value = Geometry<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a valid WKT format")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let wkt = Wkt::from_str(s).map_err(|e| serde::de::Error::custom(e))?;
        Ok(wkt.item)
    }
}

impl<'de, T> serde::Deserialize<'de> for Geometry<T>
where
    T: FromStr + Default + WktNum,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(GeometryVisitor::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{Coord, Point},
        Geometry,
    };
    use serde::de::{
        value::{Error, StrDeserializer},
        Deserializer, Error as _, IntoDeserializer,
    };

    mod wkt {
        use super::*;

        #[test]
        fn deserialize() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (10 20.1)".into_deserializer();
            let wkt = deserializer
                .deserialize_any(WktVisitor::<f64>::default())
                .unwrap();
            assert!(matches!(
                wkt.item,
                Geometry::Point(Point(Some(Coord {
                    x: _, // floating-point types cannot be used in patterns
                    y: _, // floating-point types cannot be used in patterns
                    z: None,
                    m: None,
                })))
            ));
        }

        #[test]
        fn deserialize_error() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (10 20.1A)".into_deserializer();
            let wkt = deserializer.deserialize_any(WktVisitor::<f64>::default());
            assert_eq!(
                wkt.unwrap_err(),
                Error::custom("Unable to parse input number as the desired output type")
            );
        }
    }

    mod geometry {
        use super::*;

        #[test]
        fn deserialize() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (42 3.14)".into_deserializer();
            let geometry = deserializer
                .deserialize_any(GeometryVisitor::<f64>::default())
                .unwrap();
            assert!(matches!(
                geometry,
                Geometry::Point(Point(Some(Coord {
                    x: _, // floating-point types cannot be used in patterns
                    y: _, // floating-point types cannot be used in patterns
                    z: None,
                    m: None,
                })))
            ));
        }

        #[test]
        fn deserialize_error() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (42 PI3.14)".into_deserializer();
            let geometry = deserializer.deserialize_any(GeometryVisitor::<f64>::default());
            assert_eq!(
                geometry.unwrap_err(),
                Error::custom("Expected a number for the Y coordinate")
            );
        }
    }
}
