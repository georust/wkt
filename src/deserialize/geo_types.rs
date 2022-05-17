use crate::{Geometry, Wkt, WktNum};
use serde::de::{Deserialize, Deserializer, Error};
use std::{default::Default, str::FromStr};

#[deprecated(since = "0.10.2", note = "instead use `wkt::deserialize_wkt`")]
/// Deserializes from WKT format into a [`geo_types::Geometry`].
pub fn deserialize_geometry<'de, D, T>(deserializer: D) -> Result<geo_types::Geometry<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default + WktNum,
{
    Geometry::deserialize(deserializer)
        .and_then(|g: Geometry<T>| g.try_into().map_err(D::Error::custom))
}

/// Deserializes from WKT format into an `Option<geo_types::Point>`.
///
/// # Examples
///
///
/// ```
/// # extern crate wkt;
/// # extern crate geo_types;
/// # extern crate serde_json;
/// use geo_types::Point;
/// use wkt::deserialize::geo_types::deserialize_point;
///
/// #[derive(serde::Deserialize)]
/// struct MyType {
///     #[serde(deserialize_with = "deserialize_point")]
///     pub geometry: Option<Point<f64>>,
/// }
///
/// let json = r#"{ "geometry": "POINT (3.14 42)" }"#;
/// let my_type: MyType = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type.geometry, Some(Point(_))));
///
/// let json = r#"{ "geometry": "POINT EMPTY" }"#;
/// let my_type: MyType = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type.geometry, None));
/// ```
pub fn deserialize_point<'de, D, T>(
    deserializer: D,
) -> Result<Option<geo_types::Point<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default + WktNum,
{
    Wkt::deserialize(deserializer).and_then(|wkt: Wkt<T>| {
        geo_types::Geometry::try_from(wkt)
            .map_err(D::Error::custom)
            .and_then(|geom| {
                use geo_types::Geometry::*;
                match geom {
                    Point(p) => Ok(Some(p)),
                    MultiPoint(mp) if mp.0.is_empty() => Ok(None),
                    _ => geo_types::Point::try_from(geom)
                        .map(Some)
                        .map_err(D::Error::custom),
                }
            })
    })
}
