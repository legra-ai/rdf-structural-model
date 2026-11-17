//! Order on terms and subjects.

use std::cmp::Ordering;

use borrow_or_share::Bos;

use super::kind::PositionKind;
use crate::{
    Subject,
    Term,
};

impl<Text: Bos<str> + AsRef<str>, Extension> PositionKind for Term<Text, Extension> {
    fn kind_rank(&self) -> u8 {
        match self {
            Self::Iri(_) => 0,
            Self::BlankNode(_) => 1,
            Self::Literal(_) => 2,
            Self::Triple(_) => 3,
            Self::Extension(_) => 4,
        }
    }
}

impl<Text: Bos<str> + AsRef<str> + Eq, Extension: Ord> Ord for Term<Text, Extension> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Iri(left), Self::Iri(right)) => left.cmp(right),
            (Self::BlankNode(left), Self::BlankNode(right)) => left.cmp(right),
            (Self::Literal(left), Self::Literal(right)) => left.cmp(right),
            (Self::Triple(left), Self::Triple(right)) => left.cmp(right),
            (Self::Extension(left), Self::Extension(right)) => left.cmp(right),
            _ => self.kind_rank().cmp(&other.kind_rank()),
        }
    }
}

impl<Text: Bos<str> + AsRef<str>, Extension> PositionKind for Subject<Text, Extension> {
    fn kind_rank(&self) -> u8 {
        match self {
            Self::Iri(_) => 0,
            Self::BlankNode(_) => 1,
            Self::Extension(_) => 2,
        }
    }
}

impl<Text: Bos<str> + AsRef<str> + Eq, Extension: Ord> Ord for Subject<Text, Extension> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Iri(left), Self::Iri(right)) => left.cmp(right),
            (Self::BlankNode(left), Self::BlankNode(right)) => left.cmp(right),
            (Self::Extension(left), Self::Extension(right)) => left.cmp(right),
            _ => self.kind_rank().cmp(&other.kind_rank()),
        }
    }
}

impl<Text, Extension> PartialOrd for Term<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + Eq,
    Extension: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text, Extension> PartialOrd for Subject<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + Eq,
    Extension: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
