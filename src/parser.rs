extern crate nom;
use self::nom::{
    branch::alt,
    bytes::complete::is_not,
    bytes::complete::{tag, take_while_m_n},
    character::complete::char,
    combinator::map_res,
    multi::separated_list1,
    number::complete::double,
    sequence::delimited,
    sequence::tuple,
    IResult,
};

use types::{Coord, Point};

// return byte array between two parens
fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')'))(input)
}

// separator between coordinates and z and m values
fn separator(input: &str) -> IResult<&str, &str> {
    tag(" ")(input)
}

fn coord_tag(input: &str) -> IResult<&str, &str> {
    alt((tag("COORD"), tag("coord")))(input)
}

fn point_tag(input: &str) -> IResult<&str, &str> {
    alt((tag("POINT"), tag("point")))(input)
}

// Z, M, ZM preceding opening paren in POINT
fn point_tag_prefix(input: &str) -> IResult<&str, &str> {
    alt((tag("Z"), tag("M"), tag("ZM")))(input)
}

// consume a sequence of separator-separated coordinates
fn coordinate_sequence(s: &str) -> IResult<&str, Vec<f64>> {
    separated_list1(separator, double)(s)
}
