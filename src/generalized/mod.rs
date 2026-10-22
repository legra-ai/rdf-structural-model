//! Explicit generalized RDF values, distinct from standard RDF positions.
mod borrowed;
mod conversion;
mod statement;
mod term;

pub use statement::{
    GeneralizedGraphName,
    GeneralizedQuad,
    GeneralizedTriple,
};
pub use term::GeneralizedTerm;
