//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use crate::{
    execution::PreExecutionContext,
    response::{ErrorCode, GraphqlError},
    Runtime,
};
use engine::{PersistedQueryRequestExtension, Request};
use futures::{future::BoxFuture, FutureExt};
use grafbase_telemetry::grafbase_client::X_GRAFBASE_CLIENT_NAME;
use runtime::trusted_documents_client::TrustedDocumentsError;
use std::borrow::Cow;
use tracing::instrument;

use super::cache::{Document, Key};

type DocumentFuture<'a> = BoxFuture<'a, Result<String, GraphqlError>>;

pub(crate) struct PreparedOperationDocument<'a> {
    pub cache_key: String,
    pub document_fut: Option<DocumentFuture<'a>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    /// Handle a request making use of APQ or trusted documents.
    #[instrument(skip_all)]
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
            // Apollo Client style trusted document query
            (true, maybe_ext, maybe_doc_id) => {
                let Some(client_name) = client_name else {
                    return Err(GraphqlError::new(
                        format!(
                            "Trusted document queries must include the {} header",
                            X_GRAFBASE_CLIENT_NAME.as_str()
                        ),
                        ErrorCode::TrustedDocumentError,
                    ));
                };

                let doc_id = if let Some(ext) = maybe_ext {
                    Cow::Owned(hex::encode(&ext.sha256_hash))
                } else if let Some(doc_id) = maybe_doc_id {
                    doc_id.into()
                } else {
                    unreachable!()
                };

                Ok(PreparedOperationDocument {
                    cache_key: Key::Operation {
                        name,
                        schema_version,
                        document: Document::TrustedDocumentId {
                            client_name,
                            doc_id: doc_id.clone(),
                        },
                    }
                    .to_string(),
                    document_fut: Some(self.handle_trusted_document_query(client_name, doc_id)?),
                })
            }
            (false, Some(ext), _) => Ok(PreparedOperationDocument {
                cache_key: Key::Operation {
                    name,
                    schema_version,
                    document: Document::AutomaticallyPersistedQuery(ext),
                }
                .to_string(),
                document_fut: self.handle_apq(request, ext)?,
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
        }
    }

    fn handle_trusted_document_query<'r, 'f>(
        &self,
        client_name: &'ctx str,
        document_id: Cow<'r, str>,
    ) -> Result<DocumentFuture<'f>, GraphqlError>
    where
        'r: 'f,
        'ctx: 'f,
    {
        let engine = self.engine;
        let fut = async move {
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
                Ok(document_text) => Ok(document_text),
            }
        }
        .boxed();
        Ok(fut)
    }

    /// Handle a request using Automatic Persisted Queries.
    /// We don't cache anything here, we only rely on the operation cache. We might want to use an
    /// external cache for this one day, but not another in-memory cache.
    fn handle_apq<'r, 'f>(
        &mut self,
        request: &'r Request,
        ext: &'r PersistedQueryRequestExtension,
    ) -> Result<Option<DocumentFuture<'f>>, GraphqlError>
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

        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if digest != ext.sha256_hash {
                return Err(GraphqlError::new(
                    "Invalid persisted query sha256Hash",
                    ErrorCode::PersistedQueryError,
                ));
            }
            return Ok(None);
        }

        let fut = async move {
            Err(GraphqlError::new(
                "Persisted query not found",
                ErrorCode::PersistedQueryNotFound,
            ))
        }
        .boxed();

        Ok(Some(fut))
    }
}
