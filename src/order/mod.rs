//! One total order on terms and statements, consistent with `Eq` and `Hash`.
//!
//! Values compare by position kind first, then by content: IRI text,
//! blank-node scope then label, literal lexical form, datatype, then facets
//! with language compared ASCII case-insensitively and direction last.

mod kind;
mod literal;
mod statement;
mod term;
