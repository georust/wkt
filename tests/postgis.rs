//! Differential tests against PostGIS.
//!
//! Each test parses generated WKT with this crate and with PostGIS
//! (`ST_GeomFromText`), then compares the results. PostGIS returns WKB, so the
//! coordinates are compared bit for bit, except that all NaN values are equal.
//!
//! The parsers differ on some text. `difference` identifies each known
//! difference, and the test records it as an event instead of failing:
//!
//! - A missing dimension tag: PostGIS reads `POINT(1 2 3)` as `POINT Z`, gives
//!   an untagged collection the dimension of its members, and gives an
//!   untagged member the dimension of its parent. This crate requires the tag.
//! - Validation: PostGIS rejects unclosed rings and too few points. This crate
//!   does not validate geometry.
//! - `POINTEMPTY`: PostGIS accepts it. This crate requires whitespace.
//! - Number forms such as `+1` and `0.E5`: the OGC grammar allows them, and
//!   this crate accepts them. PostGIS rejects them. The same check also finds
//!   `-inf`, `-infinity` and `-nan`: this crate accepts them, because a token
//!   that starts with `-` is read as a number and Rust parses these forms. It
//!   rejects `inf` and `nan`, which it reads as words. None of these is in the
//!   OGC grammar. PostGIS rejects all of them except `nan`.
//! - `nan`: PostGIS reads it as NaN. This crate reads it as a word.
//! - A collection whose members have different dimensions: this crate accepts
//!   it. PostGIS rejects it.
//! - Text after the geometry: this crate ignores it. This is a bug.
//! - An EMPTY ring, as in `POLYGON(EMPTY)`: this crate accepts it. The grammar
//!   and PostGIS do not. This is a bug.
//! - A PostGIS quirk: in a collection with a tag, PostGIS replaces the tag of
//!   an EMPTY member with the tag of the collection. This crate keeps it.
//! - A PostGIS bug: an untagged member of a tagged collection can have too
//!   few coordinates, and PostGIS fills the missing value from uninitialised
//!   memory.
//!
//! `unsupported_wkt_is_rejected` checks that this crate rejects text it does
//! not support: the curve types, TRIANGLE, TIN, POLYHEDRALSURFACE, and EWKT
//! with an SRID. `nested_collections_parse_like_postgis` checks the nesting
//! limit of this crate, which PostGIS does not have.
//!
//! These tests are ignored by default because they need a PostGIS server:
//!
//! ```text
//! docker run -d --name wkt-postgis -e POSTGRES_PASSWORD=wkt -p 54329:5432 postgis/postgis
//! POSTGIS_URL=postgres://postgres:wkt@localhost:54329/postgres \
//!     cargo test --test postgis -- --ignored
//! ```
//!
//! The `postgis/postgis` image is for x86-64 only. On ARM, for example Apple
//! silicon, use `imresamu/postgis`, a multi-architecture build of the same
//! image.
//!
//! `generators_cover_the_grammar` runs without PostGIS. It checks that the
//! generators make each alternative in `GRAMMAR`, the alternatives of the
//! PostGIS grammar, apart from those in `EXCLUDED_RULES` and
//! `EXCLUDED_ALTERNATIVES`. `grammar_matches_postgis_source` checks `GRAMMAR`
//! against the grammar file. PostGIS is GPL-licensed, so the file is not part
//! of this crate:
//!
//! ```text
//! curl -o lwin_wkt_parse.y \
//!     https://raw.githubusercontent.com/postgis/postgis/3.5.3/liblwgeom/lwin_wkt_parse.y
//! POSTGIS_GRAMMAR=lwin_wkt_parse.y cargo test --test postgis -- --ignored grammar_matches
//! ```

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

use geo_traits::{
    CoordTrait, Dimensions, GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait,
};
use hegel::generators as gs;
use hegel::TestCase;
use postgres::{Client, NoTls, Statement};
use wkt::error::ParseError;
use wkt::Wkt;

// The PostGIS connection, shared by all test cases.
struct Postgis {
    client: Client,
    parse: Statement,
}

static POSTGIS: OnceLock<Mutex<Postgis>> = OnceLock::new();

// Parses `wkt` with PostGIS. Returns the WKB, or the PostGIS error message.
fn postgis_wkb(wkt: &str) -> Result<Vec<u8>, String> {
    let postgis = POSTGIS.get_or_init(|| {
        let url = env::var("POSTGIS_URL").unwrap_or_else(|_| {
            panic!("set POSTGIS_URL to a PostGIS connection string; see the module documentation")
        });
        let mut client = Client::connect(&url, NoTls).expect("connect to PostGIS");
        let parse = client
            .prepare("SELECT ST_AsBinary(ST_GeomFromText($1::text))")
            .expect("prepare statement");
        Mutex::new(Postgis { client, parse })
    });
    let mut postgis = postgis.lock().unwrap_or_else(|e| e.into_inner());
    let Postgis { client, parse } = &mut *postgis;
    match client.query_one(&*parse, &[&wkt]) {
        Ok(row) => Ok(row.get(0)),
        Err(e) => Err(e
            .as_db_error()
            .map(|db| db.message().to_string())
            .unwrap_or_else(|| e.to_string())),
    }
}

// A coordinate value that compares by its bits, so that -0.0 does not equal
// 0.0. All NaN values are equal: the crate reads `-nan` with the sign bit set,
// and no PostGIS input gives that value.
#[derive(Clone, Copy)]
struct Num(f64);

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        (self.0.is_nan() && other.0.is_nan()) || self.0.to_bits() == other.0.to_bits()
    }
}

impl fmt::Debug for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

type Coords = Vec<Num>;

// A geometry in a form that both parsers can be converted to.
#[derive(Clone, Debug, PartialEq)]
enum Geom {
    Point(Dimensions, Option<Coords>),
    LineString(Dimensions, Vec<Coords>),
    Polygon(Dimensions, Vec<Vec<Coords>>),
    MultiPoint(Dimensions, Vec<Option<Coords>>),
    MultiLineString(Dimensions, Vec<Vec<Coords>>),
    MultiPolygon(Dimensions, Vec<Vec<Vec<Coords>>>),
    GeometryCollection(Dimensions, Vec<Geom>),
    Other,
}

fn coord(c: &impl CoordTrait<T = f64>) -> Coords {
    (0..c.dim().size())
        .map(|i| Num(c.nth_or_panic(i)))
        .collect()
}

fn point(p: &impl PointTrait<T = f64>) -> Option<Coords> {
    p.coord().map(|c| coord(&c))
}

fn line(l: &impl LineStringTrait<T = f64>) -> Vec<Coords> {
    l.coords().map(|c| coord(&c)).collect()
}

fn rings(p: &impl PolygonTrait<T = f64>) -> Vec<Vec<Coords>> {
    p.exterior()
        .into_iter()
        .chain(p.interiors())
        .map(|r| line(&r))
        .collect()
}

fn geom(g: &impl GeometryTrait<T = f64>) -> Geom {
    let dim = g.dim();
    match g.as_type() {
        GeometryType::Point(p) => Geom::Point(dim, point(p)),
        GeometryType::LineString(l) => Geom::LineString(dim, line(l)),
        GeometryType::Polygon(p) => Geom::Polygon(dim, rings(p)),
        GeometryType::MultiPoint(m) => {
            Geom::MultiPoint(dim, m.points().map(|p| point(&p)).collect())
        }
        GeometryType::MultiLineString(m) => {
            Geom::MultiLineString(dim, m.line_strings().map(|l| line(&l)).collect())
        }
        GeometryType::MultiPolygon(m) => {
            Geom::MultiPolygon(dim, m.polygons().map(|p| rings(&p)).collect())
        }
        GeometryType::GeometryCollection(c) => {
            Geom::GeometryCollection(dim, c.geometries().map(|g| geom(&g)).collect())
        }
        GeometryType::Rect(_) | GeometryType::Triangle(_) | GeometryType::Line(_) => Geom::Other,
    }
}

fn is_empty(g: &Geom) -> bool {
    match g {
        Geom::Point(_, p) => p.is_none(),
        Geom::LineString(_, l) => l.is_empty(),
        Geom::Polygon(_, r) => r.is_empty(),
        Geom::MultiPoint(_, m) => m.iter().all(Option::is_none),
        Geom::MultiLineString(_, m) => m.iter().all(Vec::is_empty),
        Geom::MultiPolygon(_, m) => m.iter().all(Vec::is_empty),
        Geom::GeometryCollection(_, m) => m.iter().all(is_empty),
        Geom::Other => false,
    }
}

// PostGIS writes a collection whose members are all empty as a collection
// with no members, for example `MULTIPOINT(EMPTY)` as `MULTIPOINT EMPTY`.
// Apply the same rule to both results.
fn drop_empty_members(g: Geom) -> Geom {
    let g = match g {
        Geom::GeometryCollection(d, members) => {
            Geom::GeometryCollection(d, members.into_iter().map(drop_empty_members).collect())
        }
        other => other,
    };
    if !is_empty(&g) {
        return g;
    }
    match g {
        Geom::MultiPoint(d, _) => Geom::MultiPoint(d, vec![]),
        Geom::MultiLineString(d, _) => Geom::MultiLineString(d, vec![]),
        Geom::MultiPolygon(d, _) => Geom::MultiPolygon(d, vec![]),
        Geom::GeometryCollection(d, _) => Geom::GeometryCollection(d, vec![]),
        other => other,
    }
}

fn dimensions(g: &Geom) -> Option<Dimensions> {
    match g {
        Geom::Point(d, _)
        | Geom::LineString(d, _)
        | Geom::Polygon(d, _)
        | Geom::MultiPoint(d, _)
        | Geom::MultiLineString(d, _)
        | Geom::MultiPolygon(d, _)
        | Geom::GeometryCollection(d, _) => Some(*d),
        Geom::Other => None,
    }
}

// The result of one parser.
#[derive(Debug, PartialEq)]
enum Outcome {
    Parsed(Geom),
    Rejected,
}

// The result of this crate, and its error if it rejects `text`.
fn parse_with_crate(text: &str) -> (Outcome, Option<ParseError>) {
    match Wkt::<f64>::from_str(text) {
        Ok(wkt) => (Outcome::Parsed(drop_empty_members(geom(&wkt))), None),
        Err(e) => (Outcome::Rejected, Some(e)),
    }
}

// The result of PostGIS, and its error message if it rejects `text`.
fn parse_with_postgis(text: &str) -> (Outcome, Option<String>) {
    match postgis_wkb(text) {
        Ok(bytes) => {
            let wkb = wkb::reader::read_wkb(&bytes).expect("PostGIS returns valid WKB");
            (Outcome::Parsed(drop_empty_members(geom(&wkb))), None)
        }
        Err(message) => (Outcome::Rejected, Some(message)),
    }
}

// The rule, type token and body rule of each type in the PostGIS grammar.
fn grammar_names(kind: &str) -> (&'static str, &'static str, &'static str) {
    match kind {
        "POINT" => ("point", "POINT_TOK", "ptarray"),
        "LINESTRING" => ("linestring", "LINESTRING_TOK", "ptarray"),
        "POLYGON" => ("polygon", "POLYGON_TOK", "ring_list"),
        "MULTIPOINT" => ("multipoint", "MPOINT_TOK", "point_list"),
        "MULTILINESTRING" => ("multilinestring", "MLINESTRING_TOK", "linestring_list"),
        "MULTIPOLYGON" => ("multipolygon", "MPOLYGON_TOK", "polygon_list"),
        "GEOMETRYCOLLECTION" => ("geometrycollection", "COLLECTION_TOK", "geometry_list"),
        "CIRCULARSTRING" => ("circularstring", "CIRCULARSTRING_TOK", "ptarray"),
        "COMPOUNDCURVE" => ("compoundcurve", "COMPOUNDCURVE_TOK", "compound_list"),
        "CURVEPOLYGON" => ("curvepolygon", "CURVEPOLYGON_TOK", "curvering_list"),
        "MULTICURVE" => ("multicurve", "MCURVE_TOK", "curve_list"),
        "MULTISURFACE" => ("multisurface", "MSURFACE_TOK", "surface_list"),
        "POLYHEDRALSURFACE" => ("polyhedralsurface", "POLYHEDRALSURFACE_TOK", "patch_list"),
        "TRIANGLE" => (
            "triangle",
            "TRIANGLE_TOK",
            "LBRACKET_TOK ptarray RBRACKET_TOK",
        ),
        "TIN" => ("tin", "TIN_TOK", "triangle_list"),
        _ => unreachable!("not a geometry type: {kind}"),
    }
}

const KINDS: [&str; 7] = [
    "POINT",
    "LINESTRING",
    "POLYGON",
    "MULTIPOINT",
    "MULTILINESTRING",
    "MULTIPOLYGON",
    "GEOMETRYCOLLECTION",
];

// `text` with `tag` after each geometry type name that has no tag.
fn add_tags(text: &str, tag: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut rest = text;
    while let Some(start) = rest.find(|c: char| c.is_ascii_alphabetic()) {
        let len = rest[start..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(rest.len() - start);
        let word = &rest[start..start + len];
        out += &rest[..start + len];
        rest = &rest[start + len..];
        if KINDS.iter().any(|k| k.eq_ignore_ascii_case(word)) {
            let next = rest.trim_start();
            let next_len = next
                .find(|c: char| !c.is_ascii_alphabetic())
                .unwrap_or(next.len());
            let next_word = &next[..next_len];
            if !["Z", "M", "ZM"]
                .iter()
                .any(|t| t.eq_ignore_ascii_case(next_word))
            {
                out += " ";
                out += tag;
            }
        }
    }
    out + rest
}

// True if this crate reads `text` as PostGIS does once PostGIS's dimension
// tag is written after each untagged geometry type name.
fn only_tags_missing(text: &str, postgis: &Outcome) -> bool {
    let Outcome::Parsed(expected) = postgis else {
        return false;
    };
    let tag = match dimensions(expected) {
        Some(Dimensions::Xyz) => "Z",
        Some(Dimensions::Xym) => "M",
        Some(Dimensions::Xyzm) => "ZM",
        _ => return false,
    };
    let tagged = add_tags(text, tag);
    tagged != text && parse_with_crate(&tagged).0 == *postgis
}

// True if PostGIS reads `text` as `ours`, allowing for missing dimension
// tags and for geometry validation.
fn postgis_agrees(text: &str, ours: &Outcome) -> bool {
    let (postgis, message) = parse_with_postgis(text);
    postgis == *ours
        || (parse_with_crate(text).0 == *ours && only_tags_missing(text, &postgis))
        || (matches!(ours, Outcome::Parsed(_)) && is_validation_error(message.as_deref()))
}

// PostGIS errors for a collection whose members have different dimensions,
// such as `GEOMETRYCOLLECTION(POINT EMPTY,POINT Z EMPTY)`. This crate accepts
// such a collection.
const MIXED_DIMENSIONS: [&str; 2] = [
    "Dimensions mismatch in lwcollection",
    "can not mix dimensionality in a geometry",
];

// The result of this crate without `drop_empty_members`, which removes the
// members of a collection whose members are all empty.
fn unnormalised_crate_geom(text: &str) -> Option<Geom> {
    Wkt::<f64>::from_str(text).ok().map(|wkt| geom(&wkt))
}

// True if `g` has a geometry collection with a member whose dimensions are
// different from the dimensions of the collection.
fn has_mixed_members(g: &Geom) -> bool {
    match g {
        Geom::GeometryCollection(d, members) => members
            .iter()
            .any(|m| dimensions(m) != Some(*d) || has_mixed_members(m)),
        _ => false,
    }
}

fn is_validation_error(message: Option<&str>) -> bool {
    message.is_some_and(|m| VALIDATION_ERRORS.contains(&m))
}

// PostGIS bug: in `GEOMETRYCOLLECTION Z(MULTIPOINT(0 0 0,7 8))`, PostGIS
// accepts the point with two coordinates and gives it a Z value from
// uninitialised memory. With the tags written out, PostGIS rejects the text.
// True if PostGIS accepts `text` but rejects it with its tags written out.
fn postgis_rejects_tagged(text: &str, postgis: &Outcome) -> bool {
    let Outcome::Parsed(expected) = postgis else {
        return false;
    };
    let tag = match dimensions(expected) {
        Some(Dimensions::Xyz) => "Z",
        Some(Dimensions::Xym) => "M",
        Some(Dimensions::Xyzm) => "ZM",
        _ => return false,
    };
    let tagged = add_tags(text, tag);
    tagged != text && parse_with_postgis(&tagged).0 == Outcome::Rejected
}

fn with_dim(g: Geom, d: Dimensions) -> Geom {
    match g {
        Geom::Point(_, x) => Geom::Point(d, x),
        Geom::LineString(_, x) => Geom::LineString(d, x),
        Geom::Polygon(_, x) => Geom::Polygon(d, x),
        Geom::MultiPoint(_, x) => Geom::MultiPoint(d, x),
        Geom::MultiLineString(_, x) => Geom::MultiLineString(d, x),
        Geom::MultiPolygon(_, x) => Geom::MultiPolygon(d, x),
        Geom::GeometryCollection(_, x) => Geom::GeometryCollection(d, x),
        Geom::Other => Geom::Other,
    }
}

// `g` with each empty member of a geometry collection given the dimensions of
// the collection.
fn empty_members_take_parent_dims(g: Geom) -> Geom {
    match g {
        Geom::GeometryCollection(d, members) => Geom::GeometryCollection(
            d,
            members
                .into_iter()
                .map(|m| {
                    let m = empty_members_take_parent_dims(m);
                    if is_empty(&m) {
                        with_dim(m, d)
                    } else {
                        m
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

// PostGIS quirk: in a collection with a tag, PostGIS replaces the tag of an
// EMPTY member with the tag of the collection, so that
// `GEOMETRYCOLLECTION Z(POINT M EMPTY)` has a `POINT Z EMPTY`. This crate
// keeps the tag of the member. True if the results are the same when each
// empty member has the dimensions of its collection, allowing for missing
// dimension tags.
fn only_empty_member_tags(text: &str, ours: &Outcome, postgis: &Outcome) -> bool {
    let Outcome::Parsed(expected) = postgis else {
        return false;
    };
    let tag = match dimensions(expected) {
        Some(Dimensions::Xyz) => "Z",
        Some(Dimensions::Xym) => "M",
        Some(Dimensions::Xyzm) => "ZM",
        _ => return false,
    };
    let tagged = parse_with_crate(&add_tags(text, tag)).0;
    let matches = |outcome: &Outcome| match outcome {
        Outcome::Parsed(g) => empty_members_take_parent_dims(g.clone()) == *expected,
        Outcome::Rejected => false,
    };
    matches(ours) || matches(&tagged)
}

// A bug in this crate that the differential tests find. `difference`
// reports the cases that `KnownBug::check` explains as the bug, and
// `known_bugs_still_occur` fails when `example` stops showing the bug. When a
// bug is fixed, remove it from `KNOWN_BUGS` and remove its `should_panic`
// unit test in `src/lib.rs`.
struct KnownBug {
    label: &'static str,
    // Text that PostGIS rejects and this crate accepts.
    example: &'static str,
}

// `Wkt::from_str` stops after the first geometry and ignores the text after
// it. See `only_trailing_text`.
const TRAILING_TEXT: KnownBug = KnownBug {
    label: "bug: text after the geometry is ignored",
    example: "POINT(1 2) garbage",
};

// This crate reads polygon rings with the linestring parser, which accepts
// EMPTY. See `has_empty_ring`.
const EMPTY_RING: KnownBug = KnownBug {
    label: "bug: an EMPTY ring is accepted",
    example: "POLYGON(EMPTY)",
};

const KNOWN_BUGS: [KnownBug; 2] = [TRAILING_TEXT, EMPTY_RING];

#[test]
fn known_bugs_still_occur() {
    for bug in KNOWN_BUGS {
        assert!(
            Wkt::<f64>::from_str(bug.example).is_ok(),
            "{:?} is rejected, so \"{}\" looks fixed: remove it from KNOWN_BUGS, \
             with its check in `difference` and its unit test in src/lib.rs",
            bug.example,
            bug.label
        );
    }
}

// True if `g` has a polygon with an empty ring.
fn has_empty_ring(g: &Geom) -> bool {
    let empty_ring = |rings: &Vec<Vec<Coords>>| rings.iter().any(Vec::is_empty);
    match g {
        Geom::Polygon(_, rings) => empty_ring(rings),
        Geom::MultiPolygon(_, polygons) => polygons.iter().any(empty_ring),
        Geom::GeometryCollection(_, members) => members.iter().any(has_empty_ring),
        _ => false,
    }
}

// PostGIS errors from geometry validation, not from the grammar.
const VALIDATION_ERRORS: [&str; 2] = [
    "geometry contains non-closed rings",
    "geometry requires more points",
];

// `text` with a space before each EMPTY that follows a letter, so that
// `POINTEMPTY` becomes `POINT EMPTY`.
fn separate_empty(text: &str) -> String {
    let upper = text.to_ascii_uppercase();
    let mut out = String::with_capacity(text.len() + 4);
    let mut last = 0;
    for (i, _) in upper.match_indices("EMPTY") {
        if i > 0 && upper.as_bytes()[i - 1].is_ascii_alphabetic() {
            out += &text[last..i];
            out += " ";
            last = i;
        }
    }
    out + &text[last..]
}

// True if this crate reads `text` as PostGIS does once a space is written
// before each EMPTY that follows a letter, allowing for missing dimension
// tags.
fn only_attached_empty(text: &str, postgis: &Outcome) -> bool {
    let separated = separate_empty(text);
    separated != text
        && (parse_with_crate(&separated).0 == *postgis || only_tags_missing(&separated, postgis))
}

// True if PostGIS reads `text` as `ours` when each number is written in the
// form Rust prints, allowing for missing dimension tags. This finds number
// forms that the OGC grammar allows and PostGIS rejects, such as `+1` and
// `0.E5`.
fn only_number_spelling(text: &str, ours: &Outcome) -> bool {
    let rewritten = respell_numbers(text);
    rewritten != text && postgis_agrees(&rewritten, ours)
}

// `text` with each token replaced by `f(token)`. Tokens end at whitespace,
// parentheses and commas, as in the crate's tokenizer.
fn map_tokens(text: &str, f: impl Fn(&str) -> String) -> String {
    let is_break = |c: char| c.is_whitespace() || matches!(c, '(' | ')' | ',');
    let mut out = String::with_capacity(text.len());
    let mut token = String::new();
    for c in text.chars() {
        if is_break(c) {
            out += &f(&token);
            token.clear();
            out.push(c);
        } else {
            token.push(c);
        }
    }
    out + &f(&token)
}

// `text` with each number written in a form that PostGIS reads.
fn respell_numbers(text: &str) -> String {
    map_tokens(text, |token| {
        let unsigned = token.strip_prefix('+').unwrap_or(token);
        let is_number = token.starts_with(|c: char| c.is_ascii_digit() || "+-.".contains(c));
        match unsigned.parse::<f64>() {
            Ok(n) if is_number && n.is_finite() => format!("{n:?}"),
            Ok(n) if is_number && n.is_nan() => "nan".to_string(),
            // Rust prints infinity as `inf`, which PostGIS does not read.
            // PostGIS reads an overflow as infinity.
            Ok(n) if is_number && n < 0.0 => "-1e999".to_string(),
            Ok(_) if is_number => "1e999".to_string(),
            _ => token.to_string(),
        }
    })
}

// True if this crate reads `text` as PostGIS does once each `nan` is written
// as `-nan`. The crate reads `nan` as a word, and `-nan` as a number.
fn only_nan_word(text: &str, postgis: &Outcome) -> bool {
    let rewritten = map_tokens(text, |token| {
        if token.eq_ignore_ascii_case("nan") {
            "-nan".to_string()
        } else {
            token.to_string()
        }
    });
    rewritten != text
        && (parse_with_crate(&rewritten).0 == *postgis || only_tags_missing(&rewritten, postgis))
}

// True if `text` has a strict prefix, ending in `)` or `EMPTY`, that both
// parsers read as `ours`, allowing for missing dimension tags and number
// forms that PostGIS rejects.
fn only_trailing_text(text: &str, ours: &Outcome) -> bool {
    let ends = text
        .char_indices()
        .map(|(i, c)| i + c.len_utf8())
        .filter(|&end| end < text.len());
    ends.into_iter().any(|end| {
        let prefix = &text[..end];
        let ends_geometry = prefix.ends_with(')') || prefix.to_ascii_uppercase().ends_with("EMPTY");
        ends_geometry
            && parse_with_crate(prefix).0 == *ours
            && (postgis_agrees(prefix, ours) || postgis_agrees(&respell_numbers(prefix), ours))
    })
}

// Names the known difference that explains why the results for `text`
// differ, or returns `None`.
fn difference(
    text: &str,
    ours: &Outcome,
    postgis: &Outcome,
    message: Option<&str>,
) -> Option<&'static str> {
    if only_tags_missing(text, postgis) {
        return Some("missing dimension tag");
    }
    if only_empty_member_tags(text, ours, postgis) {
        return Some("PostGIS replaces the tag of an EMPTY member");
    }
    match (ours, postgis) {
        (Outcome::Parsed(_), Outcome::Rejected) => {
            if is_validation_error(message) {
                Some("PostGIS validates the geometry")
            } else if message.is_some_and(|m| MIXED_DIMENSIONS.contains(&m))
                && unnormalised_crate_geom(text).is_some_and(|g| has_mixed_members(&g))
            {
                Some("members with different dimensions")
            } else if matches!(ours, Outcome::Parsed(g) if has_empty_ring(g)) {
                Some(EMPTY_RING.label)
            } else if only_number_spelling(text, ours) {
                Some("number form that PostGIS rejects")
            } else if only_trailing_text(text, ours) {
                Some(TRAILING_TEXT.label)
            } else {
                None
            }
        }
        (Outcome::Rejected, Outcome::Parsed(_)) if postgis_rejects_tagged(text, postgis) => {
            Some("PostGIS bug: an untagged member has too few coordinates")
        }
        (Outcome::Rejected, Outcome::Parsed(_)) if only_attached_empty(text, postgis) => {
            Some("no whitespace before EMPTY")
        }
        (Outcome::Rejected, Outcome::Parsed(_)) if only_nan_word(text, postgis) => {
            Some("nan read as a word")
        }
        _ => None,
    }
}

fn assert_same_as_postgis(tc: &TestCase, text: &str) {
    let (postgis, message) = parse_with_postgis(text);
    let (ours, error) = parse_with_crate(text);
    if ours == postgis {
        return;
    }
    match difference(text, &ours, &postgis, message.as_deref()) {
        Some(reason) => tc.event(reason),
        None => panic!(
            "input: {text:?}\n  this crate: {ours:?}, error {error:?}\n  PostGIS: {postgis:?}, \
             error {message:?}"
        ),
    }
}

// Generators. `valid_wkt` makes text that PostGIS accepts: linestrings have at
// least two points, rings are closed and have at least four points, and the
// members of a collection have the dimensions of the collection.

#[derive(Clone, Copy, Debug)]
enum Dim {
    Xy,
    Xyz,
    Xym,
    Xyzm,
}

impl Dim {
    fn size(self) -> usize {
        match self {
            Dim::Xy => 2,
            Dim::Xyz | Dim::Xym => 3,
            Dim::Xyzm => 4,
        }
    }
}

thread_local! {
    // The productions that the generators chose on this thread.
    static PRODUCTIONS_SEEN: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

thread_local! {
    // While this is above zero, `production` records nothing.
    static QUIET: Cell<u32> = const { Cell::new(0) };
}

// Records that the generator chose `name`: an alternative from `GRAMMAR`, or
// a lexer form from `LEXER_FORMS`.
fn production(tc: &TestCase, name: &str) {
    if QUIET.with(Cell::get) > 0 {
        return;
    }
    tc.event(format!("production: {name}"));
    PRODUCTIONS_SEEN.with(|seen| seen.borrow_mut().insert(name.to_string()));
}

// Runs `f` with recording off. The generators use this inside the types that
// this crate does not support, where helpers such as `polygon_body` would
// record the wrong rules.
fn quietly<R>(f: impl FnOnce() -> R) -> R {
    QUIET.with(|q| q.set(q.get() + 1));
    let result = f();
    QUIET.with(|q| q.set(q.get() - 1));
    result
}

// Records `rule: item` for the first of `n` items of a list rule, and
// `rule: rule COMMA_TOK item` for each other item.
fn list_productions(tc: &TestCase, rule: &str, item: &str, n: usize) {
    if n > 0 {
        production(tc, &format!("{rule}: {item}"));
    }
    if n > 1 {
        production(tc, &format!("{rule}: {rule} COMMA_TOK {item}"));
    }
}

fn whitespace(tc: &TestCase, text: &'static str) -> &'static str {
    if text.is_empty() {
        production(tc, "lexer: whitespace: none");
    }
    for c in text.chars() {
        production(
            tc,
            match c {
                ' ' => "lexer: whitespace: space",
                '\t' => "lexer: whitespace: tab",
                '\n' => "lexer: whitespace: newline",
                _ => "lexer: whitespace: carriage return",
            },
        );
    }
    text
}

// Optional whitespace.
fn ws(tc: &TestCase) -> &'static str {
    let text = tc.draw(gs::sampled_from(
        &["", "", " ", "  ", "\t", "\n", " \r\n "][..],
    ));
    whitespace(tc, text)
}

// Required whitespace.
fn sep(tc: &TestCase) -> &'static str {
    let text = tc.draw(gs::sampled_from(&[" ", " ", "  ", "\t", "\n"][..]));
    whitespace(tc, text)
}

fn number(tc: &TestCase) -> String {
    let text = number_text(tc);
    for name in number_productions(&text) {
        production(tc, name);
    }
    text
}

// The forms of the PostGIS DOUBLE token that `text` uses.
fn number_productions(text: &str) -> Vec<&'static str> {
    let mut names = vec![];
    let unsigned = match text.strip_prefix('-') {
        Some(rest) => {
            names.push("lexer: DOUBLE: negative");
            rest
        }
        None => text,
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(i) => (&unsigned[..i], Some(&unsigned[i + 1..])),
        None => (unsigned, None),
    };
    names.push(match mantissa.find('.') {
        None => "lexer: DOUBLE: digits",
        Some(0) => "lexer: DOUBLE: . digits",
        Some(i) if i + 1 == mantissa.len() => "lexer: DOUBLE: digits .",
        Some(_) => "lexer: DOUBLE: digits . digits",
    });
    match exponent {
        Some(e) if e.starts_with(['-', '+']) => names.push("lexer: DOUBLE: exponent with sign"),
        Some(_) => names.push("lexer: DOUBLE: exponent"),
        None => {}
    }
    names
}

fn number_text(tc: &TestCase) -> String {
    match tc.draw(gs::integers::<u8>().max_value(4)) {
        0 => tc.draw(gs::integers::<i32>()).to_string(),
        4 => lexer_number(tc),
        style => {
            let x: f64 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
            match style {
                1 => format!("{x}"),
                2 => format!("{x:e}"),
                _ => format!("{x:?}"),
            }
        }
    }
}

fn digits(tc: &TestCase, min: usize, max: usize) -> String {
    (0..count(tc, min, max))
        .map(|_| char::from(b'0' + tc.draw(gs::integers::<u8>().max_value(9))))
        .collect()
}

// A number in a form that the PostGIS lexer reads, which is
// `-?([0-9]+\.?|[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?)`: for example `5.`, `.5`,
// `007` and `-.5e-3`.
fn lexer_number(tc: &TestCase) -> String {
    let mut text = String::from(if tc.draw(gs::booleans()) { "-" } else { "" });
    if tc.draw(gs::booleans()) {
        text += &digits(tc, 1, 25);
        if tc.draw(gs::booleans()) {
            text.push('.');
        }
    } else {
        text += &digits(tc, 0, 25);
        if tc.draw(gs::booleans()) {
            text.push('.');
        }
        text += &digits(tc, 1, 25);
        if tc.draw(gs::booleans()) {
            text += tc.draw(gs::sampled_from(&["e", "E", "e-", "e+", "E-", "E+"][..]));
            text += &digits(tc, 1, 4);
        }
    }
    text
}

fn coord_text(tc: &TestCase, dim: Dim) -> String {
    production(
        tc,
        match dim.size() {
            2 => "coordinate: DOUBLE_TOK DOUBLE_TOK",
            3 => "coordinate: DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK",
            _ => "coordinate: DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK",
        },
    );
    let mut text = number(tc);
    for _ in 1..dim.size() {
        text += sep(tc);
        text += &number(tc);
    }
    text
}

// `(a, b, ...)` with random whitespace.
fn list(tc: &TestCase, items: Vec<String>) -> String {
    let mut text = String::from("(");
    for (i, item) in items.iter().enumerate() {
        text += ws(tc);
        if i > 0 {
            text += ",";
            text += ws(tc);
        }
        text += item;
    }
    text += ws(tc);
    text += ")";
    text
}

fn count(tc: &TestCase, min: usize, max: usize) -> usize {
    tc.draw(gs::integers::<usize>().min_value(min).max_value(max))
}

// `( ptarray )` with two or more coordinates.
fn linestring_body(tc: &TestCase, dim: Dim) -> String {
    let n = count(tc, 2, 4);
    list_productions(tc, "ptarray", "coordinate", n);
    let items = (0..n).map(|_| coord_text(tc, dim)).collect();
    list(tc, items)
}

fn ring_body(tc: &TestCase, dim: Dim) -> String {
    production(tc, "ring: LBRACKET_TOK ptarray RBRACKET_TOK");
    let n = count(tc, 3, 5);
    list_productions(tc, "ptarray", "coordinate", n + 1);
    let mut items: Vec<String> = (0..n).map(|_| coord_text(tc, dim)).collect();
    items.push(items[0].clone());
    list(tc, items)
}

fn polygon_body(tc: &TestCase, dim: Dim) -> String {
    let rings = count(tc, 1, 3);
    list_productions(tc, "ring_list", "ring", rings);
    let items = (0..rings).map(|_| ring_body(tc, dim)).collect();
    list(tc, items)
}

fn word(tc: &TestCase, name: &str) -> String {
    let case = tc.draw(gs::integers::<u8>().max_value(2));
    production(
        tc,
        match case {
            0 => "lexer: keyword: upper case",
            1 => "lexer: keyword: lower case",
            _ => "lexer: keyword: mixed case",
        },
    );
    match case {
        0 => name.to_string(),
        1 => name.to_lowercase(),
        _ => name
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_lowercase()
                } else {
                    c
                }
            })
            .collect(),
    }
}

// Writes the type name and, unless it is left out, the dimension tag.
// `explicit` forces the tag. Without a tag, PostGIS reads three coordinates
// as Z and four as ZM, so XYM always has a tag. Returns the text, and true if
// it has a tag.
fn header(tc: &TestCase, kind: &str, dim: Dim, explicit: bool) -> (String, bool) {
    let mut text = word(tc, kind);
    let implicit_allowed = !explicit && !matches!(dim, Dim::Xym);
    if implicit_allowed && tc.draw(gs::booleans()) {
        return (text, false);
    }
    let tag = match dim {
        Dim::Xy => return (text, false),
        Dim::Xyz => "Z",
        Dim::Xym => "M",
        Dim::Xyzm => "ZM",
    };
    production(tc, &format!("lexer: DIMENSIONALITY: {tag}"));
    if tc.draw(gs::booleans()) {
        production(tc, "lexer: DIMENSIONALITY: after whitespace");
        text += sep(tc);
    } else {
        production(tc, "lexer: DIMENSIONALITY: attached to the type name");
    }
    text += &word(tc, tag);
    (text, true)
}

// `rule` is the grammar rule of the member, such as `linestring_untagged`.
fn member_or_empty(
    tc: &TestCase,
    rule: &str,
    empty_allowed: bool,
    body: impl Fn(&TestCase) -> String,
) -> String {
    if empty_allowed && tc.draw(gs::integers::<u8>().max_value(5)) == 0 {
        production(tc, &format!("{rule}: EMPTY_TOK"));
        word(tc, "EMPTY")
    } else {
        body(tc)
    }
}

// A geometry of `kind`. `declared` is true if the parent collection has a
// dimension tag. `depth` limits the nesting of collections.
fn geometry(tc: &TestCase, kind: &str, dim: Dim, declared: bool, depth: usize) -> String {
    let empty = tc.draw(gs::integers::<u8>().max_value(5)) == 0;
    // EMPTY carries no coordinates, so the dimension must come from a tag:
    // this one, or the tag of the parent collection.
    let (mut text, tagged) = header(tc, kind, dim, empty && !declared);
    let (rule, token, body) = grammar_names(kind);
    production(tc, &format!("geometry_no_srid: {rule}"));
    production(
        tc,
        &format!(
            "{rule}: {token}{}{}",
            if tagged { " DIMENSIONALITY_TOK" } else { "" },
            if empty {
                " EMPTY_TOK".to_string()
            } else {
                format!(" LBRACKET_TOK {body} RBRACKET_TOK")
            }
        ),
    );
    if empty {
        text += sep(tc);
        text += &word(tc, "EMPTY");
        return text;
    }
    text += ws(tc);
    // PostGIS gives an EMPTY member of a multi-geometry the dimension of the
    // multi-geometry's own tag, or XY if it has none.
    let empty_members = tagged || dim.size() == 2;
    let body = match kind {
        "POINT" => {
            production(tc, "ptarray: coordinate");
            list(tc, vec![coord_text(tc, dim)])
        }
        "LINESTRING" => linestring_body(tc, dim),
        "POLYGON" => polygon_body(tc, dim),
        "MULTIPOINT" => {
            let n = count(tc, 1, 4);
            list_productions(tc, "point_list", "point_untagged", n);
            let items = (0..n)
                .map(|_| match tc.draw(gs::integers::<u8>().max_value(4)) {
                    0 if empty_members => {
                        production(tc, "point_untagged: EMPTY_TOK");
                        word(tc, "EMPTY")
                    }
                    0..=2 => {
                        production(tc, "point_untagged: coordinate");
                        coord_text(tc, dim)
                    }
                    _ => {
                        production(tc, "point_untagged: LBRACKET_TOK coordinate RBRACKET_TOK");
                        list(tc, vec![coord_text(tc, dim)])
                    }
                })
                .collect();
            list(tc, items)
        }
        "MULTILINESTRING" => {
            let n = count(tc, 1, 3);
            list_productions(tc, "linestring_list", "linestring_untagged", n);
            let items = (0..n)
                .map(|_| {
                    member_or_empty(tc, "linestring_untagged", empty_members, |tc| {
                        production(tc, "linestring_untagged: LBRACKET_TOK ptarray RBRACKET_TOK");
                        linestring_body(tc, dim)
                    })
                })
                .collect();
            list(tc, items)
        }
        "MULTIPOLYGON" => {
            let n = count(tc, 1, 3);
            list_productions(tc, "polygon_list", "polygon_untagged", n);
            let items = (0..n)
                .map(|_| {
                    member_or_empty(tc, "polygon_untagged", empty_members, |tc| {
                        production(tc, "polygon_untagged: LBRACKET_TOK ring_list RBRACKET_TOK");
                        polygon_body(tc, dim)
                    })
                })
                .collect();
            list(tc, items)
        }
        _ => {
            // PostGIS gives an untagged member the tag of its parent only,
            // not the tag of a collection further out.
            let n = count(tc, 1, 3);
            list_productions(tc, "geometry_list", "geometry_no_srid", n);
            let items = (0..n)
                .map(|_| {
                    let kinds = if depth == 0 { &KINDS[..6] } else { &KINDS[..] };
                    let kind = tc.draw(gs::sampled_from(kinds));
                    geometry(tc, kind, dim, tagged, depth.saturating_sub(1))
                })
                .collect();
            list(tc, items)
        }
    };
    text + &body
}

fn valid_wkt(tc: &TestCase) -> String {
    production(tc, "geometry: geometry_no_srid");
    valid_geometry(tc)
}

// `valid_wkt` without the record of the `geometry` rule.
fn valid_geometry(tc: &TestCase) -> String {
    let kind = tc.draw(gs::sampled_from(&KINDS[..]));
    let dim = tc.draw(gs::sampled_from(
        &[Dim::Xy, Dim::Xyz, Dim::Xym, Dim::Xyzm][..],
    ));
    let text = geometry(tc, kind, dim, false, 2);
    format!("{}{}{}", ws(tc), text, ws(tc))
}

// Characters that a mutation can insert. PostgreSQL text cannot hold NUL, so
// NUL is not included. The vertical tab, form feed and no-break space are
// whitespace that neither parser accepts.
const MUTATION_ALPHABET: &str = "0123456789.+-eE ,()\tZMzmEMPTYPOINTnaif;=\u{b}\u{c}\u{a0}\u{e9}";

// Tokens that a token mutation can insert.
const MUTATION_TOKENS: [&str; 12] = [
    "(",
    ")",
    ",",
    "EMPTY",
    "Z",
    "M",
    "ZM",
    "POINT",
    "LINESTRING",
    "GEOMETRYCOLLECTION",
    "0",
    "1.5",
];

// `text` split into words, numbers, parentheses, commas and runs of
// whitespace. Joining the tokens gives `text`.
fn tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in text.chars() {
        let single = matches!(c, '(' | ')' | ',');
        let same_kind = out.last().is_some_and(|last| {
            let l = last.chars().next().unwrap();
            !single && !matches!(l, '(' | ')' | ',') && l.is_whitespace() == c.is_whitespace()
        });
        if same_kind {
            out.last_mut().unwrap().push(c);
        } else {
            out.push(c.to_string());
        }
    }
    out
}

fn char_edit(tc: &TestCase, chars: &mut Vec<char>) {
    let at = count(tc, 0, chars.len());
    let insert = || -> char {
        tc.draw(
            gs::text()
                .alphabet(MUTATION_ALPHABET)
                .min_size(1)
                .max_size(1),
        )
        .chars()
        .next()
        .unwrap()
    };
    match tc.draw(gs::integers::<u8>().max_value(2)) {
        0 if at < chars.len() => {
            chars.remove(at);
        }
        1 if at < chars.len() => chars[at] = insert(),
        _ => chars.insert(at, insert()),
    }
}

fn token_edit(tc: &TestCase, tokens: &mut Vec<String>) {
    let at = count(tc, 0, tokens.len());
    match tc.draw(gs::integers::<u8>().max_value(3)) {
        0 if at < tokens.len() => {
            tokens.remove(at);
        }
        1 if at < tokens.len() => {
            let token = tokens[at].clone();
            tokens.insert(at, token);
        }
        2 if at + 1 < tokens.len() => tokens.swap(at, at + 1),
        _ => {
            let token = tc.draw(gs::sampled_from(&MUTATION_TOKENS[..]));
            tokens.insert(at, format!(" {token} "));
        }
    }
}

// `valid_wkt` with one to three edits. An edit changes one character, or
// deletes, duplicates, swaps or inserts one token.
fn mutated_wkt(tc: &TestCase) -> String {
    let mut text = valid_wkt(tc);
    for _ in 0..count(tc, 1, 3) {
        if tc.draw(gs::booleans()) {
            let mut chars: Vec<char> = text.chars().collect();
            char_edit(tc, &mut chars);
            text = chars.into_iter().collect();
        } else {
            let mut t = tokens(&text);
            token_edit(tc, &mut t);
            text = t.concat();
        }
    }
    text
}

// Geometry types in the PostGIS grammar that this crate does not support.
const UNSUPPORTED_KINDS: [&str; 8] = [
    "CIRCULARSTRING",
    "COMPOUNDCURVE",
    "CURVEPOLYGON",
    "MULTICURVE",
    "MULTISURFACE",
    "POLYHEDRALSURFACE",
    "TRIANGLE",
    "TIN",
];

// A geometry of a type that this crate does not support, in a form that
// PostGIS accepts.
fn unsupported_geometry(tc: &TestCase, in_collection: bool) -> String {
    let kind = tc.draw(gs::sampled_from(&UNSUPPORTED_KINDS[..]));
    let (rule, token, body_rule) = grammar_names(kind);
    production(tc, &format!("geometry_no_srid: {rule}"));
    let mut text = word(tc, kind);
    if tc.draw(gs::integers::<u8>().max_value(5)) == 0 {
        // A tag inside a collection would have to match the collection.
        if !in_collection && tc.draw(gs::booleans()) {
            production(tc, &format!("{rule}: {token} DIMENSIONALITY_TOK EMPTY_TOK"));
            text += sep(tc);
            text += &word(tc, tc.draw(gs::sampled_from(&["Z", "M", "ZM"][..])));
        } else {
            production(tc, &format!("{rule}: {token} EMPTY_TOK"));
        }
        return text + sep(tc) + &word(tc, "EMPTY");
    }
    production(
        tc,
        &format!("{rule}: {token} LBRACKET_TOK {body_rule} RBRACKET_TOK"),
    );
    text += ws(tc);
    let xy = |tc: &TestCase| coord_text(tc, Dim::Xy);
    // A circular string has an odd number of points, three or more.
    let arc = |tc: &TestCase, first: String| {
        let mut points = vec![first];
        for _ in 0..2 * count(tc, 1, 2) {
            points.push(xy(tc));
        }
        points
    };
    let triangle = |tc: &TestCase| {
        let mut points: Vec<String> = (0..3).map(|_| xy(tc)).collect();
        points.push(points[0].clone());
        list(tc, vec![list(tc, points)])
    };
    // The rules inside these types are in `EXCLUDED_RULES`.
    let body = quietly(|| match kind {
        "CIRCULARSTRING" => {
            let first = xy(tc);
            list(tc, arc(tc, first))
        }
        "COMPOUNDCURVE" => {
            // The segments meet: each segment starts at the end of the last.
            // PostGIS cannot find that infinite points meet, so the shared
            // point is finite.
            let shared = format!(
                "{} {}",
                tc.draw(gs::integers::<i32>()),
                tc.draw(gs::integers::<i32>())
            );
            let line = vec![xy(tc), shared.clone()];
            let curve = arc(tc, shared);
            let items = vec![
                list(tc, line),
                word(tc, "CIRCULARSTRING") + &list(tc, curve),
            ];
            list(tc, items)
        }
        "CURVEPOLYGON" => polygon_body(tc, Dim::Xy),
        "MULTICURVE" => {
            let first = xy(tc);
            let items = vec![
                linestring_body(tc, Dim::Xy),
                word(tc, "CIRCULARSTRING") + &list(tc, arc(tc, first)),
            ];
            list(tc, items)
        }
        "MULTISURFACE" => {
            let items = vec![
                polygon_body(tc, Dim::Xy),
                word(tc, "CURVEPOLYGON") + &polygon_body(tc, Dim::Xy),
            ];
            list(tc, items)
        }
        "POLYHEDRALSURFACE" => {
            let items = (0..count(tc, 1, 3))
                .map(|_| polygon_body(tc, Dim::Xy))
                .collect();
            list(tc, items)
        }
        "TRIANGLE" => triangle(tc),
        _ => {
            let items = (0..count(tc, 1, 3)).map(|_| triangle(tc)).collect();
            list(tc, items)
        }
    });
    text + &body
}

// Text that PostGIS accepts and this crate does not support: a type from
// `UNSUPPORTED_KINDS`, alone or in a collection, or EWKT with an SRID.
fn unsupported_wkt(tc: &TestCase) -> String {
    match tc.draw(gs::integers::<u8>().max_value(2)) {
        0 => {
            production(tc, "geometry: geometry_no_srid");
            unsupported_geometry(tc, false)
        }
        1 => {
            production(tc, "geometry: geometry_no_srid");
            production(tc, "geometry_no_srid: geometrycollection");
            production(
                tc,
                "geometrycollection: COLLECTION_TOK LBRACKET_TOK geometry_list RBRACKET_TOK",
            );
            list_productions(tc, "geometry_list", "geometry_no_srid", 2);
            let mut items = vec![geometry(tc, "POINT", Dim::Xy, false, 0)];
            let at = count(tc, 0, 1);
            items.insert(at, unsupported_geometry(tc, true));
            word(tc, "GEOMETRYCOLLECTION") + ws(tc) + &list(tc, items)
        }
        _ => {
            production(tc, "geometry: SRID_TOK SEMICOLON_TOK geometry_no_srid");
            let srid = tc.draw(gs::integers::<i32>().min_value(0));
            format!("{}SRID={srid}{};{}", ws(tc), ws(tc), valid_geometry(tc))
        }
    }
}

// Every alternative of every rule in the PostGIS WKT grammar,
// `liblwgeom/lwin_wkt_parse.y` at tag 3.5.3, as `rule: symbol symbol ...`.
// `grammar_matches_postgis_source` checks this list against the file.
const GRAMMAR: &[&str] = &[
    "geometry: geometry_no_srid",
    "geometry: SRID_TOK SEMICOLON_TOK geometry_no_srid",
    "geometry_no_srid: point",
    "geometry_no_srid: linestring",
    "geometry_no_srid: circularstring",
    "geometry_no_srid: compoundcurve",
    "geometry_no_srid: polygon",
    "geometry_no_srid: curvepolygon",
    "geometry_no_srid: multipoint",
    "geometry_no_srid: multilinestring",
    "geometry_no_srid: multipolygon",
    "geometry_no_srid: multisurface",
    "geometry_no_srid: multicurve",
    "geometry_no_srid: tin",
    "geometry_no_srid: polyhedralsurface",
    "geometry_no_srid: triangle",
    "geometry_no_srid: geometrycollection",
    "geometrycollection: COLLECTION_TOK LBRACKET_TOK geometry_list RBRACKET_TOK",
    "geometrycollection: COLLECTION_TOK DIMENSIONALITY_TOK LBRACKET_TOK geometry_list RBRACKET_TOK",
    "geometrycollection: COLLECTION_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "geometrycollection: COLLECTION_TOK EMPTY_TOK",
    "geometry_list: geometry_list COMMA_TOK geometry_no_srid",
    "geometry_list: geometry_no_srid",
    "multisurface: MSURFACE_TOK LBRACKET_TOK surface_list RBRACKET_TOK",
    "multisurface: MSURFACE_TOK DIMENSIONALITY_TOK LBRACKET_TOK surface_list RBRACKET_TOK",
    "multisurface: MSURFACE_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "multisurface: MSURFACE_TOK EMPTY_TOK",
    "surface_list: surface_list COMMA_TOK polygon",
    "surface_list: surface_list COMMA_TOK curvepolygon",
    "surface_list: surface_list COMMA_TOK polygon_untagged",
    "surface_list: polygon",
    "surface_list: curvepolygon",
    "surface_list: polygon_untagged",
    "tin: TIN_TOK LBRACKET_TOK triangle_list RBRACKET_TOK",
    "tin: TIN_TOK DIMENSIONALITY_TOK LBRACKET_TOK triangle_list RBRACKET_TOK",
    "tin: TIN_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "tin: TIN_TOK EMPTY_TOK",
    "polyhedralsurface: POLYHEDRALSURFACE_TOK LBRACKET_TOK patch_list RBRACKET_TOK",
    "polyhedralsurface: POLYHEDRALSURFACE_TOK DIMENSIONALITY_TOK LBRACKET_TOK patch_list RBRACKET_TOK",
    "polyhedralsurface: POLYHEDRALSURFACE_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "polyhedralsurface: POLYHEDRALSURFACE_TOK EMPTY_TOK",
    "multipolygon: MPOLYGON_TOK LBRACKET_TOK polygon_list RBRACKET_TOK",
    "multipolygon: MPOLYGON_TOK DIMENSIONALITY_TOK LBRACKET_TOK polygon_list RBRACKET_TOK",
    "multipolygon: MPOLYGON_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "multipolygon: MPOLYGON_TOK EMPTY_TOK",
    "polygon_list: polygon_list COMMA_TOK polygon_untagged",
    "polygon_list: polygon_untagged",
    "patch_list: patch_list COMMA_TOK patch",
    "patch_list: patch",
    "polygon: POLYGON_TOK LBRACKET_TOK ring_list RBRACKET_TOK",
    "polygon: POLYGON_TOK DIMENSIONALITY_TOK LBRACKET_TOK ring_list RBRACKET_TOK",
    "polygon: POLYGON_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "polygon: POLYGON_TOK EMPTY_TOK",
    "polygon_untagged: LBRACKET_TOK ring_list RBRACKET_TOK",
    "polygon_untagged: EMPTY_TOK",
    "patch: LBRACKET_TOK patchring_list RBRACKET_TOK",
    "curvepolygon: CURVEPOLYGON_TOK LBRACKET_TOK curvering_list RBRACKET_TOK",
    "curvepolygon: CURVEPOLYGON_TOK DIMENSIONALITY_TOK LBRACKET_TOK curvering_list RBRACKET_TOK",
    "curvepolygon: CURVEPOLYGON_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "curvepolygon: CURVEPOLYGON_TOK EMPTY_TOK",
    "curvering_list: curvering_list COMMA_TOK curvering",
    "curvering_list: curvering",
    "curvering: linestring_untagged",
    "curvering: linestring",
    "curvering: compoundcurve",
    "curvering: circularstring",
    "patchring_list: patchring_list COMMA_TOK patchring",
    "patchring_list: patchring",
    "ring_list: ring_list COMMA_TOK ring",
    "ring_list: ring",
    "patchring: LBRACKET_TOK ptarray RBRACKET_TOK",
    "ring: LBRACKET_TOK ptarray RBRACKET_TOK",
    "compoundcurve: COMPOUNDCURVE_TOK LBRACKET_TOK compound_list RBRACKET_TOK",
    "compoundcurve: COMPOUNDCURVE_TOK DIMENSIONALITY_TOK LBRACKET_TOK compound_list RBRACKET_TOK",
    "compoundcurve: COMPOUNDCURVE_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "compoundcurve: COMPOUNDCURVE_TOK EMPTY_TOK",
    "compound_list: compound_list COMMA_TOK circularstring",
    "compound_list: compound_list COMMA_TOK linestring",
    "compound_list: compound_list COMMA_TOK linestring_untagged",
    "compound_list: circularstring",
    "compound_list: linestring",
    "compound_list: linestring_untagged",
    "multicurve: MCURVE_TOK LBRACKET_TOK curve_list RBRACKET_TOK",
    "multicurve: MCURVE_TOK DIMENSIONALITY_TOK LBRACKET_TOK curve_list RBRACKET_TOK",
    "multicurve: MCURVE_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "multicurve: MCURVE_TOK EMPTY_TOK",
    "curve_list: curve_list COMMA_TOK circularstring",
    "curve_list: curve_list COMMA_TOK compoundcurve",
    "curve_list: curve_list COMMA_TOK linestring",
    "curve_list: curve_list COMMA_TOK linestring_untagged",
    "curve_list: circularstring",
    "curve_list: compoundcurve",
    "curve_list: linestring",
    "curve_list: linestring_untagged",
    "multilinestring: MLINESTRING_TOK LBRACKET_TOK linestring_list RBRACKET_TOK",
    "multilinestring: MLINESTRING_TOK DIMENSIONALITY_TOK LBRACKET_TOK linestring_list RBRACKET_TOK",
    "multilinestring: MLINESTRING_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "multilinestring: MLINESTRING_TOK EMPTY_TOK",
    "linestring_list: linestring_list COMMA_TOK linestring_untagged",
    "linestring_list: linestring_untagged",
    "circularstring: CIRCULARSTRING_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "circularstring: CIRCULARSTRING_TOK DIMENSIONALITY_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "circularstring: CIRCULARSTRING_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "circularstring: CIRCULARSTRING_TOK EMPTY_TOK",
    "linestring: LINESTRING_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "linestring: LINESTRING_TOK DIMENSIONALITY_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "linestring: LINESTRING_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "linestring: LINESTRING_TOK EMPTY_TOK",
    "linestring_untagged: LBRACKET_TOK ptarray RBRACKET_TOK",
    "linestring_untagged: EMPTY_TOK",
    "triangle_list: triangle_list COMMA_TOK triangle_untagged",
    "triangle_list: triangle_untagged",
    "triangle: TRIANGLE_TOK LBRACKET_TOK LBRACKET_TOK ptarray RBRACKET_TOK RBRACKET_TOK",
    "triangle: TRIANGLE_TOK DIMENSIONALITY_TOK LBRACKET_TOK LBRACKET_TOK ptarray RBRACKET_TOK RBRACKET_TOK",
    "triangle: TRIANGLE_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "triangle: TRIANGLE_TOK EMPTY_TOK",
    "triangle_untagged: LBRACKET_TOK LBRACKET_TOK ptarray RBRACKET_TOK RBRACKET_TOK",
    "multipoint: MPOINT_TOK LBRACKET_TOK point_list RBRACKET_TOK",
    "multipoint: MPOINT_TOK DIMENSIONALITY_TOK LBRACKET_TOK point_list RBRACKET_TOK",
    "multipoint: MPOINT_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "multipoint: MPOINT_TOK EMPTY_TOK",
    "point_list: point_list COMMA_TOK point_untagged",
    "point_list: point_untagged",
    "point_untagged: coordinate",
    "point_untagged: LBRACKET_TOK coordinate RBRACKET_TOK",
    "point_untagged: EMPTY_TOK",
    "point: POINT_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "point: POINT_TOK DIMENSIONALITY_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "point: POINT_TOK DIMENSIONALITY_TOK EMPTY_TOK",
    "point: POINT_TOK EMPTY_TOK",
    "ptarray: ptarray COMMA_TOK coordinate",
    "ptarray: coordinate",
    "coordinate: DOUBLE_TOK DOUBLE_TOK",
    "coordinate: DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK",
    "coordinate: DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK DOUBLE_TOK",
];

// Rules that occur only inside the types that this crate does not support.
// This crate rejects the type name, so the text after it has no effect, and
// the generators do not record these rules.
const EXCLUDED_RULES: [&str; 11] = [
    "compound_list",
    "curvering_list",
    "curvering",
    "curve_list",
    "surface_list",
    "patch_list",
    "patch",
    "patchring_list",
    "patchring",
    "triangle_list",
    "triangle_untagged",
];

// Alternatives that the generators do not make: the types that this crate
// does not support, with a tag and a body. This crate rejects the type name,
// and a tag needs coordinates of the same dimension.
const EXCLUDED_ALTERNATIVES: [&str; 8] = [
    "circularstring: CIRCULARSTRING_TOK DIMENSIONALITY_TOK LBRACKET_TOK ptarray RBRACKET_TOK",
    "compoundcurve: COMPOUNDCURVE_TOK DIMENSIONALITY_TOK LBRACKET_TOK compound_list RBRACKET_TOK",
    "curvepolygon: CURVEPOLYGON_TOK DIMENSIONALITY_TOK LBRACKET_TOK curvering_list RBRACKET_TOK",
    "multicurve: MCURVE_TOK DIMENSIONALITY_TOK LBRACKET_TOK curve_list RBRACKET_TOK",
    "multisurface: MSURFACE_TOK DIMENSIONALITY_TOK LBRACKET_TOK surface_list RBRACKET_TOK",
    "polyhedralsurface: POLYHEDRALSURFACE_TOK DIMENSIONALITY_TOK LBRACKET_TOK patch_list RBRACKET_TOK",
    "triangle: TRIANGLE_TOK DIMENSIONALITY_TOK LBRACKET_TOK LBRACKET_TOK ptarray RBRACKET_TOK RBRACKET_TOK",
    "tin: TIN_TOK DIMENSIONALITY_TOK LBRACKET_TOK triangle_list RBRACKET_TOK",
];

// Forms of the tokens in `liblwgeom/lwin_wkt_lex.l` that the generators
// make. The lexer rules are regular expressions, so this list is written by
// hand. The generators do not make the DOUBLE token `nan`, which this crate
// reads as a word (see `only_nan_word`).
const LEXER_FORMS: &[&str] = &[
    // DOUBLE: -?(([0-9]+\.?)|([0-9]*\.?[0-9]+)([eE][-+]?[0-9]+)?)
    "lexer: DOUBLE: negative",
    "lexer: DOUBLE: digits",
    "lexer: DOUBLE: digits .",
    "lexer: DOUBLE: digits . digits",
    "lexer: DOUBLE: . digits",
    "lexer: DOUBLE: exponent",
    "lexer: DOUBLE: exponent with sign",
    // DIMENSIONALITY: Z|M|ZM
    "lexer: DIMENSIONALITY: Z",
    "lexer: DIMENSIONALITY: M",
    "lexer: DIMENSIONALITY: ZM",
    "lexer: DIMENSIONALITY: after whitespace",
    "lexer: DIMENSIONALITY: attached to the type name",
    // Keywords, which are not case sensitive
    "lexer: keyword: upper case",
    "lexer: keyword: lower case",
    "lexer: keyword: mixed case",
    // Whitespace, [ \t\n\r]+, which is optional between most tokens
    "lexer: whitespace: none",
    "lexer: whitespace: space",
    "lexer: whitespace: tab",
    "lexer: whitespace: newline",
    "lexer: whitespace: carriage return",
];

fn is_excluded(alternative: &str) -> bool {
    let rule = alternative.split(':').next().unwrap_or_default();
    EXCLUDED_RULES.contains(&rule) || EXCLUDED_ALTERNATIVES.contains(&alternative)
}

// Runs the generators in rounds until they have made each alternative in
// `GRAMMAR` that is not excluded, and each form in `LEXER_FORMS`. Some are
// rare, so a fixed number of test cases would fail sometimes.
#[test]
fn generators_cover_the_grammar() {
    let expected: BTreeSet<&str> = GRAMMAR
        .iter()
        .copied()
        .filter(|a| !is_excluded(a))
        .chain(LEXER_FORMS.iter().copied())
        .collect();
    PRODUCTIONS_SEEN.with(|seen| seen.borrow_mut().clear());
    let missing = || -> Vec<&str> {
        PRODUCTIONS_SEEN.with(|seen| {
            let seen = seen.borrow();
            expected
                .iter()
                .copied()
                .filter(|p| !seen.contains(*p))
                .collect()
        })
    };
    for _ in 0..10 {
        hegel::Hegel::new(|tc: TestCase| {
            valid_wkt(&tc);
            unsupported_wkt(&tc);
        })
        .settings(hegel::Settings::new().test_cases(2000))
        .run();
        if missing().is_empty() {
            break;
        }
    }
    // A recorded name that is not expected is a name that is not in the
    // grammar, or an alternative that is excluded but generated.
    let unexpected: Vec<String> = PRODUCTIONS_SEEN.with(|seen| {
        seen.borrow()
            .iter()
            .filter(|name| !expected.contains(name.as_str()))
            .cloned()
            .collect()
    });
    assert!(
        missing().is_empty() && unexpected.is_empty(),
        "never generated: {:#?}\nnot expected: {unexpected:#?}",
        missing()
    );
}

#[test]
fn exclusions_are_in_the_grammar() {
    for rule in EXCLUDED_RULES {
        let prefix = format!("{rule}: ");
        assert!(
            GRAMMAR.iter().any(|a| a.starts_with(&prefix)),
            "no rule {rule} in GRAMMAR"
        );
    }
    for alternative in EXCLUDED_ALTERNATIVES {
        assert!(
            GRAMMAR.contains(&alternative),
            "not in GRAMMAR: {alternative}"
        );
    }
}

// Every alternative of every rule in the rules section of a bison file, as
// `rule: symbol symbol ...`. The actions in braces and the comments are
// removed.
fn bison_alternatives(source: &str) -> Vec<String> {
    let rules = source.split("\n%%").nth(1).expect("a rules section");
    let mut text = String::new();
    let mut depth = 0;
    let mut rest = rules;
    while let Some(c) = rest.chars().next() {
        if depth == 0 && rest.starts_with("/*") {
            let end = rest.find("*/").expect("a comment end");
            rest = &rest[end + 2..];
            continue;
        }
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if depth == 0 => text.push(c),
            _ => {}
        }
        rest = &rest[c.len_utf8()..];
    }
    let mut alternatives = vec![];
    for rule in text.split(';') {
        let Some((name, body)) = rule.split_once(':') else {
            continue;
        };
        for alternative in body.split('|') {
            let symbols: Vec<&str> = alternative.split_whitespace().collect();
            alternatives.push(format!("{}: {}", name.trim(), symbols.join(" ")));
        }
    }
    alternatives
}

#[test]
#[ignore = "needs the PostGIS grammar file; see the module documentation"]
fn grammar_matches_postgis_source() {
    let path = env::var("POSTGIS_GRAMMAR").unwrap_or_else(|_| {
        panic!("set POSTGIS_GRAMMAR to the path of lwin_wkt_parse.y; see the module documentation")
    });
    let source = std::fs::read_to_string(&path).expect("read the grammar file");
    let from_source: BTreeSet<String> = bison_alternatives(&source).into_iter().collect();
    let listed: BTreeSet<String> = GRAMMAR.iter().map(|a| a.to_string()).collect();
    let not_listed: Vec<_> = from_source.difference(&listed).collect();
    let not_in_source: Vec<_> = listed.difference(&from_source).collect();
    assert!(
        not_listed.is_empty() && not_in_source.is_empty(),
        "in the file, not in GRAMMAR: {not_listed:#?}\nin GRAMMAR, not in the file: {not_in_source:#?}"
    );
}

#[hegel::test(test_cases = 2000)]
#[ignore = "needs a PostGIS server; see the module documentation"]
fn valid_wkt_parses_like_postgis(tc: TestCase) {
    let text = valid_wkt(&tc);
    if let Err(message) = postgis_wkb(&text) {
        panic!("the generator made text that PostGIS rejects: {text:?}: {message}");
    }
    assert_same_as_postgis(&tc, &text);
}

#[hegel::test(test_cases = 5000)]
#[ignore = "needs a PostGIS server; see the module documentation"]
fn mutated_wkt_parses_like_postgis(tc: TestCase) {
    let text = mutated_wkt(&tc);
    assert_same_as_postgis(&tc, &text);
}

#[hegel::test(test_cases = 1000)]
#[ignore = "needs a PostGIS server; see the module documentation"]
fn unsupported_wkt_is_rejected(tc: TestCase) {
    let text = unsupported_wkt(&tc);
    if let Err(message) = postgis_wkb(&text) {
        panic!("the generator made text that PostGIS rejects: {text:?}: {message}");
    }
    let (ours, _) = parse_with_crate(&text);
    assert_eq!(ours, Outcome::Rejected, "input: {text:?}");
}

// The greatest number of nested collections that this crate reads. It
// rejects deeper nesting to prevent a stack overflow.
const MAX_DEPTH: usize = 128;

#[hegel::test(test_cases = 300)]
#[ignore = "needs a PostGIS server; see the module documentation"]
fn nested_collections_parse_like_postgis(tc: TestCase) {
    let depth = if tc.draw(gs::booleans()) {
        count(&tc, MAX_DEPTH - 3, MAX_DEPTH + 3)
    } else {
        count(&tc, 1, 2 * MAX_DEPTH)
    };
    let kind = tc.draw(gs::sampled_from(&KINDS[..6]));
    let inner = geometry(&tc, kind, Dim::Xy, false, 0);
    let text = format!(
        "{}{inner}{}",
        "GEOMETRYCOLLECTION(".repeat(depth),
        ")".repeat(depth)
    );
    let (ours, error) = parse_with_crate(&text);
    if depth <= MAX_DEPTH {
        assert_same_as_postgis(&tc, &text);
    } else {
        assert!(
            matches!(error, Some(ParseError::MaxDepthExceeded { .. })),
            "depth {depth}: {ours:?}, {error:?}"
        );
    }
}

#[test]
fn tokens_examples() {
    assert_eq!(
        tokens("POINT Z(1 2,3)"),
        ["POINT", " ", "Z", "(", "1", " ", "2", ",", "3", ")"]
    );
    assert_eq!(tokens("((a  b))").concat(), "((a  b))");
}

#[test]
fn respell_numbers_examples() {
    assert_eq!(respell_numbers("POINT(+1 0.E5)"), "POINT(1.0 0.0)");
    assert_eq!(respell_numbers("POINT(-1e400 -nan)"), "POINT(-1e999 nan)");
    assert_eq!(respell_numbers("POINT Z EMPTY"), "POINT Z EMPTY");
}

#[test]
fn separate_empty_examples() {
    assert_eq!(separate_empty("POINTEMPTY"), "POINT EMPTY");
    assert_eq!(separate_empty("POINT ZMempty"), "POINT ZM empty");
    assert_eq!(separate_empty("POINT EMPTY"), "POINT EMPTY");
}

#[test]
fn add_tags_examples() {
    assert_eq!(add_tags("POINT(1 2 3)", "Z"), "POINT Z(1 2 3)");
    assert_eq!(add_tags("POINT Z(1 2 3)", "Z"), "POINT Z(1 2 3)");
    assert_eq!(add_tags("POINTZ(1 2 3)", "Z"), "POINTZ(1 2 3)");
    assert_eq!(
        add_tags("GEOMETRYCOLLECTION z(point EMPTY,POINT(1e5 2 3))", "Z"),
        "GEOMETRYCOLLECTION z(point Z EMPTY,POINT Z(1e5 2 3))"
    );
}
