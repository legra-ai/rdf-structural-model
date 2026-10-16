//! Exact lexical literals and storage-independent literal metadata.

use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use crate::{
    BaseDirection,
    LanguageTag,
    ModelError,
};

/// The RDF language-string datatype IRI.
pub const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
/// The RDF directional language-string datatype IRI.
pub const RDF_DIR_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";
/// The XML Schema string datatype IRI.
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// Language and direction without imposing a storage representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(deserialize = "Text: serde::Deserialize<'de>")
)]
pub enum LiteralFacets<Text: AsRef<str> = String> {
    /// Neither language nor direction.
    None,
    /// A well-formed language tag.
    Language(LanguageTag<Text>),
    /// A language tag and its base direction.
    Directional {
        /// The well-formed language tag.
        language: LanguageTag<Text>,
        /// The base presentation direction.
        direction: BaseDirection,
    },
}

/// Literal metadata independent of payload access and storage addresses.
///
/// Extended literals may carry language facets independently of datatype.
/// [`Literal::try_new`] separately enforces the standard RDF combinations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>"
    )
)]
pub struct LiteralMetadata<Text: Bos<str> + AsRef<str> = String> {
    datatype: Iri<Text>,
    facets: LiteralFacets<Text>,
}

impl<Text: Bos<str> + AsRef<str>> LiteralMetadata<Text> {
    /// Construct metadata, including explicitly extended datatype/facet
    /// combinations.
    #[must_use]
    pub const fn new(datatype: Iri<Text>, facets: LiteralFacets<Text>) -> Self {
        Self { datatype, facets }
    }

    /// The declared datatype IRI.
    #[must_use]
    pub const fn datatype(&self) -> &Iri<Text> {
        &self.datatype
    }

    /// Language and direction independent of payload access.
    #[must_use]
    pub const fn facets(&self) -> &LiteralFacets<Text> {
        &self.facets
    }

    fn validate_standard(&self) -> Result<(), ModelError> {
        let valid = match &self.facets {
            LiteralFacets::None => {
                self.datatype.as_str() != RDF_LANG_STRING
                    && self.datatype.as_str() != RDF_DIR_LANG_STRING
            }
            LiteralFacets::Language(_) => self.datatype.as_str() == RDF_LANG_STRING,
            LiteralFacets::Directional { .. } => self.datatype.as_str() == RDF_DIR_LANG_STRING,
        };
        if !valid {
            return Err(ModelError::InconsistentLiteralFacets);
        }
        Ok(())
    }
}

/// A standard RDF literal with an exact UTF-8 lexical form.
///
/// Equality is RDF term equality, not datatype value equality. Ill-typed
/// lexical forms are representable; evaluating datatype lexical spaces is a
/// separate task.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    try_from = "LiteralInput<Text>",
    bound(
        serialize = "Text: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>"
    )
)]
pub struct Literal<Text: Bos<str> + AsRef<str> = String> {
    lexical: Text,
    metadata: LiteralMetadata<Text>,
}

impl<Text: Bos<str> + AsRef<str>> Literal<Text> {
    /// Validate standard RDF datatype/facet combinations without evaluating
    /// lexical values.
    ///
    /// # Errors
    /// Returns [`ModelError::InconsistentLiteralFacets`] for inconsistent
    /// metadata.
    pub fn try_new(lexical: Text, metadata: LiteralMetadata<Text>) -> Result<Self, ModelError> {
        metadata.validate_standard()?;
        Ok(Self { lexical, metadata })
    }

    /// The exact lexical form, without value normalization.
    #[must_use]
    pub fn lexical(&self) -> &str {
        self.lexical.as_ref()
    }

    /// Metadata accessible without copying the lexical payload.
    #[must_use]
    pub const fn metadata(&self) -> &LiteralMetadata<Text> {
        &self.metadata
    }
}

impl Literal<String> {
    /// Construct a non-language typed literal.
    ///
    /// # Errors
    /// Language datatypes require their corresponding facets and are rejected
    /// here.
    pub fn typed(lexical: impl Into<String>, datatype: Iri<String>) -> Result<Self, ModelError> {
        Self::try_new(
            lexical.into(),
            LiteralMetadata::new(datatype, LiteralFacets::None),
        )
    }

    /// Construct an XML Schema string literal.
    #[must_use]
    pub fn string(lexical: impl Into<String>) -> Self {
        Self {
            lexical: lexical.into(),
            metadata: LiteralMetadata::new(Self::builtin(XSD_STRING), LiteralFacets::None),
        }
    }

    /// Construct a language-tagged literal from a validated tag.
    #[must_use]
    pub fn language_tagged(lexical: impl Into<String>, language: LanguageTag) -> Self {
        Self {
            lexical: lexical.into(),
            metadata: LiteralMetadata::new(
                Self::builtin(RDF_LANG_STRING),
                LiteralFacets::Language(language),
            ),
        }
    }

    /// Construct a directional language-tagged literal from a validated tag.
    #[must_use]
    pub fn directional(
        lexical: impl Into<String>,
        language: LanguageTag,
        direction: BaseDirection,
    ) -> Self {
        Self {
            lexical: lexical.into(),
            metadata: LiteralMetadata::new(
                Self::builtin(RDF_DIR_LANG_STRING),
                LiteralFacets::Directional {
                    language,
                    direction,
                },
            ),
        }
    }

    fn builtin(value: &'static str) -> Iri<String> {
        Iri::parse(value)
            .expect("built-in datatype IRI is valid")
            .into()
    }

    /// Borrow the lexical form and metadata without copying their text.
    #[must_use]
    pub fn as_ref(&self) -> Literal<&str> {
        Literal {
            lexical: &self.lexical,
            metadata: self.metadata.as_ref(),
        }
    }
}

impl LiteralMetadata<String> {
    /// Borrow metadata without copying its text or accessing a payload.
    #[must_use]
    pub fn as_ref(&self) -> LiteralMetadata<&str> {
        let facets = match &self.facets {
            LiteralFacets::None => LiteralFacets::None,
            LiteralFacets::Language(tag) => LiteralFacets::Language(tag.as_ref()),
            LiteralFacets::Directional {
                language,
                direction,
            } => LiteralFacets::Directional {
                language: language.as_ref(),
                direction: *direction,
            },
        };
        LiteralMetadata::new(self.datatype.borrow(), facets)
    }
}

#[derive(serde::Deserialize)]
#[serde(
    rename = "Literal",
    deny_unknown_fields,
    bound(deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>")
)]
struct LiteralInput<Text: Bos<str> + AsRef<str>> {
    lexical: Text,
    metadata: LiteralMetadata<Text>,
}

impl<Text: Bos<str> + AsRef<str>> TryFrom<LiteralInput<Text>> for Literal<Text> {
    type Error = ModelError;
    fn try_from(input: LiteralInput<Text>) -> Result<Self, Self::Error> {
        Self::try_new(input.lexical, input.metadata)
    }
}
