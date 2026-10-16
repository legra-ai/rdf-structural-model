//! RDF base direction.

/// Base text direction of an RDF directional language string.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum BaseDirection {
    /// Left-to-right presentation.
    Ltr,
    /// Right-to-left presentation.
    Rtl,
}

impl BaseDirection {
    /// The RDF syntax direction token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}
