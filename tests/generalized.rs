//! Generalized statements retain exact terms without weakening standard RDF.
use iri_rfc3987::Iri;
use rdf_structural_model::{
    BlankNodeId,
    GeneralizedGraphName,
    GeneralizedQuad,
    GeneralizedTerm,
    GeneralizedTriple,
    GraphName,
    Literal,
    ModelError,
    Quad,
    Subject,
    Term,
    Triple,
};

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}

fn blank(scope: &str) -> GeneralizedTerm {
    GeneralizedTerm::BlankNode(BlankNodeId::try_new(iri(scope), "b0".to_owned()).unwrap())
}

#[test]
fn literal_subjects_and_scoped_predicates_remain_exact_generalized_terms() {
    let lexical: GeneralizedTerm = GeneralizedTerm::Literal(
        Literal::typed("01", iri("http://www.w3.org/2001/XMLSchema#integer")).unwrap(),
    );
    let triple = GeneralizedTriple::new(lexical.clone(), blank("urn:first"), lexical);
    assert_ne!(triple.predicate(), &blank("urn:second"));
    assert!(matches!(triple.subject(), GeneralizedTerm::Literal(value) if value.lexical() == "01"));
    assert_eq!(triple.subject(), triple.object());
    assert_eq!(Triple::try_from(triple), Err(ModelError::InvalidSubject));
}

#[test]
fn conversions_validate_positions_and_preserve_standard_quad_identity() {
    let triple: Triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Literal(Literal::string("exact")),
    );
    let quad = Quad::new(
        triple,
        GraphName::BlankNode(BlankNodeId::try_new(iri("urn:scope"), "graph".to_owned()).unwrap()),
    );
    let generalized = GeneralizedQuad::from(quad.clone());
    assert_eq!(Quad::try_from(generalized).unwrap(), quad);
    let invalid: GeneralizedTriple = GeneralizedTriple::new(
        GeneralizedTerm::Iri(iri("urn:s")),
        GeneralizedTerm::Literal(Literal::string("predicate")),
        GeneralizedTerm::Iri(iri("urn:o")),
    );
    assert_eq!(
        Triple::try_from(invalid.clone()),
        Err(ModelError::InvalidPredicate)
    );
    assert_eq!(
        Quad::try_from(GeneralizedQuad::new(invalid, GeneralizedGraphName::Default)),
        Err(ModelError::InvalidPredicate)
    );
}

#[test]
fn generalized_values_round_trip_owned_and_borrowed_in_json_and_postcard() {
    let triple: GeneralizedTriple = GeneralizedTriple::new(
        GeneralizedTerm::Literal(Literal::string("subject")),
        blank("urn:predicate"),
        GeneralizedTerm::Literal(Literal::string("object")),
    );
    let value = GeneralizedQuad::new(triple, GeneralizedGraphName::Default);
    let json = serde_json::to_string(&value).unwrap();
    let bytes = postcard::to_allocvec(&value).unwrap();
    assert_eq!(
        serde_json::from_str::<GeneralizedQuad>(&json).unwrap(),
        value
    );
    assert_eq!(
        postcard::from_bytes::<GeneralizedQuad>(&bytes).unwrap(),
        value
    );
    let borrowed_json: GeneralizedQuad<&str> = serde_json::from_str(&json).unwrap();
    let borrowed_bytes: GeneralizedQuad<&str> = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(serde_json::to_string(&borrowed_json).unwrap(), json);
    assert_eq!(serde_json::to_string(&borrowed_bytes).unwrap(), json);
    assert_eq!(serde_json::to_string(&value.as_ref()).unwrap(), json);
    let GeneralizedTerm::Literal(literal) = borrowed_bytes.triple().subject() else {
        panic!("literal subject")
    };
    assert!(literal.lexical().as_ptr() >= bytes.as_ptr());
    assert!(literal.lexical().as_ptr() < bytes.as_ptr().wrapping_add(bytes.len()));
    let with_unknown = format!("{},\"unexpected\":true}}", json.strip_suffix('}').unwrap());
    assert!(serde_json::from_str::<GeneralizedQuad>(&with_unknown).is_err());
}

#[test]
fn nested_and_explicit_extension_terms_keep_their_own_identity() {
    #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    enum Extra {
        Future(u64),
    }
    let nested = Term::triple(Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Literal(Literal::string("nested")),
    ));
    let value = GeneralizedTriple::new(
        GeneralizedTerm::from(nested),
        GeneralizedTerm::Extension(Extra::Future(7)),
        GeneralizedTerm::Iri(iri("urn:o")),
    );
    let encoded = postcard::to_allocvec(&value).unwrap();
    assert_eq!(
        postcard::from_bytes::<GeneralizedTriple<String, Extra>>(&encoded).unwrap(),
        value
    );
    assert!(postcard::from_bytes::<GeneralizedTriple>(&encoded).is_err());
    assert_eq!(Triple::try_from(value), Err(ModelError::InvalidSubject));
}

#[test]
fn generalized_nesting_and_literal_graph_names_round_trip_and_reject_standard_projection() {
    let inner: GeneralizedTriple = GeneralizedTriple::new(
        GeneralizedTerm::Literal(Literal::string("inner subject")),
        blank("urn:predicate"),
        GeneralizedTerm::Iri(iri("urn:o")),
    );
    let term = GeneralizedTerm::Triple(Box::new(inner));
    let triple = GeneralizedTriple::new(term.clone(), term.clone(), term);
    let quad = GeneralizedQuad::new(
        triple,
        GeneralizedGraphName::Term(GeneralizedTerm::Literal(Literal::string("graph"))),
    );
    let json = serde_json::to_string(&quad).unwrap();
    let bytes = postcard::to_allocvec(&quad).unwrap();
    assert_eq!(
        serde_json::from_str::<GeneralizedQuad>(&json).unwrap(),
        quad
    );
    assert_eq!(
        postcard::from_bytes::<GeneralizedQuad>(&bytes).unwrap(),
        quad
    );
    assert_eq!(Quad::try_from(quad), Err(ModelError::InvalidSubject));
    let valid: Triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Iri(iri("urn:o")),
    );
    let bad_graph = GeneralizedQuad::new(
        valid.into(),
        GeneralizedGraphName::Term(GeneralizedTerm::Literal(Literal::string("graph"))),
    );
    assert_eq!(Quad::try_from(bad_graph), Err(ModelError::InvalidGraphName));
}

#[test]
fn nested_object_conversion_recursively_checks_standard_positions() {
    let standard: Term = Term::triple(Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Literal(Literal::string("nested")),
    ));
    assert_eq!(
        Term::try_from(GeneralizedTerm::from(standard.clone())).unwrap(),
        standard
    );
    let invalid: GeneralizedTriple = GeneralizedTriple::new(
        GeneralizedTerm::Literal(Literal::string("invalid subject")),
        GeneralizedTerm::Iri(iri("urn:p")),
        GeneralizedTerm::Iri(iri("urn:o")),
    );
    let outer = GeneralizedTriple::new(
        GeneralizedTerm::Iri(iri("urn:s")),
        GeneralizedTerm::Iri(iri("urn:p")),
        GeneralizedTerm::Triple(Box::new(invalid.clone())),
    );
    assert_eq!(Triple::try_from(outer), Err(ModelError::InvalidSubject));
    assert_eq!(
        GraphName::try_from(GeneralizedGraphName::Term(GeneralizedTerm::Triple(
            Box::new(invalid)
        ))),
        Err(ModelError::InvalidGraphName)
    );
}

#[test]
fn explicit_extensions_round_trip_without_requiring_clone() {
    #[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Extra(u8);
    let standard = Quad::new(
        Triple::new(
            Subject::Extension(Extra(1)),
            iri("urn:p"),
            Term::Extension(Extra(2)),
        ),
        GraphName::Extension(Extra(3)),
    );
    let expected = serde_json::to_string(&standard).unwrap();
    let generalized = GeneralizedQuad::from(standard);
    let restored: Quad<String, Extra> = generalized.try_into().unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), expected);
}
