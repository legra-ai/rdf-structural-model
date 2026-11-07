//! Boundary cases from the documented RDF identity and Serde contracts.

use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BaseDirection,
    BlankNodeId,
    LanguageTag,
    Literal,
    LiteralFacets,
    LiteralMetadata,
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
    Subject,
    Term,
    Triple,
};
use serde::Serialize;

fn iri(text: &str) -> Iri<String> {
    Iri::parse(text).unwrap().into()
}

fn hash(value: &impl Hash) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn rdf_iri_identity_does_not_apply_uri_normalization() {
    for (first, second) in [
        ("http://EXAMPLE.org/a", "http://example.org/a"),
        ("http://example.org/%61", "http://example.org/a"),
        ("http://example.org/a/../b", "http://example.org/b"),
        ("urn:é", "urn:e\u{301}"),
    ] {
        let first: Term = Term::Iri(iri(first));
        let second: Term = Term::Iri(iri(second));
        assert_ne!(first, second);
        for value in [first, second] {
            assert_eq!(hash(&value), hash(&value.as_ref()));
            let bytes = postcard::to_allocvec(&value).unwrap();
            assert_eq!(postcard::from_bytes::<Term>(&bytes).unwrap(), value);
        }
    }
}

#[test]
fn extension_payloads_and_private_use_subtags_are_not_duplicate_variants() {
    for text in [
        "en-a-abcde-abcde",
        "en-a-abcde-b-abcde",
        "en-x-1901-1901",
        "x-a-a",
    ] {
        let tag = LanguageTag::try_new(text.to_owned()).unwrap();
        let json = serde_json::to_string(&tag).unwrap();
        let bytes = postcard::to_allocvec(&tag).unwrap();
        assert_eq!(serde_json::from_str::<LanguageTag>(&json).unwrap(), tag);
        assert_eq!(postcard::from_bytes::<LanguageTag>(&bytes).unwrap(), tag);
    }
    for text in ["sl-rozaj-ROZAJ", "en-a-abc-B-def-A-ghi"] {
        assert!(LanguageTag::try_new(text.to_owned()).is_err());
        let bytes = postcard::to_allocvec(text).unwrap();
        assert!(postcard::from_bytes::<LanguageTag>(&bytes).is_err());
    }
}

#[derive(Serialize)]
struct LiteralInput {
    lexical: String,
    metadata: LiteralMetadata,
}

#[test]
fn every_standard_datatype_facet_combination_is_checked_in_both_formats() {
    for datatype in ["urn:custom", RDF_LANG_STRING, RDF_DIR_LANG_STRING] {
        for facets in [
            LiteralFacets::None,
            LiteralFacets::Language(LanguageTag::try_new("en".to_owned()).unwrap()),
            LiteralFacets::Directional {
                language: LanguageTag::try_new("en".to_owned()).unwrap(),
                direction: BaseDirection::Ltr,
            },
        ] {
            let valid = matches!(
                (datatype, &facets),
                ("urn:custom", LiteralFacets::None)
                    | (RDF_LANG_STRING, LiteralFacets::Language(_))
                    | (RDF_DIR_LANG_STRING, LiteralFacets::Directional { .. })
            );
            let metadata = LiteralMetadata::new(iri(datatype), facets);
            assert_eq!(
                Literal::try_new("text".to_owned(), metadata.clone()).is_ok(),
                valid
            );
            let input = LiteralInput {
                lexical: "text".to_owned(),
                metadata,
            };
            let json = serde_json::to_string(&input).unwrap();
            let bytes = postcard::to_allocvec(&input).unwrap();
            assert_eq!(serde_json::from_str::<Literal>(&json).is_ok(), valid);
            assert_eq!(postcard::from_bytes::<Literal>(&bytes).is_ok(), valid);
        }
    }
}

#[test]
fn nested_round_trips_preserve_scope_lexical_form_and_language_spelling() {
    let literal = Literal::directional(
        "line\n\"quote\"\\é",
        LanguageTag::try_new("EN-us".to_owned()).unwrap(),
        BaseDirection::Rtl,
    );
    let value: Term = Term::triple(Triple::new(
        Subject::BlankNode(
            BlankNodeId::try_new(iri("urn:scope"), "same-label".to_owned()).unwrap(),
        ),
        iri("urn:predicate"),
        Term::Literal(literal.clone()),
    ));
    let json = serde_json::to_string(&value).unwrap();
    let bytes = postcard::to_allocvec(&value).unwrap();
    assert_eq!(serde_json::from_str::<Term>(&json).unwrap(), value);
    let borrowed: Term<&str> = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(serde_json::to_string(&borrowed).unwrap(), json);
    assert_eq!(hash(&borrowed), hash(&value));
    let Term::Triple(triple) = borrowed else {
        panic!("triple term")
    };
    let Term::Literal(decoded) = triple.object() else {
        panic!("literal object")
    };
    assert_eq!(decoded.lexical(), literal.lexical());
    let LiteralFacets::Directional { language, .. } = decoded.metadata().facets() else {
        panic!("directional facets")
    };
    assert_eq!(language.as_str(), "EN-us");
    assert!(decoded.lexical().as_ptr() >= bytes.as_ptr());
    assert!(decoded.lexical().as_ptr() < bytes.as_ptr().wrapping_add(bytes.len()));
}
