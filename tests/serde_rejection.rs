//! Malformed input cannot bypass model invariants, including in nested values.

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BlankNodeId,
    LanguageTag,
    Literal,
    LiteralFacets,
    LiteralMetadata,
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}
fn rejected<T: DeserializeOwned, S: Serialize>(value: &S) {
    assert!(serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).is_err());
    assert!(postcard::from_bytes::<T>(&postcard::to_allocvec(value).unwrap()).is_err());
}

#[derive(Serialize)]
struct BlankInput {
    scope: Iri<String>,
    label: String,
}
#[derive(Serialize)]
struct LiteralInput {
    lexical: String,
    metadata: LiteralMetadata,
}

#[test]
fn invalid_language_tags_are_rejected_in_text_and_binary() {
    for tag in ["", "en--US", "de-1901-1901", "en-a-abc-A-def"] {
        rejected::<LanguageTag, _>(&tag);
    }
}

#[test]
fn empty_blank_node_labels_are_rejected_in_text_and_binary() {
    rejected::<BlankNodeId, _>(&BlankInput {
        scope: iri("urn:scope"),
        label: String::new(),
    });
}

#[test]
fn language_datatypes_require_matching_facets_on_construction_and_decoding() {
    for datatype in [RDF_LANG_STRING, RDF_DIR_LANG_STRING] {
        let metadata = LiteralMetadata::new(iri(datatype), LiteralFacets::None);
        assert!(Literal::try_new("text".to_owned(), metadata.clone()).is_err());
        rejected::<Literal, _>(&LiteralInput {
            lexical: "text".to_owned(),
            metadata,
        });
    }
    let metadata = LiteralMetadata::new(
        iri("urn:custom"),
        LiteralFacets::Language(LanguageTag::try_new("en".to_owned()).unwrap()),
    );
    assert!(Literal::try_new("text".to_owned(), metadata.clone()).is_err());
    rejected::<Literal, _>(&LiteralInput {
        lexical: "text".to_owned(),
        metadata,
    });
}

#[test]
fn nested_malformed_language_and_unknown_fields_are_rejected() {
    let malformed = r#"{"lexical":"text","metadata":{"datatype":"http://www.w3.org/1999/02/22-rdf-syntax-ns#langString","facets":{"Language":"en--US"}}}"#;
    assert!(serde_json::from_str::<Literal>(malformed).is_err());
    let unknown =
        r#"{"lexical":"text","metadata":{"datatype":"urn:custom","facets":"None","unknown":true}}"#;
    assert!(serde_json::from_str::<Literal>(unknown).is_err());
    rejected::<BlankNodeId, _>(&BlankInput {
        scope: iri("urn:scope"),
        label: String::new(),
    });
}
