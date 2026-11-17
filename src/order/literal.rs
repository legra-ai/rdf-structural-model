//! Order on blank nodes and literals.

use std::cmp::Ordering;

use borrow_or_share::Bos;

use super::kind::PositionKind;
use crate::{
    BlankNodeId,
    Literal,
    LiteralFacets,
    LiteralMetadata,
};

impl<Text: Bos<str> + AsRef<str> + Eq> Ord for BlankNodeId<Text> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scope()
            .cmp(other.scope())
            .then_with(|| self.label().cmp(other.label()))
    }
}

impl<Text: AsRef<str>> PositionKind for LiteralFacets<Text> {
    fn kind_rank(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Language(_) => 1,
            Self::Directional { .. } => 2,
        }
    }
}

impl<Text: AsRef<str> + Eq> Ord for LiteralFacets<Text> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Language(left), Self::Language(right)) => left.cmp(right),
            (
                Self::Directional {
                    language: left_language,
                    direction: left_direction,
                },
                Self::Directional {
                    language: right_language,
                    direction: right_direction,
                },
            ) => left_language
                .cmp(right_language)
                .then_with(|| left_direction.cmp(right_direction)),
            _ => self.kind_rank().cmp(&other.kind_rank()),
        }
    }
}

impl<Text: Bos<str> + AsRef<str> + Eq> Ord for LiteralMetadata<Text> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.datatype()
            .cmp(other.datatype())
            .then_with(|| self.facets().cmp(other.facets()))
    }
}

impl<Text: Bos<str> + AsRef<str> + Eq> Ord for Literal<Text> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.lexical()
            .cmp(other.lexical())
            .then_with(|| self.metadata().cmp(other.metadata()))
    }
}

impl<Text> PartialOrd for BlankNodeId<Text>
where
    Text: Bos<str> + AsRef<str> + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text> PartialOrd for LiteralMetadata<Text>
where
    Text: Bos<str> + AsRef<str> + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text> PartialOrd for Literal<Text>
where
    Text: Bos<str> + AsRef<str> + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Text> PartialOrd for LiteralFacets<Text>
where
    Text: AsRef<str> + Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
