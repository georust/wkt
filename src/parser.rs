use nom::{self, multispace, IResult};

#[derive(Debug, PartialEq, Eq)]
struct Point(Coord);

#[derive(Debug, PartialEq, Eq)]
enum Coord {
    Empty,
    XY(f64),
}

const EMPTY_SET: &'static str = "EMPTY";

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
    map!(tag!(EMPTY_SET), |_| { Coord::Empty }) |
    parenthesized
));

// point = x y [ z ] [ m ];
named!(point<&[u8], Coord>, chain!(
    map!(tag!("HI"), |_| { Coord::Empty }) ,
    || { Coord::XY(1f64) }
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

/*
    #[test]
    fn test_empty_point() {
        let input = b"[abcdabcd]";
        let point = brackets(input);
        assert_eq!(IResult::Done(b"" as &[u8], Point{x: 0, y: 0}), point);
    }
    */
}
