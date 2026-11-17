//! Language-tag errors name the failing part; accessors expose the parts.

use rdf_structural_model::{
    LanguageTag,
    LanguageTagErrorReason as Reason,
    ModelError,
};

fn rejection(text: &str) -> (Reason, usize) {
    match LanguageTag::try_new(text) {
        Err(ModelError::InvalidLanguageTag(error)) => (error.reason(), error.subtag().get()),
        other => panic!("{text}: expected a language-tag error, got {other:?}"),
    }
}

#[test]
fn rejections_name_the_reason_and_subtag() {
    // bounded: one case per rejection reason.
    let cases = [
        ("", Reason::EmptySubtag, 0),
        ("en--US", Reason::EmptySubtag, 1),
        ("en-", Reason::EmptySubtag, 1),
        ("en-U_S", Reason::InvalidCharacter, 1),
        ("en-abcdefghi", Reason::SubtagTooLong, 1),
        ("e", Reason::InvalidPrimaryLanguage, 0),
        ("1en", Reason::InvalidPrimaryLanguage, 0),
        ("en-US-ab", Reason::UnexpectedSubtag, 2),
        ("en-Latn-Cyrl", Reason::UnexpectedSubtag, 2),
        ("de-1996-1996", Reason::DuplicateVariant, 2),
        ("sl-rozaj-ROZAJ", Reason::DuplicateVariant, 2),
        ("en-a-bbb-A-ccc", Reason::DuplicateExtensionSingleton, 3),
        ("en-a", Reason::EmptyExtension, 1),
        ("en-a-b-cc", Reason::EmptyExtension, 1),
        ("x", Reason::EmptyPrivateUse, 0),
        ("en-x", Reason::EmptyPrivateUse, 1),
    ];
    for (text, reason, subtag) in cases {
        assert_eq!(rejection(text), (reason, subtag), "{text}");
    }
}

#[test]
fn private_use_subtags_are_one_to_eight_alphanumerics() {
    // RFC 5646 §2.1: privateuse = "x" 1*("-" (1*8alphanum)).
    for text in ["x-a", "x-abcdefgh", "en-x-1", "en-x-a-12345678", "en-x-a-a"] {
        assert!(LanguageTag::try_new(text).is_ok(), "{text}");
    }
    assert_eq!(rejection("x-abcdefghi"), (Reason::SubtagTooLong, 1));
    assert_eq!(rejection("en-x-ab-abcdefghi"), (Reason::SubtagTooLong, 3));
}

#[test]
fn extension_subtags_are_two_to_eight_alphanumerics() {
    assert!(LanguageTag::try_new("en-a-bb-12345678").is_ok());
    assert_eq!(rejection("en-a-bb-123456789"), (Reason::SubtagTooLong, 3));
}

#[test]
fn accessors_return_original_spelling() {
    let tag = LanguageTag::try_new("zh-yue-Hant-HK-1996-fonipa-u-co-x-priv").unwrap();
    assert_eq!(tag.primary(), Some("zh"));
    assert_eq!(tag.script(), Some("Hant"));
    assert_eq!(tag.region(), Some("HK"));
    assert_eq!(tag.variants().collect::<Vec<_>>(), ["1996", "fonipa"]);

    let numeric = LanguageTag::try_new("es-419").unwrap();
    assert_eq!(
        (numeric.primary(), numeric.script(), numeric.region()),
        (Some("es"), None, Some("419"))
    );
    assert_eq!(numeric.variants().count(), 0);
}

#[test]
fn private_use_and_irregular_tags_have_no_language_parts() {
    for text in ["x-whatever", "i-klingon", "EN-gb-OED", "sgn-BE-FR"] {
        let tag = LanguageTag::try_new(text).unwrap();
        assert_eq!(
            (tag.primary(), tag.script(), tag.region()),
            (None, None, None)
        );
        assert_eq!(tag.variants().count(), 0, "{text}");
    }
    let regular = LanguageTag::try_new("zh-min-nan").unwrap();
    assert_eq!(regular.primary(), Some("zh"));
}

#[test]
fn error_message_names_reason_and_position() {
    let error = LanguageTag::try_new("en-US-ab").unwrap_err();
    assert_eq!(
        error.to_string(),
        "language tag is not well-formed BCP 47: subtag does not fit its position at subtag 2"
    );
}
