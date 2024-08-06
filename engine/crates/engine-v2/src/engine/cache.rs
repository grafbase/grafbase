use std::borrow::Cow;

use base64::{display::Base64Display, engine::general_purpose::URL_SAFE_NO_PAD};
use engine::PersistedQueryRequestExtension;
use schema::Schema;

use super::SchemaVersion;

mod namespaces {
    pub const OPERATION: &str = "op";
}

/// Unique cache key that generates a URL-safe string.
/// Stable across engine version.
pub(super) enum Key<'a> {
    Operation {
        name: Option<&'a str>,
        schema_version: &'a SchemaVersion,
        document: Document<'a>,
    },
}

pub(super) enum Document<'a> {
    AutomaticallyPersistedQuery(&'a PersistedQueryRequestExtension),
    TrustedDocumentId { client_name: &'a str, doc_id: Cow<'a, str> },
    Text(&'a str),
}

impl std::fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Schema version + Commit SHA ensures we don't need to care about
            // backwards-compatibility
            Key::Operation {
                name,
                schema_version,
                document,
            } => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&Schema::build_identifier().len().to_ne_bytes());
                hasher.update(Schema::build_identifier());
                hasher.update(&schema_version.len().to_ne_bytes());
                hasher.update(schema_version);

                if let Some(name) = name {
                    hasher.update(name.as_bytes());
                }
                // NULL bytes acting as a separator as it cannot be present in the
                // operation name.
                hasher.update(&[0x00]);
                match document {
                    Document::AutomaticallyPersistedQuery(ext) => {
                        hasher.update(b"apq");
                        hasher.update(&[0x00]);
                        hasher.update(&ext.version.to_ne_bytes());
                        hasher.update(&ext.sha256_hash);
                    }
                    Document::TrustedDocumentId { client_name, doc_id } => {
                        hasher.update(b"docid");
                        hasher.update(&[0x00]);
                        hasher.update(&client_name.len().to_ne_bytes());
                        hasher.update(client_name.as_bytes());
                        hasher.update(&doc_id.len().to_ne_bytes());
                        hasher.update(doc_id.as_bytes());
                    }
                    Document::Text(document) => {
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
