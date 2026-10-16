//! Structural errors independent of an RDF backend.

/// A rejected RDF structural value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    /// A language tag is not well-formed BCP 47.
    #[error("language tag is not well-formed BCP 47")]
    InvalidLanguageTag,
    /// An abstract blank-node identifier is empty.
    #[error("blank-node identifier must not be empty")]
    EmptyBlankNode,
    /// A language datatype has missing or inconsistent facets.
    #[error("literal datatype and language/direction facets disagree")]
    InconsistentLiteralFacets,
    /// A literal or triple term cannot occupy a standard subject position.
    #[error("term cannot occupy an RDF 1.2 subject position")]
    InvalidSubject,
}
