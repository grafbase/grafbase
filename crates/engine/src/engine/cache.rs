use std::borrow::Cow;

use base64::{display::Base64Display, engine::general_purpose::URL_SAFE_NO_PAD};
use operation::extensions::PersistedQueryRequestExtension;
use schema::Schema;

mod namespaces {
    pub const OPERATION: &str = "op";
}

/// Unique cache key that generates a URL-safe string.
pub(crate) enum CacheKey<'a> {
    Operation {
        schema: &'a Schema,
        document: &'a DocumentKey<'a>,
    },
}

impl CacheKey<'_> {
    pub(crate) fn document(schema: &Schema, document: &DocumentKey<'_>) -> String {
        CacheKey::Operation { schema, document }.to_string()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum DocumentKey<'a> {
    AutomaticPersistedQuery {
        operation_name: Option<Cow<'a, str>>,
        ext: Cow<'a, PersistedQueryRequestExtension>,
    },
    TrustedDocumentId {
        operation_name: Option<Cow<'a, str>>,
        client_name: Cow<'a, str>,
        doc_id: Cow<'a, str>,
    },
    Text {
        operation_name: Option<Cow<'a, str>>,
        document: Cow<'a, str>,
    },
}

impl DocumentKey<'_> {
    pub fn into_owned(self) -> DocumentKey<'static> {
        match self {
            DocumentKey::AutomaticPersistedQuery { operation_name, ext } => DocumentKey::AutomaticPersistedQuery {
                operation_name: operation_name.map(|name| Cow::Owned(name.into_owned())),
                ext: Cow::Owned(ext.into_owned()),
            },
            DocumentKey::TrustedDocumentId {
                operation_name,
                client_name,
                doc_id,
            } => DocumentKey::TrustedDocumentId {
                operation_name: operation_name.map(|name| Cow::Owned(name.into_owned())),
                client_name: Cow::Owned(client_name.into_owned()),
                doc_id: Cow::Owned(doc_id.into_owned()),
            },
            DocumentKey::Text {
                operation_name,
                document,
            } => DocumentKey::Text {
                operation_name: operation_name.map(|name| Cow::Owned(name.into_owned())),
                document: Cow::Owned(document.into_owned()),
            },
        }
    }
}

impl std::fmt::Display for CacheKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Schema version + Commit SHA ensures we don't need to care about
            // backwards-compatibility
            CacheKey::Operation { schema, document } => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&Schema::build_identifier().len().to_ne_bytes());
                hasher.update(Schema::build_identifier());
                hasher.update(&schema.version.len().to_ne_bytes());
                hasher.update(&schema.version);

                match document {
                    DocumentKey::AutomaticPersistedQuery { operation_name, ext } => {
                        hasher.update(b"apq");
                        hasher.update(&[0x00]);
                        if let Some(name) = operation_name {
                            hasher.update(name.as_bytes());
                        }
                        // NULL bytes acting as a separator as it cannot be present in the
                        // operation name.
                        hasher.update(&[0x00]);
                        hasher.update(&ext.version.to_ne_bytes());
                        hasher.update(&ext.sha256_hash);
                    }
                    DocumentKey::TrustedDocumentId {
                        operation_name,
                        client_name,
                        doc_id,
                    } => {
                        hasher.update(b"docid");
                        hasher.update(&[0x00]);
                        if let Some(name) = operation_name {
                            hasher.update(name.as_bytes());
                        }
                        hasher.update(&[0x00]);
                        hasher.update(&client_name.len().to_ne_bytes());
                        hasher.update(client_name.as_bytes());
                        hasher.update(&doc_id.len().to_ne_bytes());
                        hasher.update(doc_id.as_bytes());
                    }
                    DocumentKey::Text {
                        operation_name,
                        document,
                    } => {
                        hasher.update(b"doc");
                        hasher.update(&[0x00]);
                        if let Some(name) = operation_name {
                            hasher.update(name.as_bytes());
                        }
                        hasher.update(&[0x00]);
                        hasher.update(document.as_bytes());
                    }
                }
                let hash = hasher.finalize();

                f.write_fmt(format_args!(
                    "{}.blake3.{}",
                    namespaces::OPERATION,
                    Base64Display::new(hash.as_bytes(), &URL_SAFE_NO_PAD)
                ))
            }
        }
    }
}
