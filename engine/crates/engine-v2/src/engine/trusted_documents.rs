//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use super::{Engine, CLIENT_NAME_HEADER_NAME};
use crate::response::GraphqlError;
use engine::{AutomaticPersistedQuery, ErrorCode, PersistedQueryRequestExtension, RequestHeaders};
use runtime::trusted_documents_client::TrustedDocumentsError;
use std::mem;

const CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

impl Engine {
    /// Handle a request making use of APQ or trusted documents.
    pub(super) async fn handle_persisted_query(
        &self,
        request: &mut engine::Request,
        client_name: Option<&str>,
        headers: &RequestHeaders,
    ) -> Result<(), GraphqlError> {
        let trusted_documents_enabled = self.env.trusted_documents.is_enabled();
        let persisted_query_extension = mem::take(&mut request.extensions.persisted_query);
        let doc_id = mem::take(&mut request.operation_plan_cache_key.doc_id);

        match (trusted_documents_enabled, persisted_query_extension, doc_id) {
            (true, None, None) => {
                if self
                    .env
                    .trusted_documents
                    .bypass_header()
                    .map(|(name, value)| headers.find(name) == Some(value))
                    .unwrap_or_default()
                {
                    Ok(())
                } else {
                    Err(GraphqlError::new(
                        "Cannot execute a trusted document query: missing doc_id or the persistedQuery extension.",
                    ))
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
    ) -> Result<(), GraphqlError> {
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

    async fn handle_trusted_document_query(
        &self,
        request: &mut engine::Request,
        document_id: &str,
        client_name: Option<&str>,
    ) -> Result<(), GraphqlError> {
        let Some(client_name) = client_name else {
            return Err(GraphqlError::new(format!(
                "Trusted document queries must include the {CLIENT_NAME_HEADER_NAME} header"
            )));
        };

        let cache = &self.env.cache;
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

        match self.env.trusted_documents.fetch(client_name, document_id).await {
            Err(TrustedDocumentsError::RetrievalError(err)) => Err(GraphqlError::new(format!(
                "Internal server error while fetching trusted document: {err}"
            ))),
            Err(TrustedDocumentsError::DocumentNotFound) => {
                Err(GraphqlError::new(format!("Unknown document id: '{document_id}'")))
            }
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
                        log::error!(request.ray_id, "Cache error: {}", err);
                        GraphqlError::internal_server_error()
                    })?;

                request.operation_plan_cache_key.query = document_text;
                Ok(())
            }
        }
    }

    /// Handle a request using Automatic Persisted Queries.
    async fn handle_apq(
        &self,
        request: &mut engine::Request,
        PersistedQueryRequestExtension { version, sha256_hash }: &PersistedQueryRequestExtension,
    ) -> Result<(), GraphqlError> {
        if *version != 1 {
            return Err(GraphqlError::new("Persisted query version not supported"));
        }

        let cache = &self.env.cache;
        let key = cache.build_key(&format!("apq/sha256_{}", hex::encode(sha256_hash)));
        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(GraphqlError::new("Invalid persisted query sha256Hash"));
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
                    log::error!(request.ray_id, "Cache error: {}", err);
                    GraphqlError::internal_server_error()
                })?;
            return Ok(());
        }

        match cache.get_json::<AutomaticPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticPersistedQuery::V1 { query }) = entry.into_value() {
                    request.operation_plan_cache_key.query = query;
                    Ok(())
                } else {
                    Err(GraphqlError::new("Persisted query not found")
                        .with_error_code(ErrorCode::PersistedQueryNotFound))
                }
            }
            Err(err) => {
                log::error!(request.ray_id, "Cache error: {}", err);
                Err(GraphqlError::internal_server_error())
            }
        }
    }
}
