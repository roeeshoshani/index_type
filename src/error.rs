#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericIndexTooBigError;
impl core::fmt::Display for GenericIndexTooBigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "index too big")
    }
}

impl core::error::Error for GenericIndexTooBigError {}
