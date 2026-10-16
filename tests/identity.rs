//! Identity invariants are shared independently of a backend.

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BaseDirection,
    BlankNodeId,
    LanguageTag,
    Literal,
    Term,
};

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}

#[test]
fn literal_identity_preserves_lexical_form() {
    let datatype = iri("http://www.w3.org/2001/XMLSchema#integer");
    assert_ne!(
        Literal::typed("005", datatype.clone()).unwrap(),
        Literal::typed("5", datatype).unwrap()
    );
}

#[test]
fn language_identity_is_case_insensitive_and_direction_sensitive() {
    let upper = Literal::language_tagged("text", LanguageTag::try_new("EN-us".to_owned()).unwrap());
    let lower = Literal::language_tagged("text", LanguageTag::try_new("en-US".to_owned()).unwrap());
    assert_eq!(upper, lower);
    assert_ne!(
        Literal::directional(
            "text",
            LanguageTag::try_new("en".to_owned()).unwrap(),
            BaseDirection::Ltr
        ),
        Literal::directional(
            "text",
            LanguageTag::try_new("en".to_owned()).unwrap(),
            BaseDirection::Rtl
        ),
    );
}

#[test]
fn blank_node_labels_are_scoped() {
    let a = BlankNodeId::try_new(iri("urn:source:a"), "b0".to_owned()).unwrap();
    let b = BlankNodeId::try_new(iri("urn:source:b"), "b0".to_owned()).unwrap();
    assert_ne!(a, b);
    assert!(BlankNodeId::try_new(iri("urn:source:a"), String::new()).is_err());
    let _: Term = Term::BlankNode(a);
}

#[test]
fn well_formed_language_tags_include_private_use_and_grandfathered_forms() {
    for tag in [
        "en",
        "zh-Hant-TW",
        "x-private",
        "i-klingon",
        "en-a-abc-x-private",
    ] {
        assert!(LanguageTag::try_new(tag.to_owned()).is_ok(), "{tag}");
    }
    for tag in [
        "",
        "e",
        "en_uk",
        "en--uk",
        "en-123456789",
        "de-1901-1901",
        "en-a-abc-a-def",
    ] {
        assert!(LanguageTag::try_new(tag.to_owned()).is_err(), "{tag}");
    }
}
