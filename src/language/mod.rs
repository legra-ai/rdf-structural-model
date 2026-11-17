//! Well-formed, case-insensitive BCP 47 language tags.

mod error;
mod grandfathered;
mod order;
mod parse;
mod tag;

pub use error::{
    LanguageTagError,
    LanguageTagErrorReason,
    SubtagIndex,
};
pub use tag::LanguageTag;
