//! Structural view of an RDF 1.2 reifying triple (RDF 1.2 Concepts §1.5).

use borrow_or_share::Bos;
use iri_rfc3987::Iri;

use crate::{
    ModelError,
    NoExtension,
    RDF_REIFIES,
    Subject,
    Term,
    Triple,
};

/// A reifier and the triple it reifies: the triple `reifier rdf:reifies <<(
/// triple )>>`.
///
/// This is a structural helper over one statement. It neither asserts the
/// reified triple nor models annotation syntax or a dataset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize, Extension: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>, Extension: serde::Deserialize<'de>"
    )
)]
pub struct Reification<Text: Bos<str> + AsRef<str> = String, Extension = NoExtension> {
    reifier: Subject<Text, Extension>,
    triple: Triple<Text, Extension>,
}

impl<Text: Bos<str> + AsRef<str>, Extension> Reification<Text, Extension> {
    /// Pair a reifier with the triple it reifies.
    #[must_use]
    pub const fn new(reifier: Subject<Text, Extension>, triple: Triple<Text, Extension>) -> Self {
        Self { reifier, triple }
    }
    /// The reifier: the subject of the reifying triple.
    #[must_use]
    pub const fn reifier(&self) -> &Subject<Text, Extension> {
        &self.reifier
    }
    /// The reified triple, carried as the reifying triple's triple-term object.
    #[must_use]
    pub const fn triple(&self) -> &Triple<Text, Extension> {
        &self.triple
    }
    /// Consume the reification without cloning either part.
    #[must_use]
    pub fn into_parts(self) -> (Subject<Text, Extension>, Triple<Text, Extension>) {
        (self.reifier, self.triple)
    }
}

impl<Extension> Reification<String, Extension> {
    /// Borrow all text without copying lexical payloads.
    #[must_use]
    pub fn as_ref(&self) -> Reification<&str, &Extension> {
        Reification::new(self.reifier.as_ref(), self.triple.as_ref())
    }
}

/// Text representations that can name the built-in `rdf:reifies` predicate.
trait ReifiesPredicate: Bos<str> + AsRef<str> + Sized {
    fn rdf_reifies() -> Iri<Self>;
}

impl ReifiesPredicate for String {
    fn rdf_reifies() -> Iri<Self> {
        <&str as ReifiesPredicate>::rdf_reifies().into()
    }
}

impl ReifiesPredicate for &str {
    fn rdf_reifies() -> Iri<Self> {
        Iri::parse(RDF_REIFIES).expect("rdf:reifies is a valid IRI")
    }
}

/// Recognize a reifying triple: predicate `rdf:reifies`, triple-term object.
impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<Triple<Text, Extension>>
    for Reification<Text, Extension>
{
    type Error = ModelError;
    fn try_from(triple: Triple<Text, Extension>) -> Result<Self, Self::Error> {
        if triple.predicate().as_str() != RDF_REIFIES {
            return Err(ModelError::NotReifyingPredicate);
        }
        let (reifier, _, object) = triple.into_parts();
        match object {
            Term::Triple(quoted) => Ok(Self::new(reifier, *quoted)),
            Term::Iri(_) | Term::BlankNode(_) | Term::Literal(_) | Term::Extension(_) => {
                Err(ModelError::ReifiedObjectNotTripleTerm)
            }
        }
    }
}

/// Rebuild the reifying triple `reifier rdf:reifies <<( triple )>>`.
impl<Text: ReifiesPredicate, Extension> From<Reification<Text, Extension>>
    for Triple<Text, Extension>
{
    fn from(reification: Reification<Text, Extension>) -> Self {
        let (reifier, triple) = reification.into_parts();
        Self::new(reifier, Text::rdf_reifies(), Term::triple(triple))
    }
}
