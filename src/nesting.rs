//! Bounded nesting of recursive triple terms.

use crate::ModelError;

/// The deepest triple-term nesting accepted by decoding and by
/// generalized-to-standard conversion.
///
/// A top-level term has depth 0 and each enclosing triple term adds one level,
/// so `<< << <s> <p> <o> >> <p> <o> >>` has depth 2. RDF 1.2 data nests triple
/// terms a few levels deep in practice; 32 leaves ample room while keeping the
/// recursive decode well inside a default 2 MiB thread stack, even in debug
/// builds. It also stays below `serde_json`'s own recursion limit, so JSON
/// input fails with this crate's typed [`ModelError::NestingTooDeep`] first.
pub const MAX_TRIPLE_TERM_DEPTH: NestingDepth = NestingDepth::new(32);

/// A triple-term nesting depth: the number of enclosing triple terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NestingDepth(usize);

impl NestingDepth {
    /// The depth of a top-level term.
    pub const ROOT: Self = Self(0);

    /// Wrap a raw nesting depth.
    #[must_use]
    pub const fn new(depth: usize) -> Self {
        Self(depth)
    }
    /// The raw nesting depth.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
    /// Enter one more triple term, rejecting depths beyond
    /// [`MAX_TRIPLE_TERM_DEPTH`].
    ///
    /// # Errors
    ///
    /// [`ModelError::NestingTooDeep`] when the nested depth exceeds the limit.
    pub const fn descend(self) -> Result<Self, ModelError> {
        let depth = Self(self.0.saturating_add(1));
        if depth.0 > MAX_TRIPLE_TERM_DEPTH.0 {
            return Err(ModelError::NestingTooDeep {
                limit: MAX_TRIPLE_TERM_DEPTH,
                depth,
            });
        }
        Ok(depth)
    }
}

impl core::fmt::Display for NestingDepth {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(formatter)
    }
}
