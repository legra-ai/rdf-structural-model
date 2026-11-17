//! RFC 5646 irregular grandfathered tags, which do not match `langtag`.

/// Irregular grandfathered tags (RFC 5646 §2.1, `irregular`). The regular
/// grandfathered tags already match the `langtag` production.
const IRREGULAR: [&str; 17] = [
    "en-GB-oed",
    "i-ami",
    "i-bnn",
    "i-default",
    "i-enochian",
    "i-hak",
    "i-klingon",
    "i-lux",
    "i-mingo",
    "i-navajo",
    "i-pwn",
    "i-tao",
    "i-tay",
    "i-tsu",
    "sgn-BE-FR",
    "sgn-BE-NL",
    "sgn-CH-DE",
];

/// Membership in the irregular grandfathered table.
pub(super) struct IrregularTag;

impl IrregularTag {
    /// Whether `tag` is an irregular grandfathered tag, ignoring ASCII case.
    pub(super) fn contains(tag: &str) -> bool {
        IRREGULAR
            .iter()
            .any(|irregular| irregular.eq_ignore_ascii_case(tag))
    }
}
