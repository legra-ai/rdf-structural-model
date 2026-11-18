//! Proptest strategies over standard terms and quads.
//!
//! Small alphabets make equal and near-equal values collide often, and tags
//! vary in ASCII case so case-insensitive language equality is exercised.

use iri_rfc3987::Iri;
use proptest::prelude::*;
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

/// Nesting levels produced by the recursive term strategy itself.
const GENERATED_DEPTH: u32 = 4;

fn iri() -> impl Strategy<Value = Iri<String>> {
    prop::sample::select(&["urn:a", "urn:b", "urn:A", "https://example.org/é"][..])
        .prop_map(|text| Iri::parse(text).unwrap().into())
}

fn blank_node() -> impl Strategy<Value = BlankNodeId> {
    (iri(), prop::sample::select(&["a", "b", "B"][..]))
        .prop_map(|(scope, label)| BlankNodeId::try_new(scope, label.to_owned()).unwrap())
}

fn language_tag() -> impl Strategy<Value = LanguageTag> {
    (
        prop::sample::select(&["en", "en-US", "fr", "x-priv", "zh-Hant"][..]),
        prop::collection::vec(any::<bool>(), 8),
    )
        .prop_map(|(text, upper)| {
            let cased: String = text
                .chars()
                .zip(upper.iter().cycle())
                .map(|(c, up)| {
                    if *up {
                        c.to_ascii_uppercase()
                    } else {
                        c.to_ascii_lowercase()
                    }
                })
                .collect();
            LanguageTag::try_new(cased).unwrap()
        })
}

fn literal() -> impl Strategy<Value = Literal> {
    let lexical = prop::sample::select(&["", "5", "005", "text", "مرحبا"][..]);
    let direction = prop_oneof![Just(BaseDirection::Ltr), Just(BaseDirection::Rtl)];
    prop_oneof![
        (lexical.clone(), iri()).prop_map(|(lexical, datatype)| {
            Literal::try_new(
                lexical.to_owned(),
                LiteralMetadata::new(datatype, LiteralFacets::None),
            )
            .unwrap()
        }),
        lexical.clone().prop_map(Literal::string),
        (lexical.clone(), language_tag())
            .prop_map(|(lexical, tag)| Literal::language_tagged(lexical, tag)),
        (lexical, language_tag(), direction)
            .prop_map(|(lexical, tag, direction)| Literal::directional(lexical, tag, direction)),
    ]
}

/// A standard subject.
pub fn subject() -> impl Strategy<Value = Subject> {
    prop_oneof![
        iri().prop_map(Subject::Iri),
        blank_node().prop_map(Subject::BlankNode)
    ]
}

/// An arbitrary term with up to [`GENERATED_DEPTH`] nested triple terms.
pub fn term() -> impl Strategy<Value = Term> {
    let leaf = prop_oneof![
        iri().prop_map(Term::Iri),
        blank_node().prop_map(Term::BlankNode),
        literal().prop_map(Term::Literal),
    ];
    leaf.prop_recursive(GENERATED_DEPTH, 16, 1, |inner| {
        (subject(), iri(), inner).prop_map(|(subject, predicate, object)| {
            Term::triple(Triple::new(subject, predicate, object))
        })
    })
}

/// An arbitrary quad.
pub fn quad() -> impl Strategy<Value = Quad> {
    let graph = prop_oneof![
        Just(GraphName::Default),
        iri().prop_map(GraphName::Iri),
        blank_node().prop_map(GraphName::BlankNode),
    ];
    (triple(), graph).prop_map(|(triple, graph)| Quad::new(triple, graph))
}

/// An arbitrary standard triple, possibly with nested triple-term objects.
pub fn triple() -> impl Strategy<Value = Triple> {
    (subject(), iri(), term())
        .prop_map(|(subject, predicate, object)| Triple::new(subject, predicate, object))
}
