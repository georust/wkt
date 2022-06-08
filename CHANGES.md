# Changes

## UNRELEASED

### Added
* docs.rs documentation now shows all optional features.
  * <https://github.com/georust/wkt/pull/98>
* docs.rs documentation items are annotated with their feature requirements.
  * <https://github.com/georust/wkt/pull/98>
* `deserialize_wkt` serde integration for all the geo-types (and any other implementer of TryFromWkt)
  * <https://github.com/georust/wkt/pull/99>
* add support for serializing integer geometries, so you can now:
  `geo_types::Point::new(1i32, 2i32).wkt_string()`
  Note that deserializing integer geometries is not yet supported.
  * <https://github.com/georust/wkt/pull/101>

## 0.10.1 - 2022-04-28

### Added
* impl `std::fmt::Display` for `Wkt`.
  * <https://github.com/georust/wkt/pull/88>
* Additions to ToWkt trait:
  * added `wkt_string` and `write_wkt` methods to `ToWkt` trait
    * <https://github.com/georust/wkt/pull/89>
  * impl `ToWkt` on geo_type Geometry variants directly, so you can `point!(x: 1., y: 2.).wkt_string()`
    * <https://github.com/georust/wkt/pull/90>
  * `ToWkt` is no longer tied to geo-types. You can implement it on your own
    custom (non-geo_type) geometry types.
    * <https://github.com/georust/wkt/pull/90>
* New `FromWkt` trait allows a way to convert from a string or reader directly
  to geo-types, without exposing you to the intermediate `Wkt` structs.
    * <https://github.com/georust/wkt/pull/95>
* Implemented `geo_types::GeometryCollection::from(Wkt::from_str(wkt_str))`
    * <https://github.com/georust/wkt/pull/95>

## 0.10.0 - 2022-02-24
### Changed
* Now accepts `MULTIPOINT`s with fewer parentheses, as output by `ST_AsText` in postgis:
  `MULTIPOINT(0 1, 2 3)` in addition to `MULTIPOINT((0 1), (2 3))`
* BREAKING: Replace `Wkt::items` with `Wkt::item` and remove `Wkt::add_item()`.
  * <https://github.com/georust/wkt/pull/72>
* BREAKING: Reject empty strings instead of parsing them into an empty `Wkt`.
  * <https://github.com/georust/wkt/pull/72>
* BREAKING: move `Wkt::from_str` to `FromStr` trait. Add `use std::str::FromStr;` to your code to use it.
  * <https://github.com/georust/wkt/pull/79>
* Switch to 2021 edition and add examples
  * <https://github.com/georust/wkt/pull/65>
* Update to geo-types v0.7.3
* Add MIT license file

## 0.9.2 - 2020-04-30
### Added
* Minimal support for JTS extension: `LINEARRING` by parsing it as a `LINESTRING`.
* Support `POINT EMPTY` in conversion to `geo_types`.
  Converts to `MultiPoint([])`.
  * <https://github.com/georust/wkt/pull/64>
### Fixed
* Some "numeric" characters like `¾` and `①` were being treated as digits.
### Changed
* Approximately 40% faster according to `cargo bench`.

## 0.9.1

* Add `serde::Deserialize` for `Wkt` and `Geometry`.
  * <https://github.com/georust/wkt/pull/59>
* Add helper functions for deserializing from WKT format into
  `geo_types::Geometry` and `geo_types::Point`
  * <https://github.com/georust/wkt/pull/59>
  * <https://github.com/georust/wkt/pull/62>

## 0.9.0

* WKT errors impl `std::error::Error`
  * <https://github.com/georust/wkt/pull/57>
* Add TryFrom for converting directly to geo-types::Geometry enum members, such
  as `geo_types::LineString::try_from(wkt)`
  * <https://github.com/georust/wkt/pull/57>
* Add `geo-types::Geometry::from(wkt)`
* BREAKING: update geo-types, apply new `geo_types::CoordFloat`
  * <https://github.com/georust/wkt/pull/53>
* BREAKING: Add Debug to Wkt structs by using new WktFloat instead of num_traits::Float
  * <https://github.com/georust/wkt/pull/54>

## 0.8.0

* update geo-types
