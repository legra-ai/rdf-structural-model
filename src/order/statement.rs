//! Order on triples, graph names and quads.

use std::cmp::Ordering;

use borrow_or_share::Bos;

use super::kind::PositionKind;
use crate::{
    GraphName,
    Quad,
    Triple,
};

impl<Text: Bos<str> + AsRef<str> + Eq, Extension: Ord> Ord for Triple<Text, Extension> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.subject()
            .cmp(other.subject())
            .then_with(|| self.predicate().cmp(other.predicate()))
            .then_with(|| self.object().cmp(other.object()))
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> PositionKind for GraphName<Text, Extension> {
    fn kind_rank(&self) -> u8 {
        match self {
            Self::Default => 0,
            Self::Iri(_) => 1,
            Self::BlankNode(_) => 2,
            Self::Extension(_) => 3,
        }
    }
}

impl<Text: Bos<str> + AsRef<str> + Eq, Extension: Ord> Ord for GraphName<Text, Extension> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Iri(left), Self::Iri(right)) => left.cmp(right),
            (Self::BlankNode(left), Self::BlankNode(right)) => left.cmp(right),
            (Self::Extension(left), Self::Extension(right)) => left.cmp(right),
            _ => self.kind_rank().cmp(&other.kind_rank()),
        }
    }
}

/// Quads order by their triple (subject, predicate, object), then graph.
impl<Text: Bos<str> + AsRef<str> + Eq, Extension: Ord> Ord for Quad<Text, Extension> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.triple()
            .cmp(other.triple())
            .then_with(|| self.graph().cmp(other.graph()))
    }
}

impl<Text, Extension> PartialOrd for Triple<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + Eq,
    Extension: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text, Extension> PartialOrd for GraphName<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + Eq,
    Extension: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text, Extension> PartialOrd for Quad<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + Eq,
    Extension: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
