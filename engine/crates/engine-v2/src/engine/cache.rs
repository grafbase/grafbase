use base64::{display::Base64Display, engine::general_purpose::URL_SAFE_NO_PAD};
use engine::PersistedQueryRequestExtension;
use schema::Schema;

use super::SchemaVersion;

mod namespaces {
    pub const OPERATION: &str = "op";
    pub const TRUSTED_DOCUMENT: &str = "tdoc";
    pub const APQ: &str = "apq";
}

/// Unique cache key that generates a URL-safe string.
/// Stable across engine version.
pub(super) enum Key<'a> {
    Operation {
        name: Option<&'a str>,
        schema_version: &'a SchemaVersion,
        document: Document<'a>,
    },
    TrustedDocument {
        client_name: &'a str,
        document_id: &'a str,
    },
    Apq {
        ext: &'a PersistedQueryRequestExtension,
    },
}

pub(super) enum Document<'a> {
    PersistedQueryExt(&'a PersistedQueryRequestExtension),
    Id(&'a str),
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
                    Document::PersistedQueryExt(ext) => {
                        hasher.update(b"apq");
                        hasher.update(&[0x00]);
                        hasher.update(&ext.version.to_ne_bytes());
                        hasher.update(&ext.sha256_hash);
                    }
                    Document::Id(doc_id) => {
                        hasher.update(b"docid");
                        hasher.update(&[0x00]);
                        hasher.update(doc_id.as_bytes());
                    }
                    Document::Text(query) => {
                        hasher.update(b"query");
                        hasher.update(&[0x00]);
                        hasher.update(query.as_bytes());
                    }
                }
                let hash = hasher.finalize();

                f.write_fmt(format_args!(
                    "{}.blake3.{}",
                    namespaces::OPERATION,
                    Base64Display::new(hash.as_bytes(), &URL_SAFE_NO_PAD)
                ))
            }
            Key::TrustedDocument {
                client_name,
                document_id,
            } => f.write_fmt(format_args!(
                "{}.{}.{}",
                namespaces::TRUSTED_DOCUMENT,
                Base64Display::new(client_name.as_bytes(), &URL_SAFE_NO_PAD),
                Base64Display::new(document_id.as_bytes(), &URL_SAFE_NO_PAD)
            )),
            Key::Apq {
                ext: PersistedQueryRequestExtension { version, sha256_hash },
            } => f.write_fmt(format_args!(
                "{}.{}.{}",
                namespaces::APQ,
                version,
                Base64Display::new(sha256_hash, &URL_SAFE_NO_PAD)
            )),
        }
    }
}
