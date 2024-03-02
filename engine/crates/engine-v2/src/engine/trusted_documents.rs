//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use super::{Engine, CLIENT_NAME_HEADER_NAME};
use crate::response::GraphqlError;
use engine::{AutomaticPersistedQuery, ErrorCode, PersistedQueryRequestExtension, RequestExtensions};
use engine_v2_common::GraphqlRequest;
use runtime::trusted_documents_service::TrustedDocumentsError;
use std::mem;

const CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

impl Engine {
    /// Handle a request making use of APQ or trusted documents.
    pub(super) async fn handle_persisted_query(
        &self,
        request: &mut GraphqlRequest<'_, RequestExtensions>,
        headers: &http::HeaderMap,
        ray_id: &str,
    ) -> Result<(), GraphqlError> {
        let client_name = headers
            .get(CLIENT_NAME_HEADER_NAME)
            .and_then(|value| value.to_str().ok());
        let trusted_documents_enabled = self.env.trusted_documents.is_enabled();
        let persisted_query_extension = mem::take(&mut request.extensions.persisted_query);
        let doc_id = mem::take(&mut request.doc_id);

        match (trusted_documents_enabled, persisted_query_extension, doc_id) {
            (true, None, None) => Err(GraphqlError::new(
                "Cannot execute a trusted document query: missing doc_id or the persistedQuery extension.",
            )),
            (true, Some(ext), _) => {
                self.handle_apollo_client_style_trusted_document_query(request, ext, client_name, ray_id)
                    .await
            }
            (true, _, Some(document_id)) => {
                self.handle_trusted_document_query(request, &document_id, client_name, ray_id)
                    .await
            }
            (false, None, _) => Ok(()),
            (false, Some(ext), _) => self.handle_apq(request, &ext, ray_id).await,
        }
    }

    async fn handle_apollo_client_style_trusted_document_query(
        &self,
        request: &mut GraphqlRequest<'_, RequestExtensions>,
        ext: PersistedQueryRequestExtension,
        client_name: Option<&str>,
        ray_id: &str,
    ) -> Result<(), GraphqlError> {
        use std::fmt::Write;

        let document_id = {
            let mut id = String::with_capacity(ext.sha256_hash.len() * 2);

            for byte in &ext.sha256_hash {
                write!(id, "{byte:02x}").expect("write to String to succeed");
            }

            id
        };

        self.handle_trusted_document_query(request, &document_id, client_name, ray_id)
            .await
    }

    async fn handle_trusted_document_query(
        &self,
        request: &mut GraphqlRequest<'_, RequestExtensions>,
        document_id: &str,
        client_name: Option<&str>,
        ray_id: &str,
    ) -> Result<(), GraphqlError> {
        let Some(client_name) = client_name else {
            return Err(GraphqlError::new(format!(
                "Trusted document queries must include the {CLIENT_NAME_HEADER_NAME} header"
            )));
        };

        let cache = &self.env.cache;
        let cache_key = format!("trusted_documents/{client_name}/{document_id}");

        // First try fetching the document from cache.
        if let Some(document_text) = cache
            .get(&cache_key)
            .await
            .ok()
            .flatten()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        {
            request.query = Some(document_text.into());
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
                self.env
                    .cache
                    .put(&cache_key, document_text.as_bytes(), CACHE_MAX_AGE)
                    .await
                    .map_err(|err| {
                        log::error!(ray_id, "Cache error: {}", err);
                        GraphqlError::internal_server_error()
                    })?;

                request.query = Some(document_text.into());
                Ok(())
            }
        }
    }

    /// Handle a request using Automatic Persisted Queries.
    async fn handle_apq(
        &self,
        request: &mut GraphqlRequest<'_, RequestExtensions>,
        PersistedQueryRequestExtension { version, sha256_hash }: &PersistedQueryRequestExtension,
        ray_id: &str,
    ) -> Result<(), GraphqlError> {
        if *version != 1 {
            return Err(GraphqlError::new("Persisted query version not supported"));
        }

        let key = format!("apq/sha256_{}", hex::encode(sha256_hash));
        if let Some(query) = request.query.as_ref() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(query.as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(GraphqlError::new("Invalid persisted query sha256Hash"));
            }
            self.env
                .cache
                .put_json(
                    &key,
                    &AutomaticPersistedQuery::V1 {
                        query: query.to_string(),
                    },
                    CACHE_MAX_AGE,
                )
                .await
                .map_err(|err| {
                    log::error!(ray_id, "Cache error: {}", err);
                    GraphqlError::internal_server_error()
                })?;
            return Ok(());
        }

        match self.env.cache.get_json::<AutomaticPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticPersistedQuery::V1 { query }) = entry {
                    request.query = Some(query.into());
                    Ok(())
                } else {
                    Err(GraphqlError::new("Persisted query not found")
                        .with_error_code(ErrorCode::PersistedQueryNotFound))
                }
            }
            Err(err) => {
                log::error!(ray_id, "Cache error: {}", err);
                Err(GraphqlError::internal_server_error())
            }
        }
    }
}
