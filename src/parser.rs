use nom::{self, multispace, IResult, digit};
use std::str::FromStr;
use std::str::from_utf8;

#[derive(Debug, PartialEq)]
struct Point(Option<Coord>);

#[derive(Debug, PartialEq)]
struct Coord(f64, f64);

// FIXME: this doesn't properly parse floating points
// number = ?/[+-]?(\d+(\.\d*)?|\.\d+)([eE][+-]?\d+)?/? ;
fn is_numeric_byte(c: u8) -> bool {
    match c as char {
        '0'...'9' | '.' | 'e' | 'E' | '+' | '-' => true,
        _ => false,
    }
}
named!(number<&[u8], f64>, map!(
    take_while!(is_numeric_byte),
    |x| { FromStr::from_str(from_utf8(x).unwrap()).unwrap() }
));

// left_paren = "(";
named!(left_paren<&[u8], &[u8]>, delimited!(
    opt!(multispace), tag!("("), opt!(multispace)
));

// right_paren = ")";
named!(right_paren<&[u8], &[u8]>, delimited!(
    opt!(multispace), tag!(")"), opt!(multispace)
));

// empty_set = "EMPTY";
fn empty_set<T>(input: &[u8]) -> IResult<&[u8], Option<T>> {
    map!(input, tag!("EMPTY"), |_| { None })
}

// POINT

// point_text_representation = "POINT" [ z_m ] point_text;
named!(point_text_representation<&[u8], Point>, chain!(
    tag!("POINT") ~
    multispace ~
    coord: point_text ,
    || { Point(coord) }
));

// point_text =
//     empty_set |
//     left_paren point right_paren;
named!(point_text<&[u8], Option<Coord> >, alt!(
    empty_set |
    delimited!(left_paren, point, right_paren)
));

// point = x y [ z ] [ m ];
named!(point<&[u8], Option<Coord> >, chain!(
    x: number ~
    multispace ~
    y: number ,
    || { Some(Coord(x, y)) }
));

// LINESTRING

// empty_set = "EMPTY";
named!(empty_set_linestring<&[u8], Option<Vec<Coord>> >, map!(
    tag!("EMPTY"), |_| { None }
));

// linestring_text =
//     empty_set |
//     left_paren point { comma point } right_paren;
/*
named!(linestring_text<&[u8], Option<Vec<Coord>> >, alt!(
    empty_set_linestring |
    delimited!(
        left_paren,
        separated_nonempty_list!(char!(','), expr_opt!(point)),
        right_paren
    )
));
*/

// linestring_text_representation =
//     "LINESTRING" [ z_m ] linestring_text_body;
/*
named!(point_text_representation<&[u8], Point>, chain!(
    tag!("LINESTRING") ~
    multispace ~
    coord: point_text ,
    || { Point(coord) }
));
*/


#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::{point_text_representation, Point, Coord};

    #[test]
    fn test_empty_point() {
        let input = b"POINT EMPTY";
        let point = point_text_representation(input);
        assert_eq!(IResult::Done(b"" as &[u8], Point(None)), point);
    }

    #[test]
    fn test_point() {
        let input = b"POINT (1.9 2)";
        let point = point_text_representation(input);
        assert_eq!(
            IResult::Done(b"" as &[u8], Point(Some(Coord(1.9, 2.)))),
            point);
    }
}
