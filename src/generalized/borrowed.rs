//! Borrow generalized structure without copying lexical or extension payloads.
use super::{
    GeneralizedGraphName,
    GeneralizedQuad,
    GeneralizedTerm,
    GeneralizedTriple,
};

impl<Extension> GeneralizedTerm<String, Extension> {
    /// Borrow all text and extension payloads.
    ///
    /// Nested triple terms allocate only structural boxes, as in standard
    /// terms.
    #[must_use]
    pub fn as_ref(&self) -> GeneralizedTerm<&str, &Extension> {
        match self {
            Self::Iri(iri) => GeneralizedTerm::Iri(iri.borrow()),
            Self::BlankNode(blank) => GeneralizedTerm::BlankNode(blank.as_ref()),
            Self::Literal(literal) => GeneralizedTerm::Literal(literal.as_ref()),
            Self::Triple(triple) => GeneralizedTerm::Triple(Box::new(triple.as_ref().as_ref())),
            Self::Extension(extension) => GeneralizedTerm::Extension(extension),
        }
    }
}

impl<Extension> GeneralizedTriple<String, Extension> {
    /// Borrow every term without copying its lexical payloads.
    #[must_use]
    pub fn as_ref(&self) -> GeneralizedTriple<&str, &Extension> {
        GeneralizedTriple::new(
            self.subject().as_ref(),
            self.predicate().as_ref(),
            self.object().as_ref(),
        )
    }
}

impl<Extension> GeneralizedGraphName<String, Extension> {
    /// Borrow graph-name text and extension payloads.
    #[must_use]
    pub fn as_ref(&self) -> GeneralizedGraphName<&str, &Extension> {
        match self {
            Self::Default => GeneralizedGraphName::Default,
            Self::Term(term) => GeneralizedGraphName::Term(term.as_ref()),
        }
    }
}

impl<Extension> GeneralizedQuad<String, Extension> {
    /// Borrow the quad without hydrating any graph or dataset.
    #[must_use]
    pub fn as_ref(&self) -> GeneralizedQuad<&str, &Extension> {
        GeneralizedQuad::new(self.triple().as_ref(), self.graph().as_ref())
    }
}
