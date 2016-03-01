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
    parenthesized
));

// point = x y [ z ] [ m ];
named!(point<&[u8], Coord>, chain!(
    x: digit ~
    multispace ~
    y: digit ,
    || { Coord::XY(FromStr::from_str(from_utf8(x).unwrap()).unwrap(),
                   FromStr::from_str(from_utf8(y).unwrap()).unwrap()) }
));

named!(parenthesized<&[u8], Coord>, delimited!(
    delimited!(opt!(multispace), tag!("("), opt!(multispace)),
    point,
    delimited!(opt!(multispace), tag!(")"), opt!(multispace))
));

named!(number<&[u8], Coord>, delimited!(
    delimited!(opt!(multispace), tag!("("), opt!(multispace)),
    point,
    delimited!(opt!(multispace), tag!(")"), opt!(multispace))
));


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
        let input = b"POINT (1 2)";
        let point = point_text_representation(input);
        assert_eq!(
            IResult::Done(b"" as &[u8], Point(Coord::XY(1., 2.))),
            point);
    }
}
