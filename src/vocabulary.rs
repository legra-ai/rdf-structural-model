//! RDF 1.2 vocabulary IRIs the structural model names directly.

/// The RDF namespace IRI ([RDF 1.2 Concepts §1.4](https://www.w3.org/TR/2026/CRD-rdf12-concepts-20261116/#vocabularies)).
pub const RDF_NAMESPACE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
/// The RDF language-string datatype IRI (`rdf:langString`, §3.3).
pub const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
/// The RDF directional language-string datatype IRI (`rdf:dirLangString`,
/// §3.3).
pub const RDF_DIR_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";
/// The reification predicate IRI (`rdf:reifies`, §1.5) of a reifying triple.
pub const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
/// The XML Schema string datatype IRI.
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
