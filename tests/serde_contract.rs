//! The same checked semantic model works with owned and borrowed decoding.

use std::fmt::Debug;
use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BaseDirection,
    BlankNodeId,
    GraphName,
    LanguageTag,
    Literal,
    LiteralFacets,
    LiteralMetadata,
    Quad,
    Subject,
    Term,
    Triple,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}
fn tag(value: &str) -> LanguageTag {
    LanguageTag::try_new(value.to_owned()).unwrap()
}
fn blank(scope: &str) -> BlankNodeId {
    BlankNodeId::try_new(iri(scope), "b0".to_owned()).unwrap()
}
fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let json = serde_json::to_string(value).unwrap();
    let bytes = postcard::to_allocvec(value).unwrap();
    assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value);
    assert_eq!(&postcard::from_bytes::<T>(&bytes).unwrap(), value);
}

#[test]
fn standard_terms_and_all_literal_forms_round_trip() {
    // bounded: all non-recursive standard term forms, including all literal
    // categories.
    let terms: [Term; 5] = [
        Term::Iri(iri("urn:s")),
        Term::BlankNode(blank("urn:scope")),
        Term::Literal(Literal::typed("005", iri("urn:custom-datatype")).unwrap()),
        Term::Literal(Literal::language_tagged("bonjour", tag("fr"))),
        Term::Literal(Literal::directional("مرحبا", tag("ar"), BaseDirection::Rtl)),
    ];
    for term in terms {
        round_trip(&term);
    }
}

#[test]
fn nested_triples_and_every_standard_graph_position_round_trip() {
    let nested: Triple = Triple::new(
        Subject::BlankNode(blank("urn:scope")),
        iri("urn:p"),
        Term::Literal(Literal::string("object")),
    );
    let triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:reifies"),
        Term::triple(nested),
    );
    for graph in [
        GraphName::Default,
        GraphName::Iri(iri("urn:graph")),
        GraphName::BlankNode(blank("urn:graph-scope")),
    ] {
        round_trip(&Quad::new(triple.clone(), graph));
    }
}

#[test]
fn borrowed_literals_decode_without_copying_lexical_text() {
    let expected = Literal::directional("hello", tag("en-US"), BaseDirection::Ltr);
    let json = serde_json::to_string(&expected).unwrap();
    let bytes = postcard::to_allocvec(&expected).unwrap();
    let from_json: Literal<&str> = serde_json::from_str(&json).unwrap();
    let from_binary: Literal<&str> = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(from_json, expected.as_ref());
    assert_eq!(from_binary, expected.as_ref());
    assert!(from_json.lexical().as_ptr() >= json.as_ptr());
    assert!(from_json.lexical().as_ptr() < json.as_ptr().wrapping_add(json.len()));
    assert!(from_binary.lexical().as_ptr() >= bytes.as_ptr());
    assert!(from_binary.lexical().as_ptr() < bytes.as_ptr().wrapping_add(bytes.len()));
}

#[test]
fn borrowed_nested_terms_share_the_owned_serialization_shape() {
    let value: Term = Term::triple(Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Literal(Literal::string("payload")),
    ));
    let json = serde_json::to_string(&value).unwrap();
    let bytes = postcard::to_allocvec(&value).unwrap();
    let from_json: Term<&str> = serde_json::from_str(&json).unwrap();
    let from_binary: Term<&str> = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(serde_json::to_string(&from_json).unwrap(), json);
    assert_eq!(serde_json::to_string(&from_binary).unwrap(), json);
    assert_eq!(serde_json::to_string(&value.as_ref()).unwrap(), json);
}

#[test]
fn case_insensitive_tag_equality_has_matching_hashes() {
    let first = Literal::language_tagged("text", tag("EN-us"));
    let second = Literal::language_tagged("text", tag("en-US"));
    assert_eq!(first, second);
    assert_eq!(hash(&first), hash(&second));
    assert_eq!(hash(&first), hash(&first.as_ref()));
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum Extra {
    BinaryLiteral {
        metadata: LiteralMetadata,
        bytes: Vec<u8>,
    },
}

#[test]
fn typed_binary_extensions_round_trip_and_standard_decoders_reject_them() {
    // bounded: three payload bytes exercising values outside UTF-8.
    let value: Term<String, Extra> = Term::Extension(Extra::BinaryLiteral {
        metadata: LiteralMetadata::new(
            iri("urn:mime:application/octet-stream"),
            LiteralFacets::Directional {
                language: tag("en"),
                direction: BaseDirection::Ltr,
            },
        ),
        bytes: vec![0, 255, 128],
    });
    round_trip(&value);
    assert!(serde_json::from_str::<Term>(&serde_json::to_string(&value).unwrap()).is_err());
    assert!(postcard::from_bytes::<Term>(&postcard::to_allocvec(&value).unwrap()).is_err());
}

#[test]
fn standard_subjects_reject_literal_and_triple_term_positions() {
    assert!(
        Subject::try_from(Term::<String>::Literal(Literal::string("invalid subject"))).is_err()
    );
    let triple: Triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Iri(iri("urn:o")),
    );
    assert!(Subject::try_from(Term::triple(triple)).is_err());
    assert!(serde_json::from_str::<Subject>(r#"{"Literal":"wrong position"}"#).is_err());
}
