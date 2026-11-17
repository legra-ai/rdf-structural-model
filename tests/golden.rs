//! Exact JSON and Postcard bytes per term form, so encoding changes are
//! deliberate. Regenerate with `UPDATE_GOLDEN=1 cargo test --test golden`.

use std::fmt::Debug;
use std::path::PathBuf;

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BaseDirection,
    BlankNodeId,
    GraphName,
    LanguageTag,
    Literal,
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
fn blank(label: &str) -> BlankNodeId {
    BlankNodeId::try_new(iri("urn:import:a"), label.to_owned()).unwrap()
}

fn assert_golden<T: Serialize + DeserializeOwned + PartialEq + Debug>(name: &str, value: &T) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden");
    let json_path = directory.join(format!("{name}.json"));
    let binary_path = directory.join(format!("{name}.postcard"));
    let json = serde_json::to_string_pretty(value).unwrap() + "\n";
    let binary = postcard::to_allocvec(value).unwrap();
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&json_path, &json).unwrap();
        std::fs::write(&binary_path, &binary).unwrap();
    }
    let expected_json = std::fs::read_to_string(&json_path).unwrap();
    let expected_binary = std::fs::read(&binary_path).unwrap();
    assert_eq!(json, expected_json, "JSON vector {name} changed");
    assert_eq!(binary, expected_binary, "Postcard vector {name} changed");
    assert_eq!(&serde_json::from_str::<T>(&expected_json).unwrap(), value);
    assert_eq!(&postcard::from_bytes::<T>(&expected_binary).unwrap(), value);
}

#[test]
fn iri_term() {
    assert_golden::<Term>("iri", &Term::Iri(iri("https://example.org/é")));
}

#[test]
fn blank_node_term() {
    assert_golden::<Term>("blank_node", &Term::BlankNode(blank("b0")));
}

#[test]
fn string_literal_term() {
    assert_golden::<Term>("string_literal", &Term::Literal(Literal::string("Alice")));
}

#[test]
fn typed_literal_term() {
    let literal = Literal::typed("005", iri("http://www.w3.org/2001/XMLSchema#integer")).unwrap();
    assert_golden::<Term>("typed_literal", &Term::Literal(literal));
}

#[test]
fn language_literal_term() {
    let literal = Literal::language_tagged("colour", tag("en-GB"));
    assert_golden::<Term>("language_literal", &Term::Literal(literal));
}

#[test]
fn directional_literal_term() {
    let literal = Literal::directional("مرحبا", tag("ar-EG"), BaseDirection::Rtl);
    assert_golden::<Term>("directional_literal", &Term::Literal(literal));
}

#[test]
fn triple_term() {
    let inner = Term::triple(Triple::new(
        Subject::BlankNode(blank("b1")),
        iri("urn:p"),
        Term::Literal(Literal::string("o")),
    ));
    let outer = Term::triple(Triple::new(Subject::Iri(iri("urn:s")), iri("urn:q"), inner));
    assert_golden::<Term>("triple_term", &outer);
}

#[test]
fn quads_in_default_and_named_graphs() {
    let triple: Triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Iri(iri("urn:o")),
    );
    assert_golden(
        "quad_default_graph",
        &Quad::new(triple.clone(), GraphName::Default),
    );
    assert_golden(
        "quad_named_graph",
        &Quad::new(triple, GraphName::BlankNode(blank("g"))),
    );
}
