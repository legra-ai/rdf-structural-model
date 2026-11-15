//! RDF 1.2 literal structure retains payloads for later datatype
//! interpretation.
//!
//! Semantics CRD 2026-09-24 section 7.1 uses XML 1.1 Char for xsd:string.
//! Ill-typed lexical forms must still be representable; this model is not the
//! datatype evaluator. Language and directional literals retain their payloads
//! independently of the xsd:string lexical space.

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

fn assert_round_trip(literal: &Literal) {
    let json = serde_json::to_string(literal).unwrap();
    let decoded: Literal = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, *literal);
    assert_eq!(decoded.lexical().as_bytes(), literal.lexical().as_bytes());
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);

    let bytes = postcard::to_allocvec(literal).unwrap();
    let decoded: Literal = postcard::from_bytes(&bytes).unwrap();
    let borrowed: Literal<&str> = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, *literal);
    assert_eq!(borrowed, literal.as_ref());
    assert_eq!(borrowed.lexical().as_bytes(), literal.lexical().as_bytes());
    assert_eq!(serde_json::to_string(&borrowed).unwrap(), json);
}

#[test]
fn unicode_boundary_payloads_survive_owned_and_borrowed_serde_without_datatype_evaluation() {
    // bounded: sixteen scalar-boundary fixtures, each at most four UTF-8 bytes.
    // NUL, FFFE and FFFF are outside the RDF 1.2 xsd:string lexical space;
    // nonzero C0 controls are inside XML 1.1 Char. All remain structural data.
    for payload in [
        "",
        "\0",
        "\u{1}",
        "\t",
        "\n",
        "\u{b}",
        "\u{c}",
        "\r",
        "\u{1f}",
        "\u{d7ff}",
        "\u{e000}",
        "\u{fffd}",
        "\u{fffe}",
        "\u{ffff}",
        "\u{10000}",
        "\u{10ffff}",
    ] {
        // bounded: the five literal forms used by the datatype adapter.
        let literals = [
            Literal::string(payload),
            Literal::typed(payload, Iri::parse("urn:unknown-datatype").unwrap().into()).unwrap(),
            Literal::language_tagged(payload, LanguageTag::try_new("EN-us".to_owned()).unwrap()),
            Literal::directional(
                payload,
                LanguageTag::try_new("EN-us".to_owned()).unwrap(),
                BaseDirection::Ltr,
            ),
            Literal::directional(
                payload,
                LanguageTag::try_new("EN-us".to_owned()).unwrap(),
                BaseDirection::Rtl,
            ),
        ];
        for literal in literals {
            assert_round_trip(&literal);
        }
    }
}

fn iri(text: &str) -> Iri<String> {
    Iri::parse(text).unwrap().into()
}

#[test]
fn nested_boundary_literals_keep_their_graph_scope_and_direction_in_both_formats() {
    // bounded: four exceptional payloads across all three standard graph forms.
    for payload in ["\0", "\u{1}", "\u{fffe}", "\u{ffff}"] {
        let inner: Triple = Triple::new(
            Subject::BlankNode(
                BlankNodeId::try_new(iri("urn:input-scope"), "b0".to_owned()).unwrap(),
            ),
            iri("urn:predicate"),
            Term::Literal(Literal::directional(
                payload,
                LanguageTag::try_new("EN-us".to_owned()).unwrap(),
                BaseDirection::Rtl,
            )),
        );
        let reification = Triple::new(
            Subject::Iri(iri("urn:reifier")),
            iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies"),
            Term::triple(inner),
        );
        for graph in [
            GraphName::Default,
            GraphName::Iri(iri("urn:named-graph")),
            GraphName::BlankNode(
                BlankNodeId::try_new(iri("urn:graph-scope"), "b0".to_owned()).unwrap(),
            ),
        ] {
            let quad = Quad::new(reification.clone(), graph);
            let json = serde_json::to_string(&quad).unwrap();
            assert_eq!(serde_json::from_str::<Quad>(&json).unwrap(), quad);
            let bytes = postcard::to_allocvec(&quad).unwrap();
            assert_eq!(postcard::from_bytes::<Quad>(&bytes).unwrap(), quad);
            let borrowed: Quad<&str> = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(serde_json::to_string(&borrowed).unwrap(), json);
            assert_eq!(serde_json::to_string(&quad.as_ref()).unwrap(), json);
        }
    }
}
