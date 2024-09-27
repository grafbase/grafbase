//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use crate::{
    execution::PreExecutionContext,
    request::Request,
    response::{ErrorCode, GraphqlError},
    Runtime,
};
use engine::PersistedQueryRequestExtension;
use futures::{future::BoxFuture, FutureExt};
use grafbase_telemetry::grafbase_client::X_GRAFBASE_CLIENT_NAME;
use runtime::trusted_documents_client::TrustedDocumentsError;
use std::borrow::Cow;

use super::{
    cache::{Document, Key},
    Engine,
};

type DocumentFuture<'a> = BoxFuture<'a, Result<Cow<'a, str>, GraphqlError>>;

pub(crate) struct OperationDocument<'a> {
    pub cache_key: String,
    pub load_fut: DocumentFuture<'a>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    /// Determines what document should be used for the request and provides an appropriate cache
    /// key for the operation cache and as a fallback a future to load said document.
    pub(super) fn determine_operation_document<'r, 'f>(
        &mut self,
        request: &'r Request,
    ) -> Result<OperationDocument<'f>, GraphqlError>
    where
        'ctx: 'f,
        'r: 'f,
    {
        let client_name = self.request_context.client.as_ref().map(|c| c.name.as_ref());
        let trusted_documents = self.engine.runtime.trusted_documents();
        let persisted_query_extension = request.extensions.persisted_query.as_ref();
        let doc_id = request.doc_id.as_ref();
        let name = request.operation_name.as_deref();
        let schema = &self.engine.schema;

        match (trusted_documents.is_enabled(), persisted_query_extension, doc_id) {
            (true, None, None) => {
                if trusted_documents
                    .bypass_header()
                    .map(|(name, value)| self.headers().get(name).and_then(|v| v.to_str().ok()) == Some(value))
                    .unwrap_or_default()
                {
                    let document = request
                        .query
                        .as_deref()
                        .ok_or_else(|| GraphqlError::new("Missing query", ErrorCode::BadRequest))?;

                    Ok(OperationDocument {
                        cache_key: Key::Operation {
                            name,
                            schema,
                            document: Document::Text(document),
                        }
                        .to_string(),
                        load_fut: Box::pin(std::future::ready(Ok(Cow::Borrowed(document)))),
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

                Ok(OperationDocument {
                    cache_key: Key::Operation {
                        name,
                        schema,
                        document: Document::TrustedDocumentId {
                            client_name,
                            doc_id: doc_id.clone(),
                        },
                    }
                    .to_string(),
                    load_fut: handle_trusted_document_query(self.engine, client_name, doc_id).boxed(),
                })
            }
            (false, Some(ext), _) => {
                let query = request
                    .query
                    .as_deref()
                    .ok_or_else(|| GraphqlError::new("Missing query", ErrorCode::BadRequest))?;

                Ok(OperationDocument {
                    cache_key: Key::Operation {
                        name,
                        schema,
                        document: Document::AutomaticallyPersistedQuery(ext),
                    }
                    .to_string(),
                    load_fut: handle_apq(query, ext).boxed(),
                })
            }
            (false, None, _) => {
                let document = request
                    .query
                    .as_deref()
                    .ok_or_else(|| GraphqlError::new("Missing query", ErrorCode::BadRequest))?;

                Ok(OperationDocument {
                    cache_key: Key::Operation {
                        name,
                        schema,
                        document: Document::Text(document),
                    }
                    .to_string(),
                    load_fut: Box::pin(std::future::ready(Ok(Cow::Borrowed(document)))),
                })
            }
        }
    }
}

async fn handle_trusted_document_query<'ctx, 'r, R: Runtime>(
    engine: &'ctx Engine<R>,
    client_name: &'ctx str,
    document_id: Cow<'r, str>,
) -> Result<Cow<'r, str>, GraphqlError> {
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
        Ok(document_text) => Ok(Cow::Owned(document_text)),
    }
}

/// Handle a request using Automatic Persisted Queries.
/// We don't cache anything here, we only rely on the operation cache. We might want to use an
/// external cache for this one day, but not another in-memory cache.
async fn handle_apq<'r, 'f>(
    query: &'r str,
    ext: &'r PersistedQueryRequestExtension,
) -> Result<Cow<'r, str>, GraphqlError> {
    if ext.version != 1 {
        return Err(GraphqlError::new(
            "Persisted query version not supported",
            ErrorCode::PersistedQueryError,
        ));
    }

    if !query.is_empty() {
        use sha2::{Digest, Sha256};
        let digest = <Sha256 as Digest>::digest(query.as_bytes()).to_vec();
        if digest != ext.sha256_hash {
            return Err(GraphqlError::new(
                "Invalid persisted query sha256Hash",
                ErrorCode::PersistedQueryError,
            ));
        }
        return Ok(Cow::Borrowed(query));
    }

    Err(GraphqlError::new(
        "Persisted query not found",
        ErrorCode::PersistedQueryNotFound,
    ))
}
