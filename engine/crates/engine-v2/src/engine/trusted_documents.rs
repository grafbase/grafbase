//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use crate::{
    execution::PreExecutionContext,
    response::{ErrorCode, GraphqlError},
    Runtime,
};
use engine::{PersistedQueryRequestExtension, Request};
use futures::{future::BoxFuture, FutureExt};
use grafbase_tracing::grafbase_client::X_GRAFBASE_CLIENT_NAME;
use runtime::{hot_cache::HotCache, trusted_documents_client::TrustedDocumentsError};
use std::borrow::Cow;

use super::cache::{Document, Key};

type PersistedQueryFuture<'a> = BoxFuture<'a, Result<String, GraphqlError>>;

pub(crate) struct PreparedOperationDocument<'a> {
    pub cache_key: String,
    pub document_fut: Option<PersistedQueryFuture<'a>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    /// Handle a request making use of APQ or trusted documents.
    pub(super) fn prepare_operation_document<'r, 'f>(
        &mut self,
        request: &'r Request,
    ) -> Result<PreparedOperationDocument<'f>, GraphqlError>
    where
        'ctx: 'f,
        'r: 'f,
    {
        let client_name = self.request_context.client.as_ref().map(|c| c.name.as_ref());
        let trusted_documents_enabled = self.runtime.trusted_documents().is_enabled();
        let persisted_query_extension = request.extensions.persisted_query.as_ref();
        let document_id = request.document_id.as_ref();
        let name = request.operation_name();
        let schema_version = &self.engine.schema_version;

        match (trusted_documents_enabled, persisted_query_extension, document_id) {
            (true, None, None) => {
                if self
                    .runtime
                    .trusted_documents()
                    .bypass_header()
                    .map(|(name, value)| self.headers().get(name).and_then(|v| v.to_str().ok()) == Some(value))
                    .unwrap_or_default()
                {
                    Ok(PreparedOperationDocument {
                        cache_key: Key::Operation {
                            name,
                            schema_version,
                            document: Document::Text(request.query()),
                        }
                        .to_string(),
                        document_fut: None,
                    })
                } else {
                    let graphql_error = GraphqlError::new(
                        "Cannot execute a trusted document query: missing documentId, doc_id or the persistedQuery extension.",
                        ErrorCode::TrustedDocumentError
                    );
                    Err(graphql_error)
                }
            }
            (true, Some(ext), _) => Ok(PreparedOperationDocument {
                cache_key: Key::Operation {
                    name,
                    schema_version,
                    document: Document::PersistedQueryExt(ext),
                }
                .to_string(),
                document_fut: Some(self.handle_apollo_client_style_trusted_document_query(ext, client_name)?),
            }),
            (true, _, Some(document_id)) => Ok(PreparedOperationDocument {
                cache_key: Key::Operation {
                    name,
                    schema_version,
                    document: Document::Id(document_id),
                }
                .to_string(),
                document_fut: Some(self.handle_trusted_document_query(document_id.into(), client_name)?),
            }),
            (false, None, _) => Ok(PreparedOperationDocument {
                cache_key: Key::Operation {
                    name,
                    schema_version,
                    document: Document::Text(request.query()),
                }
                .to_string(),
                document_fut: None,
            }),
            (false, Some(ext), _) => Ok(PreparedOperationDocument {
                cache_key: Key::Operation {
                    name,
                    schema_version,
                    document: Document::PersistedQueryExt(ext),
                }
                .to_string(),
                document_fut: self.handle_apq(request, ext)?,
            }),
        }
    }

    fn handle_apollo_client_style_trusted_document_query<'r, 'f>(
        &self,
        ext: &'r PersistedQueryRequestExtension,
        client_name: Option<&'ctx str>,
    ) -> Result<PersistedQueryFuture<'f>, GraphqlError>
    where
        'r: 'f,
        'ctx: 'f,
    {
        use std::fmt::Write;

        let document_id = {
            let mut id = String::with_capacity(ext.sha256_hash.len() * 2);

            for byte in &ext.sha256_hash {
                write!(id, "{byte:02x}").expect("write to String to succeed");
            }

            id
        };

        self.handle_trusted_document_query(document_id.into(), client_name)
    }

    fn handle_trusted_document_query<'r, 'f>(
        &self,
        document_id: Cow<'r, str>,
        client_name: Option<&'ctx str>,
    ) -> Result<PersistedQueryFuture<'f>, GraphqlError>
    where
        'r: 'f,
        'ctx: 'f,
    {
        let Some(client_name) = client_name else {
            return Err(GraphqlError::new(
                format!(
                    "Trusted document queries must include the {} header",
                    X_GRAFBASE_CLIENT_NAME.as_str()
                ),
                ErrorCode::TrustedDocumentError,
            ));
        };

        let engine = self.engine;
        let fut = async move {
            let key = Key::TrustedDocument {
                client_name,
                document_id: &document_id,
            }
            .to_string();

            // First try fetching the document from cache.
            if let Some(document_text) = engine.trusted_documents_cache.get(&key).await {
                return Ok(document_text);
            }

            match engine
                .runtime
                .trusted_documents()
                .fetch(client_name, &document_id)
                .await
            {
                Err(TrustedDocumentsError::RetrievalError(err)) => Err(GraphqlError::new(
                    format!("Internal server error while fetching trusted document: {err}"),
                    ErrorCode::TrustedDocumentError,
                )),
                Err(TrustedDocumentsError::DocumentNotFound) => Err(GraphqlError::new(
                    format!("Unknown document id: '{document_id}'"),
                    ErrorCode::TrustedDocumentError,
                )),
                Ok(document_text) => {
                    engine.trusted_documents_cache.insert(key, document_text.clone()).await;
                    Ok(document_text)
                }
            }
        }
        .boxed();
        Ok(fut)
    }

    /// Handle a request using Automatic Persisted Queries.
    fn handle_apq<'r, 'f>(
        &mut self,
        request: &'r Request,
        ext: &'r PersistedQueryRequestExtension,
    ) -> Result<Option<PersistedQueryFuture<'f>>, GraphqlError>
    where
        'r: 'f,
        'ctx: 'f,
    {
        if ext.version != 1 {
            return Err(GraphqlError::new(
                "Persisted query version not supported",
                ErrorCode::PersistedQueryError,
            ));
        }

        let key = Key::Apq { ext }.to_string();

        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if digest != ext.sha256_hash {
                return Err(GraphqlError::new(
                    "Invalid persisted query sha256Hash",
                    ErrorCode::PersistedQueryError,
                ));
            }
            self.push_background_future(
                self.engine
                    .trusted_documents_cache
                    .insert(key, request.query().to_string())
                    .boxed(),
            );
            return Ok(None);
        }

        let engine = self.engine;
        let fut = async move {
            if let Some(query) = engine.trusted_documents_cache.get(&key).await {
                Ok(query)
            } else {
                Err(GraphqlError::new(
                    "Persisted query not found",
                    ErrorCode::PersistedQueryNotFound,
                ))
            }
        }
        .boxed();

        Ok(Some(fut))
    }
}
