# rdf-structural-model

Owned and borrowed RDF 1.2 semantic values with typed extension payloads.

This crate separates term identity from datatype value evaluation and storage.
It preserves exact literal lexical forms and scoped blank-node identity.
Semantic values use Serde directly, including non-self-describing binary formats.

The baseline is [RDF 1.2 Concepts, 7 April 2026](https://www.w3.org/TR/2026/CR-rdf12-concepts-20260407/).
It is a structural model, not a parser, graph store, reasoner or conformance claim.

## Values and identity

- `Term`, `Subject`, `Triple`, `GraphName` and `Quad` represent individual values.
  Standard subjects are IRIs or blank nodes; triple terms occur in objects.
- `BlankNodeId` combines an application-assigned scope IRI with an abstract local
  identifier. Independent imports need distinct scopes. The caller decides when
  sources intentionally share a blank-node domain; this crate has no global IDs.
- `Literal` preserves the lexical form, datatype and language/direction metadata.
  `"005"^^xsd:integer` and `"5"^^xsd:integer` are different terms. Datatype value
  comparison and ill-typed lexical-form detection belong in semantic evaluators.
- `LanguageTag` checks BCP 47 syntax, including duplicate variants and extension
  singletons. Equality and hashing ignore ASCII case; original spelling survives
  serialization. Tags need not be registered in the IANA database.
- `LiteralMetadata` and `LiteralFacets` carry no payload or storage address.
  Metadata can describe extended literals whose language is independent of the
  datatype; constructing a standard `Literal` enforces the RDF combinations.

```rust
use iri_rfc3987::Iri;
use rdf_structural_model::{BlankNodeId, GraphName, Literal, Quad, Subject, Term, Triple};

let scope = Iri::parse("urn:import:document-a").unwrap().into();
let subject = BlankNodeId::try_new(scope, "b0".to_owned()).unwrap();
let predicate = Iri::parse("https://example.org/name").unwrap().into();
let triple: Triple = Triple::new(
    Subject::BlankNode(subject), predicate, Term::Literal(Literal::string("Alice")),
);
let quad = Quad::new(triple, GraphName::Default);
let borrowed = quad.as_ref();
assert!(matches!(borrowed.graph(), GraphName::Default));
```

## Ownership and serialization

The text parameter defaults to `String`; `&str` provides borrowed values with
identical semantic and Serde shapes. `as_ref()` borrows text and extension
payloads. Borrowing nested triple terms allocates structural boxes, never copies
lexical payloads. Borrowed deserialization requires a serializer that lends
strings from its input; escaped JSON strings may require an owned value instead.

All semantic types implement Serde directly. Checked deserialization rejects
invalid tags, empty blank-node identities, inconsistent standard literal facets,
unknown fields and unsupported extension payloads. Tests exercise both JSON and
Postcard, including borrowed decoding into input buffers. Neither encoding is
an RDF concrete syntax, a canonical hash preimage or a promised stable protocol.

```rust
use rdf_structural_model::Literal;

let owned = Literal::string("exact lexical form");
let encoded = serde_json::to_string(&owned).unwrap();
let borrowed: Literal<&str> = serde_json::from_str(&encoded).unwrap();
assert_eq!(borrowed, owned.as_ref());
```

## Explicit extensions

The extension parameter defaults to the uninhabited `NoExtension` type. A
standard-profile decoder rejects additional term payloads. Applications can opt
into a typed extension enum for binary literals, referenced payloads or additional
subject/graph forms. There is no JSON value or string fallback. Applications
remain responsible for extension semantics and backend capability validation.
Standard predicates remain IRIs. Generalized values use the separate types below;
this crate provides no reasoning or storage engine.

Extended metadata is separate from standard lexical literal validation. This
allows a backend to expose metadata without loading bytes, or to carry a typed
binary payload without pretending it is an RDF string. Private storage IDs,
encryption and resolution capabilities belong in caller-owned types and adapters.

## Explicit generalized statements

`GeneralizedTerm`, `GeneralizedTriple`, `GeneralizedGraphName` and
`GeneralizedQuad` model the explicit, non-standard generalization in
[RDF 1.2 Concepts §7.2](https://www.w3.org/TR/2026/CR-rdf12-concepts-20260407/#section-generalized-rdf).
Every triple position and named graph position can hold an IRI, scoped blank
node, exact literal or recursively generalized triple term. The default graph
is a separate variant. Typed application extensions remain explicit.

Use these values for generalized reasoning intermediates, including literal
subjects, without representing literals as invented IRIs or private extensions.
They reuse the same checked literal, IRI and blank-node types. JSON and Postcard
support owned and borrowed decoding; `as_ref()` borrows lexical and extension
payloads and allocates only nested structural boxes.

`From` converts standard values into the generalized domain without cloning.
`TryFrom` converts back and rejects invalid standard subject, predicate and graph
positions, including invalid nested triple terms. Conversion never drops a
statement or normalizes its lexical forms. Existing `Term`, `Triple`,
`GraphName` and `Quad` keep their standard RDF position contracts.

```rust
use iri_rfc3987::Iri;
use rdf_structural_model::{GeneralizedTerm, GeneralizedTriple, Literal, Triple};

let statement: GeneralizedTriple = GeneralizedTriple::new(
    GeneralizedTerm::Literal(Literal::string("literal subject")),
    GeneralizedTerm::Iri(Iri::parse("urn:predicate").unwrap().into()),
    GeneralizedTerm::Literal(Literal::string("object")),
);
assert!(Triple::try_from(statement).is_err());
```

## Memory and maintenance

There is no graph/dataset container, I/O, payload resolver or global mutable state.
Owned values represent a caller-bounded structural item, not a promise that a
whole graph or large literal fits in memory. Use borrowed metadata and streamed
payload access for large values; input adapters must enforce their own budgets.

Rust 1.99.1 is the minimum compiler. `scripts/fmt.sh` uses the separately pinned
nightly formatter. Run `cargo test --locked` for the model contracts; local commit
hooks run pinned formatting and Clippy. Main CI is the sole release publisher
and uses the shared `legra-ai/github-actions` tested-revision pipeline.

Licensed under MIT OR Apache-2.0.
