#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod blank_node;
mod borrowed;
mod decode;
mod direction;
mod error;
mod generalized;
mod language;
mod literal;
mod nesting;
mod order;
mod reification;
mod statement;
mod term;
mod vocabulary;

pub use blank_node::BlankNodeId;
pub use direction::BaseDirection;
pub use error::ModelError;
pub use generalized::{
    GeneralizedGraphName,
    GeneralizedQuad,
    GeneralizedTerm,
    GeneralizedTriple,
};
pub use language::{
    LanguageTag,
    LanguageTagError,
    LanguageTagErrorReason,
    SubtagIndex,
};
pub use literal::{
    Literal,
    LiteralFacets,
    LiteralMetadata,
};
pub use nesting::{
    MAX_TRIPLE_TERM_DEPTH,
    NestingDepth,
};
pub use reification::Reification;
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
pub use vocabulary::{
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
    RDF_NAMESPACE,
    RDF_REIFIES,
    XSD_STRING,
};
