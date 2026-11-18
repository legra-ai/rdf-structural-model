//! `Reification` recognizes and rebuilds reifying triples (RDF 1.2 Concepts
//! §1.5) structurally, owned and borrowed, in JSON and Postcard.

mod common;

use common::strategy;
use iri_rfc3987::Iri;
use proptest::prelude::*;
use rdf_structural_model::{
    BaseDirection,
    BlankNodeId,
    Literal,
    ModelError,
    Quad,
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
    RDF_NAMESPACE,
    RDF_REIFIES,
    Reification,
    Subject,
    Term,
    Triple,
};

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}

fn reifier() -> Subject {
    Subject::BlankNode(BlankNodeId::try_new(iri("urn:import:a"), "r".to_owned()).unwrap())
}

fn reified() -> Triple {
    Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Literal(Literal::string("o")),
    )
}

/// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification.
#[test]
fn reifying_triple_converts_to_reification_and_back() {
    let reifying: Triple = Triple::new(reifier(), iri(RDF_REIFIES), Term::triple(reified()));
    let reification = Reification::try_from(reifying.clone()).unwrap();
    assert_eq!(reification.reifier(), &reifier());
    assert_eq!(reification.triple(), &reified());
    assert_eq!(Triple::from(reification), reifying);
}

/// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification.
#[test]
fn borrowed_reification_rebuilds_the_borrowed_reifying_triple() {
    let reification: Reification = Reification::new(reifier(), reified());
    let owned: Triple = reification.clone().into();
    let borrowed: Triple<&str, &_> = reification.as_ref().into();
    assert_eq!(borrowed, owned.as_ref());
    assert_eq!(
        Reification::try_from(borrowed).unwrap(),
        reification.as_ref()
    );
}

/// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification.
#[test]
fn a_predicate_other_than_rdf_reifies_is_refused() {
    for predicate in [
        "urn:p",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#Reifies",
    ] {
        let triple: Triple = Triple::new(reifier(), iri(predicate), Term::triple(reified()));
        assert_eq!(
            Reification::try_from(triple),
            Err(ModelError::NotReifyingPredicate)
        );
    }
}

/// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification, §3.6 Triple Terms.
#[test]
fn a_non_triple_term_object_is_refused() {
    let objects = [
        Term::Iri(iri("urn:o")),
        Term::BlankNode(BlankNodeId::try_new(iri("urn:import:a"), "b".to_owned()).unwrap()),
        Term::Literal(Literal::string("<<( urn:s urn:p urn:o )>>")),
    ];
    for object in objects {
        let triple: Triple = Triple::new(reifier(), iri(RDF_REIFIES), object);
        assert_eq!(
            Reification::try_from(triple),
            Err(ModelError::ReifiedObjectNotTripleTerm)
        );
    }
}

/// Spec: RDF 1.2 Concepts §1.4 RDF Vocabularies and Namespace IRIs.
#[test]
fn rdf_vocabulary_constants_share_the_rdf_namespace() {
    for constant in [RDF_REIFIES, RDF_LANG_STRING, RDF_DIR_LANG_STRING] {
        let local = constant.strip_prefix(RDF_NAMESPACE).unwrap();
        assert!(!local.is_empty() && !local.contains(['#', '/']));
    }
}

/// Spec: RDF 1.2 Concepts §3.4 Literals ("The textual representation of base
/// direction MUST be lowercase", added after CR 2026-04-07).
#[test]
fn base_direction_is_lowercase_only() {
    assert_eq!(
        serde_json::to_string(&BaseDirection::Rtl).unwrap(),
        r#""rtl""#
    );
    for upper in [r#""LTR""#, r#""Rtl""#] {
        assert!(serde_json::from_str::<BaseDirection>(upper).is_err());
    }
    assert_eq!(BaseDirection::Ltr.as_str(), "ltr");
}

proptest! {
    /// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification.
    #[test]
    fn reifications_round_trip_through_triples_and_serde(
        reifier in strategy::subject(),
        triple in strategy::triple(),
    ) {
        let reification = Reification::new(reifier, triple);
        let reifying: Triple = reification.clone().into();
        prop_assert_eq!(reifying.predicate().as_str(), RDF_REIFIES);
        prop_assert_eq!(&Reification::try_from(reifying).unwrap(), &reification);
        let json = serde_json::to_string(&reification).unwrap();
        prop_assert_eq!(&serde_json::from_str::<Reification>(&json).unwrap(), &reification);
        let borrowed: Reification<&str> = serde_json::from_str(&json).unwrap();
        let binary = postcard::to_allocvec(&reification).unwrap();
        prop_assert_eq!(postcard::to_allocvec(&borrowed).unwrap(), binary.clone());
        prop_assert_eq!(postcard::to_allocvec(&reification.as_ref()).unwrap(), binary.clone());
        prop_assert_eq!(&postcard::from_bytes::<Reification>(&binary).unwrap(), &reification);
    }

    /// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification.
    #[test]
    fn only_reifying_triples_convert(triple in strategy::triple()) {
        let reifying = triple.predicate().as_str() == RDF_REIFIES
            && matches!(triple.object(), Term::Triple(_));
        prop_assert_eq!(Reification::try_from(triple).is_ok(), reifying);
    }

    /// Spec: RDF 1.2 Concepts §1.5 Triple Terms and Reification, §4.3 Quads.
    #[test]
    fn reifying_a_quad_statement_keeps_its_graph(quad in strategy::quad()) {
        let (triple, graph) = quad.into_parts();
        let reifier = triple.subject().clone();
        let reifying: Triple = Reification::new(reifier.clone(), triple.clone()).into();
        let rebuilt = Quad::new(reifying, graph.clone());
        prop_assert_eq!(rebuilt.graph(), &graph);
        let reification = Reification::try_from(rebuilt.into_parts().0).unwrap();
        prop_assert_eq!(reification.into_parts(), (reifier, triple));
    }
}
