//! Depth-checked decoding of generalized terms and triples.
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
    GeneralizedTerm,
    GeneralizedTriple,
    NestingDepth,
};

const TERM_VARIANTS: &[&str] = &["Iri", "BlankNode", "Literal", "Triple", "Extension"];
const TRIPLE_FIELDS: &[&str] = &["subject", "predicate", "object"];

/// Decodes a [`GeneralizedTerm`] found at a known nesting depth.
struct TermSeed<Text, Extension> {
    depth: NestingDepth,
    marker: PhantomData<fn() -> (Text, Extension)>,
}

/// Decodes a [`GeneralizedTriple`] found at a known nesting depth.
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
    type Value = GeneralizedTerm<Text, Extension>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_enum("GeneralizedTerm", TERM_VARIANTS, self)
    }
}

impl<'de, Text, Extension> Visitor<'de> for TermSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = GeneralizedTerm<Text, Extension>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum GeneralizedTerm")
    }
    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let (index, variant) = data.variant_seed(Identifier::variant(TERM_VARIANTS))?;
        match index {
            0 => variant.newtype_variant().map(GeneralizedTerm::Iri),
            1 => variant.newtype_variant().map(GeneralizedTerm::BlankNode),
            2 => variant.newtype_variant().map(GeneralizedTerm::Literal),
            3 => {
                let depth = self.depth.descend().map_err(A::Error::custom)?;
                variant
                    .newtype_variant_seed(TripleSeed::at(depth))
                    .map(|triple| GeneralizedTerm::Triple(Box::new(triple)))
            }
            _ => variant.newtype_variant().map(GeneralizedTerm::Extension),
        }
    }
}

impl<'de, Text, Extension> DeserializeSeed<'de> for TripleSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = GeneralizedTriple<Text, Extension>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("GeneralizedTriple", TRIPLE_FIELDS, self)
    }
}

impl<'de, Text, Extension> Visitor<'de> for TripleSeed<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    type Value = GeneralizedTriple<Text, Extension>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("struct GeneralizedTriple")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let expected = &"struct GeneralizedTriple with 3 elements";
        let subject = seq
            .next_element_seed(TermSeed::at(self.depth))?
            .ok_or_else(|| A::Error::invalid_length(0, expected))?;
        let predicate = seq
            .next_element_seed(TermSeed::at(self.depth))?
            .ok_or_else(|| A::Error::invalid_length(1, expected))?;
        let object = seq
            .next_element_seed(TermSeed::at(self.depth))?
            .ok_or_else(|| A::Error::invalid_length(2, expected))?;
        Ok(GeneralizedTriple::new(subject, predicate, object))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut subject = None;
        let mut predicate = None;
        let mut object = None;
        while let Some(field) = map.next_key_seed(Identifier::field(TRIPLE_FIELDS))? {
            match field {
                0 if subject.is_none() => {
                    subject = Some(map.next_value_seed(TermSeed::at(self.depth))?);
                }
                1 if predicate.is_none() => {
                    predicate = Some(map.next_value_seed(TermSeed::at(self.depth))?);
                }
                2 if object.is_none() => {
                    object = Some(map.next_value_seed(TermSeed::at(self.depth))?);
                }
                _ => return Err(A::Error::duplicate_field(TRIPLE_FIELDS[field])),
            }
        }
        Ok(GeneralizedTriple::new(
            subject.ok_or_else(|| A::Error::missing_field("subject"))?,
            predicate.ok_or_else(|| A::Error::missing_field("predicate"))?,
            object.ok_or_else(|| A::Error::missing_field("object"))?,
        ))
    }
}

impl<'de, Text, Extension> serde::Deserialize<'de> for GeneralizedTerm<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TermSeed::at(NestingDepth::ROOT).deserialize(deserializer)
    }
}

impl<'de, Text, Extension> serde::Deserialize<'de> for GeneralizedTriple<Text, Extension>
where
    Text: Bos<str> + AsRef<str> + serde::Deserialize<'de>,
    Iri<Text>: serde::Deserialize<'de>,
    Extension: serde::Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TripleSeed::at(NestingDepth::ROOT).deserialize(deserializer)
    }
}
