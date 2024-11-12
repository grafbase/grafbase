use std::collections::BTreeMap;

mod apollo;

pub struct TrustedDocument {
    pub document_id: String,
    pub document_text: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum TrustedDocumentsManifest {
    Apollo(apollo::ApolloOperationManifest),
    Relay(RelayTrustedDocumentsManifest),
}

impl TrustedDocumentsManifest {
    pub fn into_documents(self) -> Box<dyn Iterator<Item = TrustedDocument>> {
        match self {
            TrustedDocumentsManifest::Apollo(manifest) => Box::new(manifest.operations.into_iter().map(
                |apollo::ApolloOperation {
                     id,
                     body,
                     name: _,
                     r#type: _,
                 }| TrustedDocument {
                    document_id: id,
                    document_text: body,
                },
            )),
            TrustedDocumentsManifest::Relay(map) => Box::new(map.into_iter().map(|(key, value)| TrustedDocument {
                document_id: key,
                document_text: value,
            })),
        }
    }
}

pub type RelayTrustedDocumentsManifest = BTreeMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apollo_basic() {
        // document from the docs
        let manifest = r#"
{
  "format": "apollo-persisted-query-manifest",
  "version": 1,
  "operations": [
    {
      "id": "dc67510fb4289672bea757e862d6b00e83db5d3cbbcfb15260601b6f29bb2b8f",
      "body": "query UniversalQuery { __typename }",
      "name": "UniversalQuery",
      "type": "query"
    }
  ]
}            
        "#;

        let deserialized: TrustedDocumentsManifest = serde_json::from_str(manifest).unwrap();

        let expected = expect_test::expect![[r#"
            Apollo(
                ApolloOperationManifest {
                    format: "apollo-persisted-query-manifest",
                    version: 1,
                    operations: [
                        ApolloOperation {
                            id: "dc67510fb4289672bea757e862d6b00e83db5d3cbbcfb15260601b6f29bb2b8f",
                            body: "query UniversalQuery { __typename }",
                            name: "UniversalQuery",
                            type: "query",
                        },
                    ],
                },
            )
        "#]];

        expected.assert_debug_eq(&deserialized)
    }

    #[test]
    fn relay_basic() {
        let manifest = r#"
            {
                "this-is-the-hash": "this-is-the-query",
                "id-number-2": "query-number-2"
            }
        "#;

        let deserialized: TrustedDocumentsManifest = serde_json::from_str(manifest).unwrap();

        let expected = expect_test::expect![[r#"
            Relay(
                {
                    "id-number-2": "query-number-2",
                    "this-is-the-hash": "this-is-the-query",
                },
            )
        "#]];

        expected.assert_debug_eq(&deserialized);
    }

    #[test]
    fn relay_empty() {
        let empty_manifest = r#"{}"#;
        let deserialized: TrustedDocumentsManifest = serde_json::from_str(empty_manifest).unwrap();

        let expected = expect_test::expect![[r#"
            Relay(
                {},
            )
        "#]];

        expected.assert_debug_eq(&deserialized);
    }
}
