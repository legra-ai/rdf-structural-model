//! Case-insensitive total order on language tags, consistent with `Eq`.

use std::cmp::Ordering;

use super::tag::LanguageTag;

impl<Text: AsRef<str>> Ord for LanguageTag<Text> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.folded_bytes().cmp(other.folded_bytes())
    }
}
impl<Text: AsRef<str>> PartialOrd for LanguageTag<Text> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
