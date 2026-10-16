//! RDF term categories and explicitly opted-in extensions.

use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use crate::{
    BlankNodeId,
    Literal,
    ModelError,
    Triple,
};

/// The standard profile's uninhabited extension type.
///
/// Deserialization rejects any extension payload. An application must choose a
/// concrete extension type to opt into additional term semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NoExtension {}

/// An RDF term with owned or borrowed text and an explicit extension parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub enum Term<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    /// An absolute IRI.
    Iri(Iri<Text>),
    /// A scoped blank node.
    BlankNode(BlankNodeId<Text>),
    /// A standard lexical literal.
    Literal(Literal<Text>),
    /// A recursively nested RDF 1.2 triple term.
    Triple(Box<Triple<Text, Extension>>),
    /// An application-selected typed extension; unavailable in the standard
    /// profile.
    Extension(Extension),
}

impl<Text: Bos<str> + AsRef<str>, Extension> Term<Text, Extension> {
    /// Construct an IRI term.
    #[must_use]
    pub const fn iri(iri: Iri<Text>) -> Self {
        Self::Iri(iri)
    }
    /// Construct a literal term.
    #[must_use]
    pub const fn literal(literal: Literal<Text>) -> Self {
        Self::Literal(literal)
    }
    /// Construct a nested triple term without asserting it in a graph.
    #[must_use]
    pub fn triple(triple: Triple<Text, Extension>) -> Self {
        Self::Triple(Box::new(triple))
    }
}

/// A standard subject, or an explicitly selected subject extension.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub enum Subject<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    /// An IRI subject.
    Iri(Iri<Text>),
    /// A scoped blank-node subject.
    BlankNode(BlankNodeId<Text>),
    /// An application-selected extension of standard subject positions.
    Extension(Extension),
}

impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<Term<Text, Extension>>
    for Subject<Text, Extension>
{
    type Error = ModelError;
    fn try_from(term: Term<Text, Extension>) -> Result<Self, Self::Error> {
        match term {
            Term::Iri(iri) => Ok(Self::Iri(iri)),
            Term::BlankNode(blank) => Ok(Self::BlankNode(blank)),
            Term::Extension(extension) => Ok(Self::Extension(extension)),
            Term::Literal(_) | Term::Triple(_) => Err(ModelError::InvalidSubject),
        }
    }
}
