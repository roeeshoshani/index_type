#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexTooBigError;
impl core::fmt::Display for IndexTooBigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "index too big")
    }
}
