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

#[cfg(feature = "geo-traits_0_2")]
impl From<Dimension> for geo_traits_0_2::Dimensions {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::XY => Self::Xy,
            Dimension::XYZ => Self::Xyz,
            Dimension::XYM => Self::Xym,
            Dimension::XYZM => Self::Xyzm,
        }
    }
}

#[cfg(feature = "geo-traits_0_3")]
impl From<Dimension> for geo_traits_0_3::Dimensions {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::XY => Self::Xy,
            Dimension::XYZ => Self::Xyz,
            Dimension::XYM => Self::Xym,
            Dimension::XYZM => Self::Xyzm,
        }
    }
}
