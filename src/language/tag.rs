//! The language-tag value: validation, accessors and Serde.

use std::hash::{
    Hash,
    Hasher,
};

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use super::parse::{
    LanguageTagParts,
    TagParser,
};
use crate::ModelError;

/// A well-formed language tag backed by owned or borrowed text.
///
/// Spelling is preserved; equality, hashing and ordering are ASCII
/// case-insensitive. Validation is syntactic (RFC 5646 §2.1, plus unique
/// variants and extension singletons) and does not consult the IANA
/// registration database.
#[derive(Debug, Clone, Copy)]
pub struct LanguageTag<Text = String>(Text);

impl<Text: AsRef<str>> LanguageTag<Text> {
    /// Validate a BCP 47 tag, including unique variants and extension
    /// singletons.
    ///
    /// # Errors
    /// Returns [`ModelError::InvalidLanguageTag`] naming the reason and the
    /// failing subtag position.
    pub fn try_new(text: Text) -> Result<Self, ModelError> {
        TagParser::parse(text.as_ref()).map_err(ModelError::InvalidLanguageTag)?;
        Ok(Self(text))
    }

    /// Original language-tag spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Borrow the tag without allocating or reparsing.
    #[must_use]
    pub fn as_ref(&self) -> LanguageTag<&str> {
        LanguageTag(self.as_str())
    }

    /// The primary language subtag in its original spelling; `None` for
    /// private-use and irregular grandfathered tags.
    #[must_use]
    pub fn primary(&self) -> Option<&str> {
        self.parts().primary
    }

    /// The four-letter script subtag, if present.
    #[must_use]
    pub fn script(&self) -> Option<&str> {
        self.parts().script
    }

    /// The two-letter or three-digit region subtag, if present.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.parts().region
    }

    /// The variant subtags in order.
    pub fn variants(&self) -> impl Iterator<Item = &str> {
        let (start, count) = self.parts().variants;
        self.as_str().split('-').skip(start).take(count)
    }

    /// The tag was validated on construction, so parsing cannot fail here.
    fn parts(&self) -> LanguageTagParts<'_> {
        TagParser::parse(self.as_str()).unwrap_or_default()
    }

    /// The ASCII-lowercased bytes that define equality, hashing and order.
    pub(super) fn folded_bytes(&self) -> impl Iterator<Item = u8> {
        self.as_str().bytes().map(|byte| byte.to_ascii_lowercase())
    }
}

impl<Text: AsRef<str>> PartialEq for LanguageTag<Text> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq_ignore_ascii_case(other.as_str())
    }
}
impl<Text: AsRef<str>> Eq for LanguageTag<Text> {}
impl<Text: AsRef<str>> Hash for LanguageTag<Text> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().len().hash(state);
        for byte in self.folded_bytes() {
            byte.hash(state);
        }
    }
}
impl<Text: AsRef<str>> Serialize for LanguageTag<Text> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de, Text: Deserialize<'de> + AsRef<str>> Deserialize<'de> for LanguageTag<Text> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_new(Text::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
