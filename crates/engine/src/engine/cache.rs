use std::borrow::Cow;

use base64::{display::Base64Display, engine::general_purpose::URL_SAFE_NO_PAD};
use schema::Schema;

use crate::request::extensions::PersistedQueryRequestExtension;

pub use self::warming::OperationForWarming;

mod warming;

mod namespaces {
    pub const OPERATION: &str = "op";
}

/// Unique cache key that generates a URL-safe string.
pub(crate) enum Key<'a> {
    Operation {
        name: Option<&'a str>,
        schema: &'a Schema,
        document: DocumentKey<'a>,
    },
}

impl<'a> Key<'a> {
    pub fn document_key(&self) -> &'_ DocumentKey<'a> {
        match self {
            Key::Operation { document, .. } => document,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum DocumentKey<'a> {
    AutomaticPersistedQuery(Cow<'a, PersistedQueryRequestExtension>),
    TrustedDocumentId {
        client_name: Cow<'a, str>,
        doc_id: Cow<'a, str>,
    },
    Text(Cow<'a, str>),
}

impl DocumentKey<'_> {
    pub fn to_static(&self) -> DocumentKey<'static> {
        match self.clone() {
            DocumentKey::AutomaticPersistedQuery(inner) => {
                DocumentKey::AutomaticPersistedQuery(Cow::Owned(inner.into_owned()))
            }
            DocumentKey::TrustedDocumentId { client_name, doc_id } => DocumentKey::TrustedDocumentId {
                client_name: Cow::Owned(client_name.into_owned()),
                doc_id: Cow::Owned(doc_id.into_owned()),
            },
            DocumentKey::Text(inner) => DocumentKey::Text(Cow::Owned(inner.into_owned())),
        }
    }
}

impl std::fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Schema version + Commit SHA ensures we don't need to care about
            // backwards-compatibility
            Key::Operation { name, schema, document } => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&Schema::build_identifier().len().to_ne_bytes());
                hasher.update(Schema::build_identifier());
                hasher.update(&schema.version.len().to_ne_bytes());
                hasher.update(&schema.version);

                if let Some(name) = name {
                    hasher.update(name.as_bytes());
                }
                // NULL bytes acting as a separator as it cannot be present in the
                // operation name.
                hasher.update(&[0x00]);
                match document {
                    DocumentKey::AutomaticPersistedQuery(ext) => {
                        hasher.update(b"apq");
                        hasher.update(&[0x00]);
                        hasher.update(&ext.version.to_ne_bytes());
                        hasher.update(&ext.sha256_hash);
                    }
                    DocumentKey::TrustedDocumentId { client_name, doc_id } => {
                        hasher.update(b"docid");
                        hasher.update(&[0x00]);
                        hasher.update(&client_name.len().to_ne_bytes());
                        hasher.update(client_name.as_bytes());
                        hasher.update(&doc_id.len().to_ne_bytes());
                        hasher.update(doc_id.as_bytes());
                    }
                    DocumentKey::Text(document) => {
                        hasher.update(b"doc");
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
