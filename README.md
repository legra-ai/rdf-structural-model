# rdf-structural-model

Owned and borrowed RDF 1.2 semantic values with typed extension payloads.

This crate separates term identity from datatype value evaluation and storage.
It preserves exact literal lexical forms and scoped blank-node identity.
Semantic values use Serde directly, including non-self-describing binary formats.

The baseline is [RDF 1.2 Concepts, Candidate Recommendation Draft 16 November 2026](https://www.w3.org/TR/2026/CRD-rdf12-concepts-20261116/),
the newest dated publication; it supersedes the CR Snapshot of 7 April 2026.
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
- `LanguageTag` checks RFC 5646 well-formedness, including duplicate variants and
  extension singletons; every subtag, private-use ones included, is 1-8 ASCII
  alphanumerics. Rejections are `ModelError::InvalidLanguageTag(LanguageTagError)`,
  carrying a `LanguageTagErrorReason` and the failing `SubtagIndex`. `primary()`,
  `script()`, `region()` and `variants()` return subtags in their original
  spelling; private-use and irregular grandfathered tags have no language parts.
  Equality, hashing and ordering ignore ASCII case; original spelling survives
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

## Order

`Term`, `Subject`, `Literal`, `Triple`, `GraphName` and `Quad`, owned and
borrowed alike, implement one total order that agrees with `Eq` and `Hash`:

1. Position kind first, in declaration order: terms order IRI, blank node,
   literal, triple term, extension; subjects IRI, blank node, extension; graph
   names default graph, IRI, blank node, extension.
2. IRIs by their exact text.
3. Blank nodes by scope IRI, then label.
4. Literals by lexical form, then datatype IRI, then facets: none before a
   language before a language with direction; languages compare ASCII
   case-insensitively, so `en-US` and `EN-us` are equal; then direction, `ltr`
   before `rtl`.
5. Triples by subject, predicate, then object; quads by their triple, then
   graph name.

The order is structural, for sorting and deduplication. It is not SPARQL
`ORDER BY` and never compares datatype values.

## Specification coverage

Every test carries a `/// Spec: RDF 1.2 Concepts §…` doc line naming the
sections it covers. Against the 16 November 2026 draft:

| Section | Coverage |
| ------- | -------- |
| §1.4 Vocabularies | `RDF_NAMESPACE`, `RDF_LANG_STRING`, `RDF_DIR_LANG_STRING`, `RDF_REIFIES` (plus `XSD_STRING`). |
| §1.5 Triple Terms and Reification | `Reification` recognizes and rebuilds reifying triples; it never asserts the reified triple. |
| §3.1 Triples | Position-typed `Triple`: subjects are IRIs or blank nodes, predicates IRIs, objects any term. |
| §3.3 IRIs | Exact IRI text from `iri-rfc3987`; no normalization in identity. |
| §3.4 Literals | Exact lexical forms; `rdf:langString`/`rdf:dirLangString` iff facets; well-formed, case-insensitive language tags; base direction `ltr`/`rtl`, lowercase only (a requirement new in this draft). |
| §3.5 Blank Nodes | Scoped `BlankNodeId`; no global identifiers. |
| §3.6 Triple Terms | Recursive triple terms in object position, depth-bounded by `MAX_TRIPLE_TERM_DEPTH`. |
| §3.7 Graph Comparison | Term equality only; the structural total order is a crate contract, not isomorphism. |
| §4.3 Quads | `Quad` and `GraphName`; no dataset container. |
| §7.1 Symmetric RDF | No dedicated type: symmetric triples (literal or triple-term subjects) are representable as generalized values and refused by standard `TryFrom`. |
| §7.2 Generalized RDF | `GeneralizedTerm`, `GeneralizedTriple`, `GeneralizedGraphName`, `GeneralizedQuad`. |

Not modeled: graphs and datasets as containers (§3, §4), version announcement
(§1.10, §2.1, Appendix C), datatype value spaces (§5, Appendix A), and fragment
identifiers (§6). The other changes since the April snapshot are editorial or
concern concrete syntaxes and content negotiation.

## Reification

`Reification { reifier, triple }` is the structural view of one reifying triple
`reifier rdf:reifies <<( triple )>>`. `TryFrom<Triple>` accepts only a triple
whose predicate is exactly `rdf:reifies` and whose object is a triple term,
failing with `ModelError::NotReifyingPredicate` or
`ModelError::ReifiedObjectNotTripleTerm`. `From<Reification> for Triple` rebuilds
the reifying triple, owned or borrowed. There is no annotation syntax, dataset or
assertion of the reified triple.

```rust
use iri_rfc3987::Iri;
use rdf_structural_model::{Literal, RDF_REIFIES, Reification, Subject, Term, Triple};

let reified: Triple = Triple::new(
    Subject::Iri(Iri::parse("urn:alice").unwrap().into()),
    Iri::parse("urn:name").unwrap().into(),
    Term::Literal(Literal::string("Alice")),
);
let reifier = Subject::Iri(Iri::parse("urn:claim").unwrap().into());
let reifying: Triple = Reification::new(reifier, reified.clone()).into();
assert_eq!(reifying.predicate().as_str(), RDF_REIFIES);
assert_eq!(Reification::try_from(reifying).unwrap().triple(), &reified);
```

## Ownership and serialization

The text parameter defaults to `String`; `&str` provides borrowed values with
identical semantic and Serde shapes. `as_ref()` borrows text and extension
payloads. Borrowing nested triple terms allocates structural boxes, never copies
lexical payloads. Borrowed deserialization requires a serializer that lends
strings from its input; escaped JSON strings may require an owned value instead.

All semantic types implement Serde directly. Checked deserialization rejects
invalid tags, empty blank-node identities, inconsistent standard literal facets,
unknown fields and unsupported extension payloads. Decoding `Term`, `Triple`,
`GeneralizedTerm` and `GeneralizedTriple` is depth-checked: triple terms nested
deeper than `MAX_TRIPLE_TERM_DEPTH` (32 enclosing triple terms) fail with
`ModelError::NestingTooDeep` instead of exhausting the stack, in every format,
including Postcard, which has no recursion limit of its own. Tests exercise both JSON and
Postcard, including borrowed decoding into input buffers, property round-trips of
arbitrary nested terms and exact golden vectors in `tests/fixtures/golden/`
(regenerated only with `UPDATE_GOLDEN=1`). Neither encoding is
an RDF concrete syntax, a canonical hash preimage or a promised stable protocol.

```rust
use rdf_structural_model::Literal;

let owned = Literal::string("exact lexical form");
let encoded = serde_json::to_string(&owned).unwrap();
let borrowed: Literal<&str> = serde_json::from_str(&encoded).unwrap();
assert_eq!(borrowed, owned.as_ref());
```

Metadata-only adapters can call `LiteralMetadata::validate_standard()` before
reading lexical payloads. This checks the same datatype/facet pairing as
`Literal::try_new`; it does not recognize datatypes or evaluate lexical values.
Extended metadata remains representable until a caller explicitly requests this
standard-profile check.

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
[RDF 1.2 Concepts §7.2](https://www.w3.org/TR/2026/CRD-rdf12-concepts-20261116/#section-generalized-rdf).
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
positions, including invalid nested triple terms, and applies the same
`MAX_TRIPLE_TERM_DEPTH` as decoding. Conversion never drops a
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
