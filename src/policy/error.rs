#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    Overflow,
    NotAtEnd,
    RetentionActive,
    ImmutablePath,
    ReadOnlyMetadata,
    ReservedName,
    InvalidName,
}
