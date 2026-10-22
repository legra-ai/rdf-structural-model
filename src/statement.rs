//! Individual triples and quads, without an in-memory graph container.

use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use crate::{
    BlankNodeId,
    NoExtension,
    Subject,
    Term,
};

/// One RDF triple. Predicates are always IRIs; standard subjects cannot be
/// literals or triple terms.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub struct Triple<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    subject: Subject<Text, Extension>,
    predicate: Iri<Text>,
    object: Term<Text, Extension>,
}

impl<Text: Bos<str> + AsRef<str>, Extension> Triple<Text, Extension> {
    /// Construct a triple from position-typed operands.
    #[must_use]
    pub const fn new(
        subject: Subject<Text, Extension>,
        predicate: Iri<Text>,
        object: Term<Text, Extension>,
    ) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }
    /// Consume the triple without cloning any operand.
    #[must_use]
    pub fn into_parts(self) -> (Subject<Text, Extension>, Iri<Text>, Term<Text, Extension>) {
        (self.subject, self.predicate, self.object)
    }
    /// The subject.
    #[must_use]
    pub const fn subject(&self) -> &Subject<Text, Extension> {
        &self.subject
    }
    /// The predicate IRI.
    #[must_use]
    pub const fn predicate(&self) -> &Iri<Text> {
        &self.predicate
    }
    /// The object, which may itself be a triple term.
    #[must_use]
    pub const fn object(&self) -> &Term<Text, Extension> {
        &self.object
    }
}

/// A dataset graph position, including an explicit default graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub enum GraphName<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    /// The unnamed default graph.
    Default,
    /// An IRI graph name.
    Iri(Iri<Text>),
    /// A scoped blank-node graph name.
    BlankNode(BlankNodeId<Text>),
    /// An explicitly selected graph-name extension.
    Extension(Extension),
}

/// One quad; this does not allocate or hydrate a dataset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub struct Quad<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    triple: Triple<Text, Extension>,
    graph: GraphName<Text, Extension>,
}

impl<Text: Bos<str> + AsRef<str>, Extension> Quad<Text, Extension> {
    /// Associate a triple with its dataset graph.
    #[must_use]
    pub const fn new(triple: Triple<Text, Extension>, graph: GraphName<Text, Extension>) -> Self {
        Self { triple, graph }
    }
    /// Consume the quad without cloning its triple or graph name.
    #[must_use]
    pub fn into_parts(self) -> (Triple<Text, Extension>, GraphName<Text, Extension>) {
        (self.triple, self.graph)
    }
    /// The triple in this graph.
    #[must_use]
    pub const fn triple(&self) -> &Triple<Text, Extension> {
        &self.triple
    }
    /// The graph position.
    #[must_use]
    pub const fn graph(&self) -> &GraphName<Text, Extension> {
        &self.graph
    }
}
