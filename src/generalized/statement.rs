//! Individual generalized statements and graph positions.
use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use super::GeneralizedTerm;
use crate::NoExtension;

/// A generalized triple, allowing generalized terms in every position.
///
/// This models the explicit generalization described by RDF 1.2 Concepts §7.2,
/// not a standard RDF triple. Literal and blank-node identity remain exact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub struct GeneralizedTriple<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    subject: GeneralizedTerm<Text, Extension>,
    predicate: GeneralizedTerm<Text, Extension>,
    object: GeneralizedTerm<Text, Extension>,
}

impl<Text: Bos<str> + AsRef<str>, Extension> GeneralizedTriple<Text, Extension> {
    /// Construct a generalized triple without imposing standard position rules.
    #[must_use]
    pub const fn new(
        subject: GeneralizedTerm<Text, Extension>,
        predicate: GeneralizedTerm<Text, Extension>,
        object: GeneralizedTerm<Text, Extension>,
    ) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }
    /// The exact subject term.
    #[must_use]
    pub const fn subject(&self) -> &GeneralizedTerm<Text, Extension> {
        &self.subject
    }
    /// The exact predicate term.
    #[must_use]
    pub const fn predicate(&self) -> &GeneralizedTerm<Text, Extension> {
        &self.predicate
    }
    /// The exact object term.
    #[must_use]
    pub const fn object(&self) -> &GeneralizedTerm<Text, Extension> {
        &self.object
    }
    /// Consume this value without cloning any term or lexical payload.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        GeneralizedTerm<Text, Extension>,
        GeneralizedTerm<Text, Extension>,
        GeneralizedTerm<Text, Extension>,
    ) {
        (self.subject, self.predicate, self.object)
    }
}

/// A generalized graph position, including an explicit default graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub enum GeneralizedGraphName<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    /// The unnamed default graph.
    Default,
    /// Any generalized term used as an explicit graph name.
    Term(GeneralizedTerm<Text, Extension>),
}

/// One generalized quad, with no graph or dataset container.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub struct GeneralizedQuad<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    triple: GeneralizedTriple<Text, Extension>,
    graph: GeneralizedGraphName<Text, Extension>,
}

impl<Text: Bos<str> + AsRef<str>, Extension> GeneralizedQuad<Text, Extension> {
    /// Associate a generalized triple with its explicit graph position.
    #[must_use]
    pub const fn new(
        triple: GeneralizedTriple<Text, Extension>,
        graph: GeneralizedGraphName<Text, Extension>,
    ) -> Self {
        Self { triple, graph }
    }
    /// The triple in this graph.
    #[must_use]
    pub const fn triple(&self) -> &GeneralizedTriple<Text, Extension> {
        &self.triple
    }
    /// The graph position.
    #[must_use]
    pub const fn graph(&self) -> &GeneralizedGraphName<Text, Extension> {
        &self.graph
    }
    /// Consume the quad without cloning its terms or graph name.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        GeneralizedTriple<Text, Extension>,
        GeneralizedGraphName<Text, Extension>,
    ) {
        (self.triple, self.graph)
    }
}
