//! Owned IRI allocation accounting through the public literal model.

use iri_rfc3987::Iri;
use rdf_structural_model::Literal;

#[test]
fn literal_datatype_exposes_its_retained_iri_capacity() {
    let mut text = String::with_capacity(4096);
    text.push_str("urn:datatype:large-allocation");
    let retained_capacity = text.capacity();
    let datatype = Iri::parse_owned(text).unwrap();

    let literal = Literal::typed("exact lexical form", datatype).unwrap();
    let datatype: &Iri<String> = literal.metadata().datatype();

    assert_eq!(datatype.capacity(), retained_capacity);
    assert!(datatype.capacity() > datatype.as_str().len());
    assert_eq!(literal.lexical(), "exact lexical form");
}
