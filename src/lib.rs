#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod blank_node;
mod borrowed;
mod direction;
mod error;
mod language;
mod literal;
mod statement;
mod term;

pub use blank_node::BlankNodeId;
pub use direction::BaseDirection;
pub use error::ModelError;
pub use language::LanguageTag;
pub use literal::{
    Literal,
    LiteralFacets,
    LiteralMetadata,
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
    XSD_STRING,
};
pub use statement::{
    GraphName,
    Quad,
    Triple,
};
pub use term::{
    NoExtension,
    Subject,
    Term,
};
