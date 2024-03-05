use super::{Engine, CLIENT_NAME_HEADER_NAME};
use crate::response::GraphqlError;
use engine::PersistedQueryRequestExtension;
use runtime::trusted_documents::TrustedDocumentsError;
use std::mem;

impl Engine {
    /// Handle a request making use of APQ or trusted documents.
    pub(super) async fn handle_persisted_query(
        &self,
        request: &mut engine::Request,
        client_name: Option<&str>,
    ) -> Result<(), GraphqlError> {
        let enforce_trusted_documents = self.env.trusted_documents.trusted_documents_enabled();
        let persisted_query_extension = mem::take(&mut request.extensions.persisted_query);
        let doc_id = mem::take(&mut request.operation_plan_cache_key.doc_id);

        match (enforce_trusted_documents, persisted_query_extension, doc_id) {
            (true, None, None) => Err(GraphqlError::new("Only trusted document queries are accepted.")),
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

        match self.env.trusted_documents.fetch(client_name, &document_id).await {
            Err(TrustedDocumentsError::RetrievalError(err)) => Err(GraphqlError::new(format!(
                "Internal server error while fetching trusted document: {err}"
            ))),
            Err(TrustedDocumentsError::DocumentNotFound) => {
                Err(GraphqlError::new(format!("Document id unknown: {document_id}")))
            }
            Ok(document_text) => {
                cache
                    .put(
                        &cache_key,
                        runtime::cache::EntryState::Fresh,
                        document_text.clone().into_bytes(),
                        runtime::cache::CacheMetadata {
                            max_age: std::time::Duration::from_secs(24 * 60 * 60),
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
}
