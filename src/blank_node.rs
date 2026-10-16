//! Blank-node identity explicitly scoped by the application.

use borrow_or_share::Bos;
use iri_rfc3987::Iri;
use serde::{
    Deserialize,
    Deserializer,
};

use crate::ModelError;

/// An abstract blank-node identity, independent of a concrete syntax label.
///
/// Scope is an application-assigned IRI identifying a shared blank-node domain.
/// Use distinct scopes when importing independent sources. This crate never
/// creates scopes or allocates global identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(
        serialize = "Text: serde::Serialize",
        deserialize = "Text: serde::Deserialize<'de>, Iri<Text>: serde::Deserialize<'de>"
    )
)]
pub struct BlankNodeId<Text: Bos<str> + AsRef<str> = String> {
    scope: Iri<Text>,
    #[serde(deserialize_with = "BlankNodeId::deserialize_label")]
    label: Text,
}

impl<Text: Bos<str> + AsRef<str>> BlankNodeId<Text> {
    /// Create an identity in an explicitly supplied scope.
    ///
    /// # Errors
    /// Returns [`ModelError::EmptyBlankNode`] if the abstract identifier is
    /// empty.
    pub fn try_new(scope: Iri<Text>, label: Text) -> Result<Self, ModelError> {
        Self::validate_label(label.as_ref())?;
        Ok(Self { scope, label })
    }

    /// The application-assigned scope IRI.
    #[must_use]
    pub const fn scope(&self) -> &Iri<Text> {
        &self.scope
    }

    /// The abstract identifier within that scope, without syntax decoration.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    fn validate_label(label: &str) -> Result<(), ModelError> {
        if label.is_empty() {
            return Err(ModelError::EmptyBlankNode);
        }
        Ok(())
    }

    fn deserialize_label<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Text, D::Error>
    where
        Text: Deserialize<'de>,
    {
        let label = Text::deserialize(deserializer)?;
        Self::validate_label(label.as_ref()).map_err(serde::de::Error::custom)?;
        Ok(label)
    }
}

impl BlankNodeId<String> {
    /// Borrow scope and label without copying their text.
    #[must_use]
    pub fn as_ref(&self) -> BlankNodeId<&str> {
        BlankNodeId {
            scope: self.scope.borrow(),
            label: &self.label,
        }
    }
}
