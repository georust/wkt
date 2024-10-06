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
