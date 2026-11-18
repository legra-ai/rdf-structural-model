//! Arbitrary nested terms round-trip through JSON and Postcard, owned and
//! borrowed, up to and including `MAX_TRIPLE_TERM_DEPTH`.

mod common;

use common::strategy;
use iri_rfc3987::Iri;
use proptest::prelude::*;
use rdf_structural_model::{
    MAX_TRIPLE_TERM_DEPTH,
    Quad,
    Subject,
    Term,
    Triple,
};

fn depth(term: &Term) -> usize {
    match term {
        Term::Triple(triple) => 1 + depth(triple.object()),
        _ => 0,
    }
}

/// A generated term wrapped in extra triple terms, never past the limit.
fn deep_term() -> impl Strategy<Value = Term> {
    (strategy::term(), any::<usize>()).prop_map(|(term, extra)| {
        let room = MAX_TRIPLE_TERM_DEPTH.get() - depth(&term);
        (0..extra % (room + 1)).fold(term, |object, _| {
            Term::triple(Triple::new(
                Subject::Iri(Iri::parse("urn:s").unwrap().into()),
                Iri::parse("urn:p").unwrap().into(),
                object,
            ))
        })
    })
}

proptest! {
    /// Spec: RDF 1.2 Concepts §3.6 Triple Terms.
    #[test]
    fn terms_round_trip_owned_and_borrowed(term in deep_term()) {
        prop_assert!(depth(&term) <= MAX_TRIPLE_TERM_DEPTH.get());
        let json = serde_json::to_string(&term).unwrap();
        let bytes = postcard::to_allocvec(&term).unwrap();
        prop_assert_eq!(&serde_json::from_str::<Term>(&json).unwrap(), &term);
        prop_assert_eq!(&postcard::from_bytes::<Term>(&bytes).unwrap(), &term);
        let from_json: Term<&str> = serde_json::from_str(&json).unwrap();
        let from_bytes: Term<&str> = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(&from_json, &from_bytes);
        prop_assert_eq!(postcard::to_allocvec(&from_json).unwrap(), bytes.clone());
        prop_assert_eq!(postcard::to_allocvec(&term.as_ref()).unwrap(), bytes.clone());
        prop_assert_eq!(serde_json::to_string(&from_bytes).unwrap(), json);
    }

    /// Spec: RDF 1.2 Concepts §4.3 Quads.
    #[test]
    fn quads_round_trip_owned_and_borrowed(quad in strategy::quad()) {
        let json = serde_json::to_string(&quad).unwrap();
        let bytes = postcard::to_allocvec(&quad).unwrap();
        prop_assert_eq!(&serde_json::from_str::<Quad>(&json).unwrap(), &quad);
        prop_assert_eq!(&postcard::from_bytes::<Quad>(&bytes).unwrap(), &quad);
        let from_json: Quad<&str> = serde_json::from_str(&json).unwrap();
        let from_bytes: Quad<&str> = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(&from_json, &from_bytes);
        prop_assert_eq!(postcard::to_allocvec(&from_json).unwrap(), bytes.clone());
        prop_assert_eq!(postcard::to_allocvec(&quad.as_ref()).unwrap(), bytes.clone());
        prop_assert_eq!(serde_json::to_string(&from_bytes).unwrap(), json);
    }
}

/// Spec: RDF 1.2 Concepts §3.6 Triple Terms.
#[test]
fn a_term_at_exactly_the_limit_round_trips() {
    let term = (0..MAX_TRIPLE_TERM_DEPTH.get()).fold(
        Term::iri(Iri::parse("urn:o").unwrap().into()),
        |object, _| {
            Term::triple(Triple::new(
                Subject::Iri(Iri::parse("urn:s").unwrap().into()),
                Iri::parse("urn:p").unwrap().into(),
                object,
            ))
        },
    );
    let bytes = postcard::to_allocvec(&term).unwrap();
    assert_eq!(postcard::from_bytes::<Term>(&bytes).unwrap(), term);
    let json = serde_json::to_string(&term).unwrap();
    let borrowed: Term<&str> = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&borrowed).unwrap(), json);
}
