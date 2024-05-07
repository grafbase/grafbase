//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use super::{Gateway, CLIENT_NAME_HEADER_NAME};
use engine::{AutomaticPersistedQuery, ErrorCode, PersistedQueryRequestExtension, ServerError};
use runtime::trusted_documents_client::TrustedDocumentsError;
use std::mem;
use tracing::instrument;

const CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

impl<Executor> Gateway<Executor>
where
    Executor: super::Executor + 'static,
    Executor::Error: From<super::Error> + std::error::Error + Send + 'static,
    Executor::StreamingResponse: super::ConstructableResponse<Error = Executor::Error>,
{
    /// Handle a request making use of APQ or trusted documents.
    pub(super) async fn handle_persisted_query(
        &self,
        request: &mut engine::Request,
        client_name: Option<&str>,
        headers: &http::HeaderMap,
    ) -> Result<(), PersistedQueryError> {
        let trusted_documents_enabled = self.trusted_documents.is_enabled();
        let persisted_query_extension = mem::take(&mut request.extensions.persisted_query);
        let doc_id = mem::take(&mut request.operation_plan_cache_key.document_id);

        match (trusted_documents_enabled, persisted_query_extension, doc_id) {
            (true, None, None) => {
                if self
                    .trusted_documents
                    .bypass_header()
                    .map(|(name, value)| headers.get(name).and_then(|header| header.to_str().ok()) == Some(value))
                    .unwrap_or_default()
                {
                    Ok(())
                } else {
                    Err(PersistedQueryError::BadRequest)
                }
            }
            (true, Some(ext), _) => {
                self.handle_apollo_client_style_trusted_document_query(request, ext, client_name)
                    .await
            }
            (true, _, Some(document_id)) => {
                self.handle_trusted_document_query(request, &document_id, client_name)
                    .await
            }
            (false, None, _) => Ok(()),
            (false, Some(ext), _) => self.handle_apq(request, &ext).await,
        }
    }

    async fn handle_apollo_client_style_trusted_document_query(
        &self,
        request: &mut engine::Request,
        ext: PersistedQueryRequestExtension,
        client_name: Option<&str>,
    ) -> Result<(), PersistedQueryError> {
        use std::fmt::Write;

        let document_id = {
            let mut id = String::with_capacity(ext.sha256_hash.len() * 2);

            for byte in &ext.sha256_hash {
                write!(id, "{byte:02x}").expect("write to String to succeed");
            }

            id
        };

        self.handle_trusted_document_query(request, &document_id, client_name)
            .await
    }

    #[instrument(skip_all)]
    async fn handle_trusted_document_query(
        &self,
        request: &mut engine::Request,
        document_id: &str,
        client_name: Option<&str>,
    ) -> Result<(), PersistedQueryError> {
        let Some(client_name) = client_name else {
            return Err(PersistedQueryError::MissingClientName);
        };

        let cache = &self.cache;
        let cache_key = cache.build_key(&format!("trusted_documents/{client_name}/{document_id}"));

        // First try fetching the document from cache.
        if let Some(document_text) = cache
            .get(&cache_key)
            .await
            .ok()
            .and_then(|entry| entry.into_value())
            .and_then(|bytes| String::from_utf8(bytes).ok())
        {
            request.operation_plan_cache_key.query = document_text;
            return Ok(());
        }

        match self.trusted_documents.fetch(client_name, document_id).await {
            Err(TrustedDocumentsError::RetrievalError(err)) => {
                tracing::error!("Trusted document retrieval error: {err}");
                Err(PersistedQueryError::InternalServerError)
            }
            Err(TrustedDocumentsError::DocumentNotFound) => Err(PersistedQueryError::UnknownDocumentId {
                document_id: document_id.to_owned(),
            }),
            Ok(document_text) => {
                cache
                    .put(
                        &cache_key,
                        runtime::cache::EntryState::Fresh,
                        document_text.clone().into_bytes(),
                        runtime::cache::CacheMetadata {
                            max_age: CACHE_MAX_AGE,
                            stale_while_revalidate: std::time::Duration::ZERO,
                            tags: Vec::new(),
                            should_purge_related: false,
                            should_cache: true,
                        },
                    )
                    .await
                    .map_err(|err| {
                        tracing::error!("Cache error: {}", err);
                        PersistedQueryError::InternalServerError
                    })?;

                request.operation_plan_cache_key.query = document_text;
                Ok(())
            }
        }
    }

    /// Handle a request using Automatic Persisted Queries.
    #[instrument(skip_all)]
    async fn handle_apq(
        &self,
        request: &mut engine::Request,
        PersistedQueryRequestExtension { version, sha256_hash }: &PersistedQueryRequestExtension,
    ) -> Result<(), PersistedQueryError> {
        if *version != 1 {
            return Err(PersistedQueryError::UnsupportedVersion);
        }

        let cache = &self.cache;
        let key = cache.build_key(&format!("apq/sha256_{}", hex::encode(sha256_hash)));
        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(PersistedQueryError::BadSha256);
            }
            cache
                .put_json(
                    &key,
                    runtime::cache::EntryState::Fresh,
                    &AutomaticPersistedQuery::V1 {
                        query: request.query().to_string(),
                    },
                    runtime::cache::CacheMetadata {
                        max_age: CACHE_MAX_AGE,
                        stale_while_revalidate: std::time::Duration::ZERO,
                        tags: Vec::new(),
                        should_purge_related: false,
                        should_cache: true,
                    },
                )
                .await
                .map_err(|err| {
                    tracing::error!("Cache error: {}", err);
                    PersistedQueryError::InternalServerError
                })?;
            return Ok(());
        }

        match cache.get_json::<AutomaticPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticPersistedQuery::V1 { query }) = entry.into_value() {
                    request.operation_plan_cache_key.query = query;
                    Ok(())
                } else {
                    Err(PersistedQueryError::NotFound)
                }
            }
            Err(err) => {
                tracing::error!("Cache error: {}", err);
                Err(PersistedQueryError::InternalServerError)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum PersistedQueryError {
    #[error("Persisted query not found")]
    NotFound,
    #[error("Persisted query version not supported")]
    UnsupportedVersion,
    #[error("Internal server error (trusted documents)")]
    InternalServerError,
    #[error("Cannot execute a trusted document query: missing documentId, doc_id or the persistedQuery extension.")]
    BadRequest,
    #[error("Trusted document queries must include the {CLIENT_NAME_HEADER_NAME} header")]
    MissingClientName,
    #[error("Invalid persisted query sha256Hash")]
    BadSha256,
    #[error("Unknown document id: '{document_id}'")]
    UnknownDocumentId { document_id: String },
}

impl From<PersistedQueryError> for ServerError {
    fn from(err: PersistedQueryError) -> Self {
        let message = err.to_string();
        let error = ServerError::new(message, None);
        if matches!(err, PersistedQueryError::NotFound) {
            ServerError {
                extensions: Some(engine::ErrorExtensionValues(
                    [(
                        "code".to_string(),
                        engine::Value::String(ErrorCode::PersistedQueryNotFound.to_string()),
                    )]
                    .into_iter()
                    .collect(),
                )),
                ..error
            }
        } else {
            error
        }
    }
}
