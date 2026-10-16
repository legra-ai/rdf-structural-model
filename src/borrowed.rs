//! Borrow text without changing structural or identity semantics.

use crate::{
    GraphName,
    Quad,
    Subject,
    Term,
    Triple,
};

impl<Extension> Term<String, Extension> {
    /// Borrow all text and extension payloads.
    ///
    /// Lexical payloads are never copied. Nested triple terms allocate only
    /// their structural boxes; this is not an allocation-free graph projection.
    #[must_use]
    pub fn as_ref(&self) -> Term<&str, &Extension> {
        match self {
            Self::Iri(iri) => Term::Iri(iri.borrow()),
            Self::BlankNode(blank) => Term::BlankNode(blank.as_ref()),
            Self::Literal(literal) => Term::Literal(literal.as_ref()),
            Self::Triple(triple) => Term::Triple(Box::new(triple.as_ref().as_ref())),
            Self::Extension(extension) => Term::Extension(extension),
        }
    }
}

impl<Extension> Subject<String, Extension> {
    /// Borrow subject text and an explicitly selected extension payload.
    #[must_use]
    pub fn as_ref(&self) -> Subject<&str, &Extension> {
        match self {
            Self::Iri(iri) => Subject::Iri(iri.borrow()),
            Self::BlankNode(blank) => Subject::BlankNode(blank.as_ref()),
            Self::Extension(extension) => Subject::Extension(extension),
        }
    }
}

impl<Extension> Triple<String, Extension> {
    /// Borrow all term text without copying lexical payloads.
    #[must_use]
    pub fn as_ref(&self) -> Triple<&str, &Extension> {
        Triple::new(
            self.subject().as_ref(),
            self.predicate().borrow(),
            self.object().as_ref(),
        )
    }
}

impl<Extension> GraphName<String, Extension> {
    /// Borrow graph-name text or its extension payload.
    #[must_use]
    pub fn as_ref(&self) -> GraphName<&str, &Extension> {
        match self {
            Self::Default => GraphName::Default,
            Self::Iri(iri) => GraphName::Iri(iri.borrow()),
            Self::BlankNode(blank) => GraphName::BlankNode(blank.as_ref()),
            Self::Extension(extension) => GraphName::Extension(extension),
        }
    }
}

impl<Extension> Quad<String, Extension> {
    /// Borrow all quad text without hydrating a dataset.
    #[must_use]
    pub fn as_ref(&self) -> Quad<&str, &Extension> {
        Quad::new(self.triple().as_ref(), self.graph().as_ref())
    }
}
