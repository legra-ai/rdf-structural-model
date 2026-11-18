//! Structural errors independent of an RDF backend.

use crate::{
    LanguageTagError,
    NestingDepth,
};

/// A rejected RDF structural value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    /// A language tag is not well-formed BCP 47.
    #[error(transparent)]
    InvalidLanguageTag(LanguageTagError),
    /// An abstract blank-node identifier is empty.
    #[error("blank-node identifier must not be empty")]
    EmptyBlankNode,
    /// A language datatype has missing or inconsistent facets.
    #[error("literal datatype and language/direction facets disagree")]
    InconsistentLiteralFacets,
    /// A literal or triple term cannot occupy a standard subject position.
    #[error("term cannot occupy an RDF 1.2 subject position")]
    InvalidSubject,
    /// Only an IRI can occupy a standard predicate position.
    #[error("term cannot occupy an RDF 1.2 predicate position")]
    InvalidPredicate,
    /// A literal or triple term cannot name a standard RDF graph.
    #[error("term cannot occupy an RDF 1.2 graph-name position")]
    InvalidGraphName,
    /// A reifying triple's predicate must be `rdf:reifies` (RDF 1.2 Concepts
    /// §1.5).
    #[error("reifying triple predicate must be rdf:reifies")]
    NotReifyingPredicate,
    /// A reifying triple's object must be a triple term (RDF 1.2 Concepts
    /// §1.5).
    #[error("reifying triple object must be a triple term")]
    ReifiedObjectNotTripleTerm,
    /// Triple terms nest deeper than [`crate::MAX_TRIPLE_TERM_DEPTH`].
    #[error("triple-term nesting depth {depth} exceeds the limit of {limit}")]
    NestingTooDeep {
        /// The enforced limit.
        limit: NestingDepth,
        /// The depth reached when decoding or conversion stopped.
        depth: NestingDepth,
    },
}
