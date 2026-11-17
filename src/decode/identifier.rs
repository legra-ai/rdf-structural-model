//! Variant and field identifiers decoded by name or by index.
use core::fmt;

use serde::de::{
    DeserializeSeed,
    Deserializer,
    Error,
    Unexpected,
    Visitor,
};

/// Whether an identifier names an enum variant or a struct field.
#[derive(Debug, Clone, Copy)]
pub(super) enum IdentifierKind {
    Variant,
    Field,
}

/// Decodes one identifier from a fixed name table into its index.
#[derive(Debug, Clone, Copy)]
pub(super) struct Identifier {
    kind: IdentifierKind,
    names: &'static [&'static str],
}

impl Identifier {
    pub(super) const fn variant(names: &'static [&'static str]) -> Self {
        Self {
            kind: IdentifierKind::Variant,
            names,
        }
    }
    pub(super) const fn field(names: &'static [&'static str]) -> Self {
        Self {
            kind: IdentifierKind::Field,
            names,
        }
    }
    fn unknown<E: Error>(self, name: &str) -> E {
        match self.kind {
            IdentifierKind::Variant => E::unknown_variant(name, self.names),
            IdentifierKind::Field => E::unknown_field(name, self.names),
        }
    }
}

impl<'de> DeserializeSeed<'de> for Identifier {
    type Value = usize;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<usize, D::Error> {
        deserializer.deserialize_identifier(self)
    }
}

impl Visitor<'_> for Identifier {
    type Value = usize;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "one of {:?}", self.names)
    }
    fn visit_u64<E: Error>(self, index: u64) -> Result<usize, E> {
        usize::try_from(index)
            .ok()
            .filter(|index| *index < self.names.len())
            .ok_or_else(|| E::invalid_value(Unexpected::Unsigned(index), &self))
    }
    fn visit_str<E: Error>(self, name: &str) -> Result<usize, E> {
        self.names
            .iter()
            .position(|candidate| *candidate == name)
            .ok_or_else(|| self.unknown(name))
    }
    fn visit_bytes<E: Error>(self, name: &[u8]) -> Result<usize, E> {
        match core::str::from_utf8(name) {
            Ok(name) => self.visit_str(name),
            Err(_) => Err(E::invalid_value(Unexpected::Bytes(name), &self)),
        }
    }
}
