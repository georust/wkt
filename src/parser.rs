use nom::{self, multispace, IResult, digit};
use std::str::FromStr;
use std::str::from_utf8;

#[derive(Debug, PartialEq)]
struct Point(Coord);

#[derive(Debug, PartialEq)]
enum Coord {
    Empty,
    XY(f64, f64),
}

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
named!(empty_set<&[u8], Coord>, map!(
    tag!("EMPTY"), |_| { Coord::Empty }
));

// point_text_representation = "POINT" [ z_m ] point_text;
named!(point_text_representation<&[u8], Point>, chain!(
    tag!("POINT") ~
    multispace ~
    coord: point_text ,
    || { Point(coord) }
));

// point_text =
//    empty_set |
//    left_paren point right_paren;
named!(point_text<&[u8], Coord>, alt!(
    empty_set |
    delimited!(left_paren, point, right_paren)
));

// point = x y [ z ] [ m ];
named!(point<&[u8], Coord>, chain!(
    x: number ~
    multispace ~
    y: number ,
    || { Coord::XY(x, y) }
));

// linestring_text =
//   empty_set |
//   left_paren point { comma point } right_paren;


#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::{point_text_representation, Point, Coord};

    #[test]
    fn test_empty_point() {
        let input = b"POINT EMPTY";
        let point = point_text_representation(input);
        assert_eq!(IResult::Done(b"" as &[u8], Point(Coord::Empty)), point);
    }

    #[test]
    fn test_point() {
        let input = b"POINT (1.9 2)";
        let point = point_text_representation(input);
        assert_eq!(
            IResult::Done(b"" as &[u8], Point(Coord::XY(1.9, 2.))),
            point);
    }
}
