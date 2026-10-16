//! Well-formed, case-insensitive BCP 47 language tags.

use std::hash::{
    Hash,
    Hasher,
};

use langtag::LangTag;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::ModelError;

/// A well-formed language tag backed by owned or borrowed text.
///
/// Spelling is preserved; equality and hashing are ASCII case-insensitive.
/// Validation is syntactic and does not consult the IANA registration database.
#[derive(Debug, Clone, Copy)]
pub struct LanguageTag<Text = String>(Text);

impl<Text: AsRef<str>> LanguageTag<Text> {
    /// Validate a BCP 47 tag, including unique variants and extension
    /// singletons.
    ///
    /// # Errors
    /// Returns [`ModelError::InvalidLanguageTag`] for malformed or duplicate
    /// subtags.
    pub fn try_new(text: Text) -> Result<Self, ModelError> {
        LangTag::new(text.as_ref()).map_err(|_| ModelError::InvalidLanguageTag)?;
        Self::validate_unique_subtags(text.as_ref())?;
        Ok(Self(text))
    }

    fn validate_unique_subtags(tag: &str) -> Result<(), ModelError> {
        if tag
            .split('-')
            .next()
            .is_some_and(|primary| primary.eq_ignore_ascii_case("x"))
        {
            return Ok(());
        }
        for (index, subtag) in tag.split('-').enumerate().skip(1) {
            if subtag.eq_ignore_ascii_case("x") {
                break;
            }
            let variant =
                subtag.len() >= 5 || (subtag.len() == 4 && subtag.as_bytes()[0].is_ascii_digit());
            let singleton = subtag.len() == 1;
            if (variant || singleton)
                && tag
                    .split('-')
                    .take(index)
                    .skip(1)
                    .any(|previous| previous.eq_ignore_ascii_case(subtag))
            {
                return Err(ModelError::InvalidLanguageTag);
            }
            // Extension payloads are not variants. Their subtags may repeat.
            if singleton {
                return Self::validate_extension_singletons(tag, index);
            }
        }
        Ok(())
    }

    fn validate_extension_singletons(tag: &str, start: usize) -> Result<(), ModelError> {
        for (index, subtag) in tag.split('-').enumerate().skip(start) {
            if subtag.eq_ignore_ascii_case("x") {
                break;
            }
            if subtag.len() == 1
                && tag
                    .split('-')
                    .take(index)
                    .skip(start)
                    .any(|previous| previous.eq_ignore_ascii_case(subtag))
            {
                return Err(ModelError::InvalidLanguageTag);
            }
        }
        Ok(())
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
        for byte in self.as_str().bytes() {
            byte.to_ascii_lowercase().hash(state);
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
