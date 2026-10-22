//! Explicit, checked conversion between standard and generalized domains.
use borrow_or_share::Bos;

use super::{
    GeneralizedGraphName,
    GeneralizedQuad,
    GeneralizedTerm,
    GeneralizedTriple,
};
use crate::{
    GraphName,
    ModelError,
    Quad,
    Subject,
    Term,
    Triple,
};

impl<Text: Bos<str> + AsRef<str>, Extension> From<Term<Text, Extension>>
    for GeneralizedTerm<Text, Extension>
{
    fn from(value: Term<Text, Extension>) -> Self {
        match value {
            Term::Iri(iri) => Self::Iri(iri),
            Term::BlankNode(blank) => Self::BlankNode(blank),
            Term::Literal(literal) => Self::Literal(literal),
            Term::Triple(triple) => Self::Triple(Box::new((*triple).into())),
            Term::Extension(extension) => Self::Extension(extension),
        }
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<GeneralizedTerm<Text, Extension>>
    for Term<Text, Extension>
{
    type Error = ModelError;
    fn try_from(value: GeneralizedTerm<Text, Extension>) -> Result<Self, Self::Error> {
        Ok(match value {
            GeneralizedTerm::Iri(iri) => Self::Iri(iri),
            GeneralizedTerm::BlankNode(blank) => Self::BlankNode(blank),
            GeneralizedTerm::Literal(literal) => Self::Literal(literal),
            GeneralizedTerm::Triple(triple) => Self::Triple(Box::new((*triple).try_into()?)),
            GeneralizedTerm::Extension(extension) => Self::Extension(extension),
        })
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> From<Triple<Text, Extension>>
    for GeneralizedTriple<Text, Extension>
{
    fn from(value: Triple<Text, Extension>) -> Self {
        let (subject, predicate, object) = value.into_parts();
        let subject = match subject {
            Subject::Iri(iri) => GeneralizedTerm::Iri(iri),
            Subject::BlankNode(blank) => GeneralizedTerm::BlankNode(blank),
            Subject::Extension(extension) => GeneralizedTerm::Extension(extension),
        };
        Self::new(subject, GeneralizedTerm::Iri(predicate), object.into())
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<GeneralizedTriple<Text, Extension>>
    for Triple<Text, Extension>
{
    type Error = ModelError;
    fn try_from(value: GeneralizedTriple<Text, Extension>) -> Result<Self, Self::Error> {
        let (subject, predicate, object) = value.into_parts();
        let subject = match subject {
            GeneralizedTerm::Iri(iri) => Subject::Iri(iri),
            GeneralizedTerm::BlankNode(blank) => Subject::BlankNode(blank),
            GeneralizedTerm::Extension(extension) => Subject::Extension(extension),
            GeneralizedTerm::Literal(_) | GeneralizedTerm::Triple(_) => {
                return Err(ModelError::InvalidSubject);
            }
        };
        let GeneralizedTerm::Iri(predicate) = predicate else {
            return Err(ModelError::InvalidPredicate);
        };
        Ok(Self::new(subject, predicate, object.try_into()?))
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> From<GraphName<Text, Extension>>
    for GeneralizedGraphName<Text, Extension>
{
    fn from(value: GraphName<Text, Extension>) -> Self {
        match value {
            GraphName::Default => Self::Default,
            GraphName::Iri(iri) => Self::Term(GeneralizedTerm::Iri(iri)),
            GraphName::BlankNode(blank) => Self::Term(GeneralizedTerm::BlankNode(blank)),
            GraphName::Extension(extension) => Self::Term(GeneralizedTerm::Extension(extension)),
        }
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<GeneralizedGraphName<Text, Extension>>
    for GraphName<Text, Extension>
{
    type Error = ModelError;
    fn try_from(value: GeneralizedGraphName<Text, Extension>) -> Result<Self, Self::Error> {
        match value {
            GeneralizedGraphName::Default => Ok(Self::Default),
            GeneralizedGraphName::Term(GeneralizedTerm::Iri(iri)) => Ok(Self::Iri(iri)),
            GeneralizedGraphName::Term(GeneralizedTerm::BlankNode(blank)) => {
                Ok(Self::BlankNode(blank))
            }
            GeneralizedGraphName::Term(GeneralizedTerm::Extension(extension)) => {
                Ok(Self::Extension(extension))
            }
            GeneralizedGraphName::Term(
                GeneralizedTerm::Literal(_) | GeneralizedTerm::Triple(_),
            ) => Err(ModelError::InvalidGraphName),
        }
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> From<Quad<Text, Extension>>
    for GeneralizedQuad<Text, Extension>
{
    fn from(value: Quad<Text, Extension>) -> Self {
        let (triple, graph) = value.into_parts();
        Self::new(triple.into(), graph.into())
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> TryFrom<GeneralizedQuad<Text, Extension>>
    for Quad<Text, Extension>
{
    type Error = ModelError;
    fn try_from(value: GeneralizedQuad<Text, Extension>) -> Result<Self, Self::Error> {
        let (triple, graph) = value.into_parts();
        Ok(Self::new(triple.try_into()?, graph.try_into()?))
    }
}
