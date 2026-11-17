//! The rank of a value's variant within its position.

/// A variant's rank; values of different kinds order by this rank alone.
pub(super) trait PositionKind {
    /// The variant rank, in declaration order.
    fn kind_rank(&self) -> u8;
}
