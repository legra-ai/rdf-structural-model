# rdf-structural-model

Owned and borrowed RDF 1.2 semantic values with typed extension payloads.

This crate separates term identity from datatype value evaluation and storage.
It preserves exact literal lexical forms and scoped blank-node identity.
Semantic values use Serde directly, including non-self-describing binary formats.

The baseline is [RDF 1.2 Concepts, 7 April 2026](https://www.w3.org/TR/2026/CR-rdf12-concepts-20260407/).
It is a structural model, not a parser, graph store, reasoner or conformance claim.
