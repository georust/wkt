/// The dimension of geometry that we're parsing.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dimension {
    #[default]
    XY,
    XYZ,
    XYM,
    XYZM,
}

impl From<Dimension> for geo_traits::Dimensions {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::XY => Self::Xy,
            Dimension::XYZ => Self::Xyz,
            Dimension::XYM => Self::Xym,
            Dimension::XYZM => Self::Xyzm,
        }
    }
}
