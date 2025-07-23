//! Handling of trusted documents and Automatic Persisted Queries (APQ).

use crate::{
    Engine, Runtime,
    engine::cache::DocumentKey,
    response::{ErrorCode, GraphqlError},
};
use futures::{FutureExt, TryFutureExt as _, future::BoxFuture};
use grafbase_telemetry::grafbase_client::X_GRAFBASE_CLIENT_NAME;
use operation::{Request, extensions::PersistedQueryRequestExtension};
use runtime::{
    operation_cache::OperationCache as _,
    trusted_documents_client::{TrustedDocumentsEnforcementMode, TrustedDocumentsError},
};
use schema::LogLevel;
use sha2::Digest;
use std::borrow::Cow;
use tracing::Instrument;

use super::{CacheKey, OperationDocument, PrepareContext};

pub(super) struct ExtractedOperationDocument<'a> {
    pub key: DocumentKey<'a>,
    document_or_future: DocumentOrFuture<'a>,
}

enum DocumentOrFuture<'a> {
    Document(Cow<'a, str>),
    Future(DocumentFuture<'a>),
}

type DocumentFuture<'a> = BoxFuture<'a, Result<Cow<'a, str>, GraphqlError>>;

impl<'a> ExtractedOperationDocument<'a> {
    pub(super) async fn into_operation_document(self) -> Result<OperationDocument<'a>, GraphqlError> {
        match self.document_or_future {
            DocumentOrFuture::Document(content) => Ok(OperationDocument { key: self.key, content }),
            DocumentOrFuture::Future(future) => {
                let span = tracing::info_span!("load trusted document");
                Ok(OperationDocument {
                    key: self.key,
                    content: future.instrument(span).await?,
                })
            }
        }
    }
}

fn wrap_document(doc: &str) -> DocumentOrFuture<'_> {
    DocumentOrFuture::Document(Cow::Borrowed(doc))
}

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    /// Determines what document should be used for the request and provides an appropriate cache
    /// key for the operation cache and as a fallback a future to load said document.
    pub(super) async fn extract_operation_document<'r, 'f>(
        &mut self,
        request: &'r Request,
    ) -> Result<ExtractedOperationDocument<'f>, GraphqlError>
    where
        'ctx: 'f,
        'r: 'f,
    {
        let client_name = self.request_context.client.as_ref().map(|c| c.name.as_ref());
        let trusted_documents = self.runtime().trusted_documents();
        let persisted_query_extension = request.extensions.persisted_query.as_ref();
        let doc_id = request.doc_id.as_ref();
        let operation_name = request.operation_name.as_deref().map(Cow::Borrowed);
        let apq_enabled = self.schema().config.apq_enabled;

        match (trusted_documents.enforcement_mode(), persisted_query_extension, doc_id) {
            (TrustedDocumentsEnforcementMode::Enforce, None, None) => {
                if trusted_documents
                    .bypass_header()
                    .map(|(name, value)| self.headers().get(name).and_then(|v| v.to_str().ok()) == Some(value))
                    .unwrap_or_default()
                {
                    let document = request
                        .query
                        .as_deref()
                        .ok_or_else(|| GraphqlError::new("Missing query", ErrorCode::BadRequest))?;

                    Ok(ExtractedOperationDocument {
                        key: DocumentKey::Text {
                            operation_name,
                            document: Cow::Borrowed(document),
                        },
                        document_or_future: wrap_document(document),
                    })
                } else if let Some(query) = request.query.as_deref().filter(|query| !query.is_empty()) {
                    // Just an inline document.
                    let Some(client_name) = client_name else {
                        return Err(missing_client_name_error());
                    };

                    //  We have to take a guess on the document id here. The best guess is a sha256, because that's what is the most common across the ecosystem.
                    let hash = sha2::Sha256::digest(query.as_bytes());
                    let doc_id = hex::encode(hash);

                    Ok(ExtractedOperationDocument {
                        key: DocumentKey::TrustedDocumentId {
                            operation_name,
                            client_name: Cow::Borrowed(client_name),
                            doc_id: Cow::Owned(doc_id.clone()),
                        },
                        document_or_future: DocumentOrFuture::Future(
                            handle_trusted_document_query(self.engine, client_name, Cow::Owned(doc_id.clone()))
                                .map_err({
                                    let log_level =
                                        self.schema().config.trusted_documents.inline_document_unknown_log_level;

                                    move |err| {
                                        log_unknown_inline_document(log_level, doc_id.as_str());
                                        GraphqlError::new(
                                            format!("The query document does not match any trusted document. ({err})"),
                                            err.code,
                                        )
                                    }
                                })
                                .boxed(),
                        ),
                    })
                } else {
                    let graphql_error = GraphqlError::new(
                        "Cannot execute a trusted document query.",
                        ErrorCode::TrustedDocumentError,
                    );
                    Err(graphql_error)
                }
            }
            // Apollo Client style trusted document query
            (TrustedDocumentsEnforcementMode::Enforce, maybe_ext, maybe_doc_id) => {
                let Some(client_name) = client_name else {
                    return Err(missing_client_name_error());
                };

                let doc_id = if let Some(ext) = maybe_ext {
                    Cow::Owned(hex::encode(&ext.sha256_hash))
                } else if let Some(doc_id) = maybe_doc_id {
                    doc_id.into()
                } else {
                    unreachable!()
                };

                Ok(ExtractedOperationDocument {
                    key: DocumentKey::TrustedDocumentId {
                        operation_name,
                        client_name: Cow::Borrowed(client_name),
                        doc_id: doc_id.clone(),
                    },
                    document_or_future: DocumentOrFuture::Future(
                        handle_trusted_document_query(self.engine, client_name, doc_id).boxed(),
                    ),
                })
            }
            (TrustedDocumentsEnforcementMode::Allow, _, Some(doc_id)) => {
                let Some(client_name) = client_name else {
                    return Err(missing_client_name_error());
                };
                let query = request.query.as_deref();

                self.handle_trusted_document_document_query_permissive(
                    operation_name,
                    query,
                    client_name,
                    Cow::Borrowed(doc_id),
                )
                .await
            }
            (TrustedDocumentsEnforcementMode::Allow, Some(ext), _) if !apq_enabled => {
                let Some(client_name) = client_name else {
                    return Err(missing_client_name_error());
                };

                let query = request.query.as_deref();
                let doc_id = Cow::Owned(hex::encode(&ext.sha256_hash));

                self.handle_trusted_document_document_query_permissive(operation_name, query, client_name, doc_id)
                    .await
            }
            (TrustedDocumentsEnforcementMode::Ignore | TrustedDocumentsEnforcementMode::Allow, Some(ext), _) => {
                if !apq_enabled {
                    return Err(GraphqlError::new(
                        "Persisted query not found",
                        ErrorCode::PersistedQueryNotFound,
                    ));
                }

                Ok(ExtractedOperationDocument {
                    key: DocumentKey::AutomaticPersistedQuery {
                        operation_name,
                        ext: Cow::Borrowed(ext),
                    },
                    document_or_future: DocumentOrFuture::Future(handle_apq(request.query.as_deref(), ext).boxed()),
                })
            }
            (TrustedDocumentsEnforcementMode::Ignore, None, _)
            | (TrustedDocumentsEnforcementMode::Allow, None, None) => {
                let document = request
                    .query
                    .as_deref()
                    .ok_or_else(|| GraphqlError::new("Missing query", ErrorCode::BadRequest))?;

                Ok(ExtractedOperationDocument {
                    key: DocumentKey::Text {
                        operation_name,
                        document: Cow::Borrowed(document),
                    },
                    document_or_future: wrap_document(document),
                })
            }
        }
    }

    async fn handle_trusted_document_document_query_permissive<'r, 'f>(
        &self,
        operation_name: Option<Cow<'r, str>>,
        query: Option<&'r str>,
        client_name: &'r str,
        doc_id: Cow<'r, str>,
    ) -> Result<ExtractedOperationDocument<'f>, GraphqlError>
    where
        'ctx: 'f,
        'r: 'f,
    {
        // Reflect what actually gets executed in the key. If there is an inline query document, it will always take priority.
        if let Some(inline_document) = query.filter(|query| !query.is_empty()) {
            let cache_key = CacheKey::document(
                self.schema(),
                &DocumentKey::TrustedDocumentId {
                    operation_name: operation_name.clone(),
                    client_name: Cow::Borrowed(client_name),
                    doc_id: doc_id.clone(),
                },
            );

            let trusted_doc_matches_inline_doc = match self.operation_cache().get(&cache_key).await {
                Some(trusted_doc) => trusted_doc.document.content == inline_document,
                None => handle_trusted_document_query(self.engine, client_name, doc_id.clone())
                    .await
                    .map(|trusted_doc| trusted_doc == inline_document)
                    .unwrap_or(true),
            };

            if !trusted_doc_matches_inline_doc {
                log_document_id_and_query_mismatch(self.engine, doc_id.as_ref());
            }

            Ok(ExtractedOperationDocument {
                key: DocumentKey::Text {
                    operation_name,
                    document: Cow::Borrowed(inline_document),
                },
                document_or_future: DocumentOrFuture::Document(Cow::Borrowed(inline_document)),
            })
        } else {
            Ok(ExtractedOperationDocument {
                key: DocumentKey::TrustedDocumentId {
                    operation_name,
                    client_name: Cow::Borrowed(client_name),
                    doc_id: doc_id.clone(),
                },
                document_or_future: DocumentOrFuture::Future(
                    handle_trusted_document_query(self.engine, client_name, doc_id).boxed(),
                ),
            })
        }
    }
}

fn missing_client_name_error() -> GraphqlError {
    GraphqlError::new(
        format!(
            "Trusted document queries must include the {} header",
            X_GRAFBASE_CLIENT_NAME.as_str()
        ),
        ErrorCode::TrustedDocumentError,
    )
}

#[tracing::instrument(skip_all)]
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
        Err(TrustedDocumentsError::DocumentNotFound) => {
            log_unknown_trusted_document_id(engine, document_id.as_ref());

            Err(GraphqlError::new(
                format!("Unknown trusted document id: '{document_id}'"),
                ErrorCode::TrustedDocumentError,
            ))
        }
        Ok(document_text) => Ok(Cow::Owned(document_text)),
    }
}

/// Handle a request using Automatic Persisted Queries.
/// We don't cache anything here, we only rely on the operation cache. We might want to use an
/// external cache for this one day, but not another in-memory cache.
#[tracing::instrument(skip_all)]
async fn handle_apq<'r>(
    query: Option<&'r str>,
    ext: &'r PersistedQueryRequestExtension,
) -> Result<Cow<'r, str>, GraphqlError> {
    if ext.version != 1 {
        return Err(GraphqlError::new(
            "Persisted query version not supported",
            ErrorCode::PersistedQueryError,
        ));
    }

    if let Some(query) = query.filter(|q| !q.is_empty()) {
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

/// When a request contains a trusted document id, but the trusted document is not found in object storage. Default: INFO.
fn log_unknown_trusted_document_id<R: Runtime>(engine: &Engine<R>, document_id: &str) {
    const MESSAGE: &str = "Unknown trusted document";

    match engine.schema.config.trusted_documents.document_id_unknown_log_level {
        LogLevel::Off => (),
        LogLevel::Debug => tracing::debug!(MESSAGE, document_id),
        LogLevel::Info => tracing::info!(MESSAGE, document_id),
        LogLevel::Warn => tracing::warn!(MESSAGE, document_id),
        LogLevel::Error => tracing::error!(MESSAGE, document_id),
    }
}

/// When a request contains a trusted document id and an inline document in `query`, but the trusted document body does not match the inline document.
fn log_document_id_and_query_mismatch<R: Runtime>(engine: &Engine<R>, document_id: &str) {
    const MESSAGE: &str = "The request contained both a GraphQL query document and a document id, but the trusted document with that ID does not match the inline query document.";

    match engine
        .schema
        .config
        .trusted_documents
        .document_id_and_query_mismatch_log_level
    {
        LogLevel::Off => (),
        LogLevel::Debug => tracing::debug!(MESSAGE, document_id),
        LogLevel::Info => tracing::info!(MESSAGE, document_id),
        LogLevel::Warn => tracing::warn!(MESSAGE, document_id),
        LogLevel::Error => tracing::error!(MESSAGE, document_id),
    }
}

/// When a request contains only an inline document but it does not correspond to any trusted document.
fn log_unknown_inline_document(log_level: LogLevel, document_id: &str) {
    const MESSAGE: &str = "The GraphQL query document in the request does not match any trusted document.";

    match log_level {
        LogLevel::Off => (),
        LogLevel::Debug => tracing::debug!(MESSAGE, document_id),
        LogLevel::Info => tracing::info!(MESSAGE, document_id),
        LogLevel::Warn => tracing::warn!(MESSAGE, document_id),
        LogLevel::Error => tracing::error!(MESSAGE, document_id),
    }
}
