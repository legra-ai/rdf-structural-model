//! Hostile deeply nested triple terms are rejected with a typed error instead
//! of exhausting the decoder's stack.

use iri_rfc3987::Iri;
use rdf_structural_model::{
    GeneralizedTerm,
    GeneralizedTriple,
    MAX_TRIPLE_TERM_DEPTH,
    ModelError,
    NestingDepth,
    Subject,
    Term,
    Triple,
};

const HOSTILE_DEPTH: usize = 10_000;

fn iri(value: &str) -> Iri<String> {
    Iri::parse(value).unwrap().into()
}

/// One encoded nesting level split around its innermost object, so payloads of
/// any depth are built without constructing (or recursively dropping) values.
struct Layer {
    prefix: Vec<u8>,
    leaf: Vec<u8>,
    suffix: Vec<u8>,
}

impl Layer {
    fn split(level: &[u8], leaf: Vec<u8>) -> Self {
        let start = level
            .windows(leaf.len())
            .position(|window| window == leaf.as_slice())
            .unwrap();
        Self {
            prefix: level[..start].to_vec(),
            suffix: level[start + leaf.len()..].to_vec(),
            leaf,
        }
    }
    fn nest(&self, depth: usize) -> Vec<u8> {
        let mut bytes = self.prefix.repeat(depth);
        bytes.extend_from_slice(&self.leaf);
        bytes.extend(self.suffix.repeat(depth));
        bytes
    }
}

fn standard_level() -> (Term, Term) {
    let leaf = Term::iri(iri("urn:leaf"));
    let level = Term::triple(Triple::new(
        Subject::Iri(iri("urn:s")),
        iri("urn:p"),
        leaf.clone(),
    ));
    (level, leaf)
}

fn generalized_level() -> (GeneralizedTerm, GeneralizedTerm) {
    let leaf = GeneralizedTerm::Iri(iri("urn:leaf"));
    let level = GeneralizedTerm::Triple(Box::new(GeneralizedTriple::new(
        GeneralizedTerm::Iri(iri("urn:s")),
        GeneralizedTerm::Iri(iri("urn:p")),
        leaf.clone(),
    )));
    (level, leaf)
}

fn json_layer<T: serde::Serialize>((level, leaf): (T, T)) -> Layer {
    Layer::split(
        &serde_json::to_vec(&level).unwrap(),
        serde_json::to_vec(&leaf).unwrap(),
    )
}

fn postcard_layer<T: serde::Serialize>((level, leaf): (T, T)) -> Layer {
    Layer::split(
        &postcard::to_allocvec(&level).unwrap(),
        postcard::to_allocvec(&leaf).unwrap(),
    )
}

fn assert_too_deep_json(message: &str) {
    assert!(message.contains("nesting depth"), "{message}");
}

#[test]
fn hostile_json_triple_terms_are_rejected_with_a_typed_error() {
    let payload = json_layer(standard_level()).nest(HOSTILE_DEPTH);
    let error = serde_json::from_slice::<Term>(&payload).unwrap_err();
    assert_too_deep_json(&error.to_string());
}

#[test]
fn hostile_postcard_triple_terms_are_rejected() {
    let payload = postcard_layer(standard_level()).nest(HOSTILE_DEPTH);
    assert!(postcard::from_bytes::<Term>(&payload).is_err());
}

#[test]
fn hostile_json_generalized_terms_are_rejected_with_a_typed_error() {
    let payload = json_layer(generalized_level()).nest(HOSTILE_DEPTH);
    let error = serde_json::from_slice::<GeneralizedTerm>(&payload).unwrap_err();
    assert_too_deep_json(&error.to_string());
}

#[test]
fn hostile_postcard_generalized_terms_are_rejected() {
    let payload = postcard_layer(generalized_level()).nest(HOSTILE_DEPTH);
    assert!(postcard::from_bytes::<GeneralizedTerm>(&payload).is_err());
}

fn limit() -> usize {
    MAX_TRIPLE_TERM_DEPTH.get()
}

fn too_deep() -> ModelError {
    ModelError::NestingTooDeep {
        limit: MAX_TRIPLE_TERM_DEPTH,
        depth: NestingDepth::new(limit() + 1),
    }
}

#[test]
fn the_limit_itself_decodes_and_one_more_level_is_rejected_in_json() {
    let standard = json_layer(standard_level());
    let accepted: Term = serde_json::from_slice(&standard.nest(limit())).unwrap();
    assert_eq!(
        serde_json::to_vec(&accepted).unwrap(),
        standard.nest(limit())
    );
    let error = serde_json::from_slice::<Term>(&standard.nest(limit() + 1)).unwrap_err();
    assert!(
        error.to_string().starts_with(&too_deep().to_string()),
        "{error}"
    );

    let generalized = json_layer(generalized_level());
    let accepted: GeneralizedTerm = serde_json::from_slice(&generalized.nest(limit())).unwrap();
    assert_eq!(
        serde_json::to_vec(&accepted).unwrap(),
        generalized.nest(limit())
    );
    assert!(serde_json::from_slice::<GeneralizedTerm>(&generalized.nest(limit() + 1)).is_err());
}

#[test]
fn the_limit_itself_decodes_and_one_more_level_is_rejected_in_postcard() {
    let standard = postcard_layer(standard_level());
    let accepted: Term = postcard::from_bytes(&standard.nest(limit())).unwrap();
    assert_eq!(
        postcard::to_allocvec(&accepted).unwrap(),
        standard.nest(limit())
    );
    assert!(postcard::from_bytes::<Term>(&standard.nest(limit() + 1)).is_err());

    let generalized = postcard_layer(generalized_level());
    let accepted: GeneralizedTerm = postcard::from_bytes(&generalized.nest(limit())).unwrap();
    assert_eq!(
        postcard::to_allocvec(&accepted).unwrap(),
        generalized.nest(limit())
    );
    assert!(postcard::from_bytes::<GeneralizedTerm>(&generalized.nest(limit() + 1)).is_err());
}

#[test]
fn borrowed_terms_at_the_limit_decode_from_the_input_buffer() {
    let payload = json_layer(standard_level()).nest(limit());
    let borrowed: Term<&str> = serde_json::from_slice(&payload).unwrap();
    assert_eq!(serde_json::to_vec(&borrowed).unwrap(), payload);
}

fn generalized_nested(depth: usize) -> GeneralizedTerm {
    (0..depth).fold(GeneralizedTerm::Iri(iri("urn:leaf")), |object, _| {
        GeneralizedTerm::Triple(Box::new(GeneralizedTriple::new(
            GeneralizedTerm::Iri(iri("urn:s")),
            GeneralizedTerm::Iri(iri("urn:p")),
            object,
        )))
    })
}

#[test]
fn generalized_to_standard_conversion_applies_the_same_limit() {
    assert!(Term::try_from(generalized_nested(limit())).is_ok());
    assert_eq!(
        Term::try_from(generalized_nested(limit() + 1)),
        Err(too_deep())
    );
    let GeneralizedTerm::Triple(triple) = generalized_nested(limit() + 2) else {
        unreachable!("nested terms are triple terms");
    };
    assert_eq!(Triple::try_from(*triple), Err(too_deep()));
}
