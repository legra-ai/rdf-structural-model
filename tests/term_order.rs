//! The hand-written order is total and consistent with `Eq` and `Hash`.

mod common;

use std::cmp::Ordering;
use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use common::strategy;
use iri_rfc3987::Iri;
use proptest::prelude::*;
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

fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn assert_consistent<T: Ord + Hash + std::fmt::Debug>(a: &T, b: &T, c: &T) {
    assert_eq!(a.cmp(b) == Ordering::Equal, a == b, "{a:?} vs {b:?}");
    assert_eq!(a.cmp(b), b.cmp(a).reverse());
    assert_eq!(a.partial_cmp(b), Some(a.cmp(b)));
    if a == b {
        assert_eq!(hash(a), hash(b));
    }
    if a <= b && b <= c {
        assert!(a <= c, "transitivity: {a:?} <= {b:?} <= {c:?}");
    }
}

fn assert_sorting_is_canonical<T: Ord + Clone + std::fmt::Debug>(mut values: Vec<T>) {
    let mut reversed: Vec<T> = values.iter().rev().cloned().collect();
    values.sort();
    reversed.sort();
    assert!(values.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(values, reversed);
}

proptest! {
    #[test]
    fn term_order_is_total_and_consistent(
        a in strategy::term(), b in strategy::term(), c in strategy::term()
    ) {
        assert_consistent(&a, &b, &c);
        let (ra, rb, rc) = (a.as_ref(), b.as_ref(), c.as_ref());
        assert_consistent(&ra, &rb, &rc);
        prop_assert_eq!(a.cmp(&b), ra.cmp(&rb));
    }

    #[test]
    fn quad_order_is_total_and_consistent(
        a in strategy::quad(), b in strategy::quad(), c in strategy::quad()
    ) {
        assert_consistent(&a, &b, &c);
        assert_consistent(a.triple(), b.triple(), c.triple());
        assert_consistent(a.graph(), b.graph(), c.graph());
        assert_consistent(a.triple().subject(), b.triple().subject(), c.triple().subject());
        let (ra, rb, rc) = (a.as_ref(), b.as_ref(), c.as_ref());
        assert_consistent(&ra, &rb, &rc);
        prop_assert_eq!(a.cmp(&b), ra.cmp(&rb));
    }

    #[test]
    fn sorting_is_independent_of_input_order(
        terms in prop::collection::vec(strategy::term(), 0..24),
        quads in prop::collection::vec(strategy::quad(), 0..24),
    ) {
        assert_sorting_is_canonical(terms);
        assert_sorting_is_canonical(quads);
    }
}

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}
fn tag(value: &str) -> LanguageTag {
    LanguageTag::try_new(value.to_owned()).unwrap()
}

#[test]
fn documented_order_of_kinds_and_components() {
    let blank = |scope: &str, label: &str| {
        Term::BlankNode(BlankNodeId::try_new(iri(scope), label.to_owned()).unwrap())
    };
    let nested = Term::triple(Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Iri(iri("urn:o")),
    ));
    // bounded: one representative per documented ordering step.
    let ascending: [Term; 10] = [
        Term::Iri(iri("urn:a")),
        Term::Iri(iri("urn:b")),
        blank("urn:a", "z"),
        blank("urn:b", "a"),
        Term::Literal(Literal::typed("a", iri("urn:z")).unwrap()),
        Term::Literal(Literal::typed("b", iri("urn:a")).unwrap()),
        Term::Literal(Literal::language_tagged("c", tag("EN"))),
        Term::Literal(Literal::language_tagged("c", tag("fr"))),
        Term::Literal(Literal::directional("d", tag("en"), BaseDirection::Ltr)),
        nested,
    ];
    assert!(
        ascending.windows(2).all(|pair| pair[0] < pair[1]),
        "{ascending:?}"
    );
    let ltr = Literal::directional("c", tag("en"), BaseDirection::Ltr);
    let rtl = Literal::directional("c", tag("EN"), BaseDirection::Rtl);
    assert!(ltr < rtl);
    assert_eq!(
        Literal::language_tagged("c", tag("EN-us"))
            .cmp(&Literal::language_tagged("c", tag("en-US"))),
        Ordering::Equal
    );
    let triple: Triple = Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        Term::Iri(iri("urn:o")),
    );
    assert!(
        Quad::new(triple.clone(), GraphName::Default)
            < Quad::new(triple, GraphName::Iri(iri("urn:g")))
    );
}
