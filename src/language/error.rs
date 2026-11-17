//! Typed language-tag rejection reasons that name the failing subtag.

/// The zero-based position of a subtag within a hyphen-separated tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubtagIndex(usize);

impl SubtagIndex {
    /// Wrap a zero-based subtag position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    /// The zero-based subtag position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl core::fmt::Display for SubtagIndex {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Why an [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646#section-2.1) tag
/// is not well-formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageTagErrorReason {
    /// A subtag is empty: the tag is empty or has adjacent, leading or
    /// trailing hyphens.
    EmptySubtag,
    /// A subtag contains a character other than an ASCII letter or digit.
    InvalidCharacter,
    /// A subtag is longer than eight characters, the limit for every subtag,
    /// including private-use subtags.
    SubtagTooLong,
    /// The first subtag is neither a 2-8 letter language nor `x`.
    InvalidPrimaryLanguage,
    /// A subtag does not fit the grammar at its position.
    UnexpectedSubtag,
    /// A variant subtag repeats an earlier variant.
    DuplicateVariant,
    /// An extension singleton repeats an earlier singleton.
    DuplicateExtensionSingleton,
    /// An extension singleton has no 2-8 character subtag after it.
    EmptyExtension,
    /// The private-use singleton `x` has no subtag after it.
    EmptyPrivateUse,
}

impl LanguageTagErrorReason {
    /// A short human-readable description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::EmptySubtag => "empty subtag",
            Self::InvalidCharacter => "subtag contains a non-alphanumeric character",
            Self::SubtagTooLong => "subtag is longer than eight characters",
            Self::InvalidPrimaryLanguage => "invalid primary language subtag",
            Self::UnexpectedSubtag => "subtag does not fit its position",
            Self::DuplicateVariant => "duplicate variant subtag",
            Self::DuplicateExtensionSingleton => "duplicate extension singleton",
            Self::EmptyExtension => "extension singleton without subtags",
            Self::EmptyPrivateUse => "private-use singleton without subtags",
        }
    }
}

/// A malformed language tag: the reason and the subtag where it was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("language tag is not well-formed BCP 47: {} at subtag {subtag}", reason.description())]
pub struct LanguageTagError {
    reason: LanguageTagErrorReason,
    subtag: SubtagIndex,
}

impl LanguageTagError {
    /// Report `reason` at subtag position `subtag`.
    #[must_use]
    pub const fn new(reason: LanguageTagErrorReason, subtag: SubtagIndex) -> Self {
        Self { reason, subtag }
    }
    /// Why the tag was rejected.
    #[must_use]
    pub const fn reason(self) -> LanguageTagErrorReason {
        self.reason
    }
    /// The zero-based position of the failing subtag.
    #[must_use]
    pub const fn subtag(self) -> SubtagIndex {
        self.subtag
    }
}
