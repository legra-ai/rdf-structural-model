//! Metadata-only readers share standard literal invariants with constructors.

use iri_rfc3987::Iri;
use rdf_structural_model::{
    BaseDirection,
    LanguageTag,
    Literal,
    LiteralFacets,
    LiteralMetadata,
    ModelError,
    RDF_DIR_LANG_STRING,
    RDF_LANG_STRING,
    Subject,
    Term,
    XSD_STRING,
};
use serde::{
    Serialize,
    Serializer,
};

fn iri(text: &str) -> Iri<String> {
    Iri::parse(text).unwrap().into()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MetadataEnvelope {
    metadata: LiteralMetadata,
}

#[derive(Serialize)]
struct LiteralInput {
    lexical: String,
    metadata: LiteralMetadata,
}

struct LiteralTermInput(LiteralInput);
impl Serialize for LiteralTermInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_variant("Term", 2, "Literal", &self.0)
    }
}

#[derive(Serialize)]
struct TripleInput {
    subject: Subject,
    predicate: Iri<String>,
    object: LiteralTermInput,
}

struct TripleTermInput(TripleInput);
impl Serialize for TripleTermInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_variant("Term", 3, "Triple", &self.0)
    }
}

#[test]
fn metadata_validation_matches_construction_and_nested_decoding_without_reading_payload() {
    // bounded: four datatype forms by four language/direction facet forms.
    for datatype in [
        XSD_STRING,
        "urn:unrecognized-datatype",
        RDF_LANG_STRING,
        RDF_DIR_LANG_STRING,
    ] {
        for facets in [
            LiteralFacets::None,
            LiteralFacets::Language(LanguageTag::try_new("EN-us".to_owned()).unwrap()),
            LiteralFacets::Directional {
                language: LanguageTag::try_new("EN-us".to_owned()).unwrap(),
                direction: BaseDirection::Ltr,
            },
            LiteralFacets::Directional {
                language: LanguageTag::try_new("EN-us".to_owned()).unwrap(),
                direction: BaseDirection::Rtl,
            },
        ] {
            let valid = match (&facets, datatype) {
                (LiteralFacets::None, RDF_LANG_STRING | RDF_DIR_LANG_STRING) => false,
                (LiteralFacets::None, _)
                | (LiteralFacets::Language(_), RDF_LANG_STRING)
                | (LiteralFacets::Directional { .. }, RDF_DIR_LANG_STRING) => true,
                _ => false,
            };
            let expected = if valid {
                Ok(())
            } else {
                Err(ModelError::InconsistentLiteralFacets)
            };
            let metadata = LiteralMetadata::new(iri(datatype), facets);
            assert_eq!(metadata.validate_standard(), expected);
            assert_eq!(metadata.as_ref().validate_standard(), expected);
            // NUL must not become a lexical-value policy in the structural
            // check.
            assert_eq!(
                Literal::try_new("\0".to_owned(), metadata.clone()).map(|_| ()),
                expected
            );

            // Extension metadata remains constructible and serializable even
            // when the explicit standard-profile check rejects its pairing.
            let envelope = MetadataEnvelope {
                metadata: metadata.clone(),
            };
            let json = serde_json::to_string(&envelope).unwrap();
            let restored: MetadataEnvelope = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.metadata, metadata);
            assert_eq!(restored.metadata.validate_standard(), expected);
            let bytes = postcard::to_allocvec(&envelope).unwrap();
            let restored: MetadataEnvelope = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(restored.metadata, metadata);
            assert_eq!(restored.metadata.validate_standard(), expected);

            let nested = TripleTermInput(TripleInput {
                subject: Subject::Iri(iri("urn:subject")),
                predicate: iri("urn:predicate"),
                object: LiteralTermInput(LiteralInput {
                    lexical: "\0".to_owned(),
                    metadata,
                }),
            });
            let json = serde_json::to_string(&nested).unwrap();
            let decoded = serde_json::from_str::<Term>(&json);
            assert_eq!(decoded.is_ok(), valid);
            if !valid {
                assert!(
                    decoded
                        .unwrap_err()
                        .to_string()
                        .contains(&ModelError::InconsistentLiteralFacets.to_string())
                );
            }
            let bytes = postcard::to_allocvec(&nested).unwrap();
            assert_eq!(postcard::from_bytes::<Term>(&bytes).is_ok(), valid);
        }
    }
}
