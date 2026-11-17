//! Depth-checked decoding of standard terms and triples.
use core::fmt;
use core::marker::PhantomData;

use borrow_or_share::Bos;
use iri_rfc3987::Iri;
use serde::de::{
    DeserializeSeed,
    Deserializer,
    EnumAccess,
    Error,
    MapAccess,
    SeqAccess,
    VariantAccess,
    Visitor,
};

use super::identifier::Identifier;
use crate::{
    NestingDepth,
    Subject,
    Term,
    Triple,
};

const TERM_VARIANTS: &[&str] = &["Iri", "BlankNode", "Literal", "Triple", "Extension"];
const TRIPLE_FIELDS: &[&str] = &["subject", "predicate", "object"];

/// Decodes a [`Term`] found at a known nesting depth.
struct TermSeed<Text, Extension> {
    depth: NestingDepth,
    marker: PhantomData<fn() -> (Text, Extension)>,
}

/// Decodes a [`Triple`] found at a known nesting depth.
struct TripleSeed<Text, Extension> {
    depth: NestingDepth,
    marker: PhantomData<fn() -> (Text, Extension)>,
}

impl<Text, Extension> TermSeed<Text, Extension> {
    const fn at(depth: NestingDepth) -> Self {
        Self {
            depth,
            marker: PhantomData,
        }
    }
}

impl<Text, Extension> TripleSeed<Text, Extension> {
    const fn at(depth: NestingDepth) -> Self {
        Self {
            depth,
            marker: PhantomData,
        }
    }
}

impl<'de, Text, Extension> DeserializeSeed<'de> for TermSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = Term<Text, Extension>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_enum("Term", TERM_VARIANTS, self)
    }
}

impl<'de, Text, Extension> Visitor<'de> for TermSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = Term<Text, Extension>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum Term")
    }
    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let (index, variant) = data.variant_seed(Identifier::variant(TERM_VARIANTS))?;
        match index {
            0 => variant.newtype_variant().map(Term::Iri),
            1 => variant.newtype_variant().map(Term::BlankNode),
            2 => variant.newtype_variant().map(Term::Literal),
            3 => {
                let depth = self.depth.descend().map_err(A::Error::custom)?;
                variant
                    .newtype_variant_seed(TripleSeed::at(depth))
                    .map(Term::triple)
            }
            _ => variant.newtype_variant().map(Term::Extension),
        }
    }
}

impl<'de, Text, Extension> DeserializeSeed<'de> for TripleSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = Triple<Text, Extension>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("Triple", TRIPLE_FIELDS, self)
    }
}

impl<'de, Text, Extension> Visitor<'de> for TripleSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = Triple<Text, Extension>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("struct Triple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let expected = &"struct Triple with 3 elements";
        let subject: Subject<Text, Extension> = seq
            .next_element()?
            .ok_or_else(|| A::Error::invalid_length(0, expected))?;
        let predicate: Iri<Text> = seq
            .next_element()?
            .ok_or_else(|| A::Error::invalid_length(1, expected))?;
        let object = seq
            .next_element_seed(TermSeed::at(self.depth))?
            .ok_or_else(|| A::Error::invalid_length(2, expected))?;
        Ok(Triple::new(subject, predicate, object))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut subject: Option<Subject<Text, Extension>> = None;
        let mut predicate: Option<Iri<Text>> = None;
        let mut object = None;
        while let Some(field) = map.next_key_seed(Identifier::field(TRIPLE_FIELDS))? {
            match field {
                0 if subject.is_none() => subject = Some(map.next_value()?),
                1 if predicate.is_none() => predicate = Some(map.next_value()?),
                2 if object.is_none() => {
                    object = Some(map.next_value_seed(TermSeed::at(self.depth))?);
                }
                _ => return Err(A::Error::duplicate_field(TRIPLE_FIELDS[field])),
            }
        }
        Ok(Triple::new(
            subject.ok_or_else(|| A::Error::missing_field("subject"))?,
            predicate.ok_or_else(|| A::Error::missing_field("predicate"))?,
            object.ok_or_else(|| A::Error::missing_field("object"))?,
        ))
    }
}

impl<'de, Text, Extension> serde::Deserialize<'de> for Term<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TermSeed::at(NestingDepth::ROOT).deserialize(deserializer)
    }
}

impl<'de, Text, Extension> serde::Deserialize<'de> for Triple<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TripleSeed::at(NestingDepth::ROOT).deserialize(deserializer)
    }
}
