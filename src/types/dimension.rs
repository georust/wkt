/// The dimension of geometry that we're parsing.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default)]
pub enum Dimension {
    #[default]
    XY,
    XYZ,
    XYM,
    XYZM,
}
