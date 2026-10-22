//! Terms whose nested triples may themselves be generalized.
use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use super::GeneralizedTriple;
use crate::{
    BlankNodeId,
    Literal,
    NoExtension,
};

/// A generalized RDF term, including recursively generalized triple terms.
///
/// This is an explicit non-standard RDF domain, suitable for reasoning
/// intermediates. It does not relax the position restrictions of
/// [`crate::Term`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub enum GeneralizedTerm<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    /// An absolute IRI.
    Iri(Iri<Text>),
    /// A scoped blank node.
    BlankNode(BlankNodeId<Text>),
    /// An exact lexical literal.
    Literal(Literal<Text>),
    /// A recursively generalized triple term.
    Triple(Box<GeneralizedTriple<Text, Extension>>),
    /// An explicitly selected application extension.
    Extension(Extension),
}
