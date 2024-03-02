use std::{borrow::Cow, time::Duration};

use engine_v2_common::{
    BatchGraphqlRequest, GraphqlRequest, OperationCacheControlCacheKey, PersistedQueryRequestExtension,
    ResponseCacheKey, StreamingFormat,
};
pub use engine_v2_common::{ExecutionMetadata, HttpGraphqlRequest, HttpGraphqlResponse, ResponseBody, SchemaVersion};
use futures::{Future, StreamExt};
use headers::HeaderMapExt;
use runtime::{
    async_runtime::AsyncRuntime,
    auth::AccessToken,
    cache::{Cache, OperationCacheControl, TaggedResponseContent},
};

use crate::{
    error::error_response, AutomaticPersistedQuery, CacheControl, ErrorCode, ErrorExtensionValues, QueryEnv, Request,
    RequestExtensions, Response, Schema, ServerError,
};

pub struct EngineV1 {
    pub schema: Schema,
    pub schema_version: SchemaVersion,
    pub env: EngineV1Env,
}

#[derive(Clone)]
pub struct EngineV1Env {
    pub cache: Cache,
    pub cache_operation_cache_control: bool,
    pub async_runtime: AsyncRuntime,
}

impl EngineV1 {
    pub fn new(schema: Schema, schema_version: SchemaVersion, env: EngineV1Env) -> Self {
        Self {
            schema,
            schema_version,
            env,
        }
    }

    pub async fn unchecked_execute_introspection(
        &self,
        request: GraphqlRequest<'_, RequestExtensions>,
        // TODO: remove me once we have proper tracing...
        ray_id: &str,
    ) -> HttpGraphqlResponse {
        let mut request =
            Request::build(&request, ray_id).set_introspection_state(crate::IntrospectionState::ForceEnabled);
        request.operation_plan_cache_key.disable_operation_limits = true;
        self.schema.execute(request).await.into()
    }

    pub async fn execute_with_access_token(
        &self,
        headers: &http::HeaderMap,
        access_token: &AccessToken,
        // TODO: remove me once we have proper tracing...
        ray_id: &str,
        request: HttpGraphqlRequest<'_>,
    ) -> HttpGraphqlResponse {
        let batch_request = match BatchGraphqlRequest::<'_, RequestExtensions>::from_http_request(&request) {
            Ok(r) => r,
            Err(message) => return HttpGraphqlResponse::error(&message),
        };
        let streaming_format = headers.typed_get::<StreamingFormat>();
        match batch_request {
            BatchGraphqlRequest::Single(mut request) => {
                if let Some(streaming_format) = streaming_format {
                    if let Err(err) = self.handle_persisted_query(ray_id, &mut request).await {
                        return error_response(vec![err.into()]);
                    }
                    HttpGraphqlResponse::from_stream(
                        ray_id,
                        streaming_format,
                        self.schema.execute_stream(Request::build(&request, ray_id)),
                    )
                    .await
                } else {
                    self.execute_single(headers, access_token, ray_id, request).await
                }
            }
            BatchGraphqlRequest::Batch(requests) => {
                if streaming_format.is_some() {
                    return HttpGraphqlResponse::error("batch requests can't use multipart or event-stream responses");
                }
                HttpGraphqlResponse::batch_response(
                    futures_util::stream::iter(requests.into_iter())
                        .then(|request| self.execute_single(headers, access_token, ray_id, request))
                        .collect::<Vec<_>>()
                        .await,
                )
                .await
            }
        }
    }

    async fn handle_persisted_query(
        &self,
        ray_id: &str,
        request: &mut GraphqlRequest<'_, RequestExtensions>,
    ) -> Result<(), PersistedQueryError> {
        let Some(PersistedQueryRequestExtension { version, sha256_hash }) = &request.extensions.persisted_query else {
            return Ok(());
        };

        if *version != 1 {
            return Err(PersistedQueryError::UnsupportedVersion);
        }

        let key = format!("apq/sha256_{}", hex::encode(sha256_hash));
        if let Some(query) = request.query.as_ref() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(query.as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(PersistedQueryError::InvalidSha256Hash);
            }
            self.env
                .cache
                .put_json(
                    &key,
                    &AutomaticPersistedQuery::V1 {
                        query: query.to_string(),
                    },
                    Duration::from_secs(24 * 60 * 60),
                )
                .await
                .map_err(|err| {
                    log::error!(ray_id, "Cache error: {}", err);
                    PersistedQueryError::InternalServerError
                })?;
            return Ok(());
        }

        match self.env.cache.get_json::<AutomaticPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticPersistedQuery::V1 { query }) = entry {
                    request.query = Some(Cow::Owned(query));
                    Ok(())
                } else {
                    Err(PersistedQueryError::NotFound)
                }
            }
            Err(err) => {
                log::error!(ray_id, "Cache error: {}", err);
                Err(PersistedQueryError::InternalServerError)
            }
        }
    }

    async fn execute_single(
        &self,
        headers: &http::HeaderMap,
        access_token: &AccessToken,
        ray_id: &str,
        mut request: GraphqlRequest<'_, RequestExtensions>,
    ) -> HttpGraphqlResponse {
        if let Err(err) = self.handle_persisted_query(ray_id, &mut request).await {
            return error_response(vec![err.into()]);
        }
        let extensions = self.schema.create_extensions(Default::default());
        let extensions = extensions.clone();
        match self
            .schema
            .prepare_request(extensions, Request::build(&request, ray_id), Default::default())
            .await
        {
            Ok((env_builder, cache_control)) => {
                let env = env_builder.build();
                self.execute_once(headers, access_token, request, env, cache_control)
                    .await
            }
            Err(errors) => Response::bad_request(errors).into(),
        }
    }

    async fn execute_once(
        &self,
        headers: &http::HeaderMap,
        access_token: &AccessToken,
        request: GraphqlRequest<'_, RequestExtensions>,
        env: QueryEnv,
        cache_control: CacheControl,
    ) -> HttpGraphqlResponse {
        let operation_cache_control = OperationCacheControl::from(&cache_control);
        let execution = {
            let env = env.clone();
            let schema = self.schema.clone();
            async move {
                let fut = async { schema.execute_once(env.clone()).await.cache_control(cache_control) };
                futures_util::pin_mut!(fut);
                env.extensions
                    .execute(env.operation_name.as_deref(), &env.operation, &mut fut)
                    .await
            }
        };
        if matches!(env.operation_type(), engine_parser::types::OperationType::Mutation) {
            let response = execution.await;
            if !response.data.cache_tags().is_empty() {
                let tags = response.data.cache_tags().iter().cloned().collect::<Vec<_>>();
                let cache = self.env.cache.clone();
                self.env
                    .async_runtime
                    .spawn_faillible(async move { cache.purge_by_tags(tags).await });
            }
            response.into()
        } else if let Some(cache_key) =
            ResponseCacheKey::build(headers, access_token, &request, &operation_cache_control)
        {
            self.cached_execution(headers, request, operation_cache_control, cache_key, execution)
                .await
        } else {
            execution.await.into()
        }
    }

    async fn cached_execution(
        &self,
        headers: &http::HeaderMap,
        request: GraphqlRequest<'_, RequestExtensions>,
        operation_cache_control: OperationCacheControl,
        cache_key: ResponseCacheKey,
        execution: impl Future<Output = Response> + Send + 'static,
    ) -> HttpGraphqlResponse {
        if self.env.cache_operation_cache_control {
            self.background_cache_operation_cache_control(&request, &operation_cache_control);
        }
        let result = self
            .env
            .cache
            .cached_execution(
                &cache_key.to_string(),
                headers.typed_get(),
                operation_cache_control,
                async move {
                    let response = execution.await;
                    let body = response.into_json_bytes()?;
                    if response.errors.is_empty() {
                        let cache_tags = response.data.cache_tags().iter().cloned().collect::<Vec<_>>();
                        Ok(TaggedResponseContent { body, cache_tags })
                    } else {
                        Err(body)
                    }
                },
            )
            .await;
        match result {
            Ok(cached_response) => cached_response.into(),
            Err(body) => HttpGraphqlResponse::from_json_bytes(body.into()),
        }
    }

    fn background_cache_operation_cache_control(
        &self,
        request: &GraphqlRequest<'_, RequestExtensions>,
        operation_cache_control: &OperationCacheControl,
    ) {
        let operation_cache_control = operation_cache_control.clone();
        let key = OperationCacheControlCacheKey::build(&self.schema_version, request);
        let cache = self.env.cache.clone();
        self.env.async_runtime.spawn_faillible(async move {
            cache
                .put_json(
                    &key.to_string(),
                    &operation_cache_control,
                    Duration::from_secs(24 * 60 * 60),
                )
                .await
        });
    }
}

#[derive(Debug, thiserror::Error)]
enum PersistedQueryError {
    #[error("Persisted query not found")]
    NotFound,
    #[error("Persisted query version not supported")]
    UnsupportedVersion,
    #[error("Invalid persisted query sha256Hash")]
    InvalidSha256Hash,
    #[error("Internal server error")]
    InternalServerError,
}

impl From<PersistedQueryError> for ServerError {
    fn from(err: PersistedQueryError) -> Self {
        let message = err.to_string();
        let error = ServerError::new(message, None);
        if matches!(err, PersistedQueryError::NotFound) {
            ServerError {
                extensions: Some(ErrorExtensionValues(
                    [(
                        "code".to_string(),
                        crate::Value::String(ErrorCode::PersistedQueryNotFound.to_string()),
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
