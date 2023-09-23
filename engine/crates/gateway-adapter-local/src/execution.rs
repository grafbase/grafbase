use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use dynamodb::{DynamoDBBatchersData, DynamoDBContext};
use engine::{registry::resolvers::graphql, registry::VersionedRegistry, RequestHeaders, Response, StreamingPayload};
use futures_util::{future::BoxFuture, stream::BoxStream, AsyncBufReadExt, SinkExt, Stream, StreamExt};
use gateway_adapter::{ExecutionEngine, ExecutionError, ExecutionRequest, ExecutionResult};
use gateway_core::StreamingFormat;
use runtime_local::{Bridge, LocalSearchEngine, UdfInvokerImpl};
use worker::Env;
use worker_env::{EnvExt, VarType};

pub const REGISTRY_ENV_VAR: &str = "REGISTRY";
pub const BRIDGE_PORT_ENV_VAR: &str = "BRIDGE_PORT";

cfg_if::cfg_if! {
    if #[cfg(not(feature = "sqlite"))] {
        const AWS_ACCESS_KEY_ID_ENV_VAR: &str = "AWS_ACCESS_KEY_ID";
        const AWS_SECRET_ACCESS_KEY_ENV_VAR: &str = "AWS_SECRET_ACCESS_KEY";
        const DYNAMODB_TABLE_NAME_ENV_VAR: &str = "DYNAMODB_TABLE_NAME";
    }
}

const RAY_ID_HEADER: &str = "ray-id";

pub struct LocalExecution {
    env: HashMap<String, String>,
    bridge_port: u16,
}

#[allow(unused_variables, clippy::expect_fun_call)]
fn get_db_context(execution_request: &ExecutionRequest, env: &HashMap<String, String>) -> DynamoDBContext {
    #[cfg(not(feature = "sqlite"))]
    {
        return DynamoDBContext::new(
            execution_request
                .execution_headers
                .get(RAY_ID_HEADER)
                .map(|v| v.to_string())
                .unwrap_or_default(),
            env.get(AWS_ACCESS_KEY_ID_ENV_VAR)
                .expect(&format!("Missing env var {AWS_ACCESS_KEY_ID_ENV_VAR}"))
                .to_string(),
            env.get(AWS_SECRET_ACCESS_KEY_ENV_VAR)
                .expect(&format!("Missing env var {AWS_SECRET_ACCESS_KEY_ENV_VAR}"))
                .to_string(),
            execution_request.closest_aws_region.clone(),
            env.get(DYNAMODB_TABLE_NAME_ENV_VAR)
                .expect(&format!("Missing env var {DYNAMODB_TABLE_NAME_ENV_VAR}"))
                .to_string(),
            Default::default(),
            execution_request.auth.clone(),
        );
    }

    #[cfg(feature = "sqlite")]
    return DynamoDBContext::new(
        execution_request
            .execution_headers
            .get(RAY_ID_HEADER)
            .map(|v| v.to_string())
            .unwrap_or_default(),
        String::new(),
        String::new(),
        execution_request.closest_aws_region.clone(),
        String::new(),
        Default::default(),
        execution_request.auth.clone(),
    );
}

impl LocalExecution {
    #[allow(clippy::expect_fun_call)]
    pub fn from_env(env: &Env) -> worker::Result<Self> {
        let bridge_port = env
            .var_get::<String>(VarType::Var, BRIDGE_PORT_ENV_VAR)
            .expect(&format!("Missing env var {BRIDGE_PORT_ENV_VAR}"));

        let registry = env
            .var_get(VarType::Var, REGISTRY_ENV_VAR)
            .expect(&format!("Missing env var {REGISTRY_ENV_VAR}"));

        let mut local_env = HashMap::from([
            (BRIDGE_PORT_ENV_VAR.to_string(), bridge_port.clone()),
            (REGISTRY_ENV_VAR.to_string(), registry),
        ]);

        #[cfg(not(feature = "sqlite"))]
        {
            let dynamodb_table = env
                .var_get(VarType::Var, DYNAMODB_TABLE_NAME_ENV_VAR)
                .expect(&format!("Missing env var {DYNAMODB_TABLE_NAME_ENV_VAR}"));
            let aws_access_key_id = env
                .var_get(VarType::Secret, AWS_ACCESS_KEY_ID_ENV_VAR)
                .expect(&format!("Missing env var {AWS_ACCESS_KEY_ID_ENV_VAR}"));
            let aws_secret_access_key = env
                .var_get(VarType::Secret, AWS_SECRET_ACCESS_KEY_ENV_VAR)
                .expect(&format!("Missing env var {AWS_SECRET_ACCESS_KEY_ENV_VAR}"));

            local_env.insert(AWS_ACCESS_KEY_ID_ENV_VAR.to_string(), aws_access_key_id);
            local_env.insert(AWS_SECRET_ACCESS_KEY_ENV_VAR.to_string(), aws_secret_access_key);
            local_env.insert(DYNAMODB_TABLE_NAME_ENV_VAR.to_string(), dynamodb_table);
        }

        let bridge_port = bridge_port
            .parse()
            .expect(&format!("{BRIDGE_PORT_ENV_VAR} should be an integer"));

        Ok(Self {
            env: local_env,
            bridge_port,
        })
    }

    #[allow(clippy::expect_fun_call)]
    pub fn build_schema(&self, execution_request: &ExecutionRequest) -> ExecutionResult<engine::Schema> {
        let db_context = get_db_context(execution_request, &self.env);

        let dynamodb_batchers_data = DynamoDBBatchersData::new(
            &Arc::new(db_context.clone()),
            #[cfg(feature = "sqlite")]
            &Arc::new(dynamodb::LocalContext {
                bridge_port: self.bridge_port.to_string(),
            }),
        );

        let fetch_log_endpoint_url = Some(format!("http://{}:{}", std::net::Ipv4Addr::LOCALHOST, self.bridge_port));
        let search_engine = LocalSearchEngine::new(self.bridge_port);
        let versioned_registry: VersionedRegistry<'_> = serde_json::from_str(
            self.env
                .get(REGISTRY_ENV_VAR)
                .expect("should have REGISTRY env var defined"),
        )
        .map_err(|e| ExecutionError::InternalError(e.to_string()))?;

        let registry = versioned_registry.registry.into_owned();

        let ray_id = execution_request
            .execution_headers
            .get(RAY_ID_HEADER)
            .map(|v| v.to_string())
            .unwrap_or_else(|| ulid::Ulid::new().to_string()); // Random one in local.

        let bridge = Bridge::new(self.bridge_port);
        let resolver_engine = UdfInvokerImpl::create_engine(bridge.clone());
        let gql_request_exec_context = runtime::GraphqlRequestExecutionContext {
            ray_id: ray_id.clone(),
            fetch_log_endpoint_url,
            headers: execution_request.execution_headers.clone(),
        };

        Ok(engine::Schema::build(registry)
            .data(dynamodb_batchers_data)
            .data(graphql::QueryBatcher::new())
            .data(search_engine)
            .data(resolver_engine)
            .data(gql_request_exec_context)
            .data(RequestHeaders::from(&execution_request.execution_headers))
            .extension(graphql_extensions::runtime_log::RuntimeLogExtension::new(Box::new(
                runtime_local::LogEventReceiverImpl::new(bridge),
            )))
            .extension(graphql_extensions::authorization::AuthExtension::new(ray_id.clone()))
            .finish())
    }
}

#[async_trait(? Send)]
impl ExecutionEngine for LocalExecution {
    type ExecutionResponse = Response;

    async fn execute(self: Arc<Self>, mut execution_request: ExecutionRequest) -> ExecutionResult<Response> {
        let schema = self.build_schema(&execution_request)?;

        // decorate the graphql request context with auth data for extension
        execution_request.request.data.insert(execution_request.auth);

        Ok(schema.execute(execution_request.request).await)
    }

    async fn execute_stream(
        self: Arc<Self>,
        mut execution_request: ExecutionRequest,
        streaming_format: StreamingFormat,
    ) -> ExecutionResult<(worker::Response, Option<BoxFuture<'static, ()>>)> {
        let schema = self.build_schema(&execution_request)?;

        // decorate the graphql request context with auth data for extension
        execution_request.request.data.insert(execution_request.auth);

        let payload_stream = schema.execute_stream(execution_request.request);

        let (response_stream, process_future) = into_byte_stream_and_future(payload_stream, streaming_format);

        let mut response = worker::Response::from_stream(response_stream)?;

        let headers = response.headers_mut();
        headers.set("Cache-Control", "no-cache")?;
        headers.set(
            "Content-Type",
            match streaming_format {
                StreamingFormat::IncrementalDelivery => "multipart/mixed; boundary=\"-\"",
                StreamingFormat::GraphQLOverSSE => "text/event-stream",
            },
        )?;

        return Ok((response, Some(process_future)));
    }
}

fn into_byte_stream_and_future(
    payload_stream: impl Stream<Item = StreamingPayload> + Send + 'static,
    streaming_format: StreamingFormat,
) -> (BoxStream<'static, Result<Bytes, String>>, BoxFuture<'static, ()>) {
    match streaming_format {
        StreamingFormat::IncrementalDelivery => {
            let mut byte_stream = Box::pin(multipart_stream::serialize(
                payload_stream.map(|payload| {
                    Ok(multipart_stream::Part {
                        headers: Default::default(),
                        body: Bytes::from(serde_json::to_vec(&payload).map_err(|e| e.to_string())?),
                    })
                }),
                "-",
            ));

            // We should be able to just return the above stream,  but miniflare as run
            // in the CLI has an issue where _sometimes_ code run inside a response stream
            // is considered outside of the request context and we get panics when doing I/O etc.
            //
            // We work around this by returning a future that does the work, which can
            // be resolved inside a wait_until, guaranteeing it's inside the request context.
            //
            // Miniflare 3 doesn't appear to have this problem so we can probably get
            // rid of this hack when/if we upgrade
            let (mut tx, rx) = futures_channel::mpsc::channel(10);
            let process_stream = Box::pin(async move {
                while let Some(bytes) = byte_stream.next().await {
                    tx.send(bytes).await.ok();
                }
            });

            (Box::pin(rx), process_stream)
        }
        StreamingFormat::GraphQLOverSSE => {
            let mut payload_stream = Box::pin(payload_stream);

            let (sse_sender, sse_encoder) = async_sse::encode();
            let response_stream = sse_encoder.lines().map(|line| {
                line.map(|mut line| {
                    line.push_str("\r\n");
                    line.into()
                })
                .map_err(|e| e.to_string())
            });

            let sender_future = Box::pin(async move {
                while let Some(payload) = payload_stream.next().await {
                    let payload_json = match serde_json::to_string(&payload) {
                        Ok(json) => json,
                        Err(error) => {
                            tracing::error!("Could not encode StreamingPayload as JSON: {error:?}");
                            return;
                        }
                    };

                    if let Err(error) = sse_sender.send("next", &payload_json, None).await {
                        tracing::error!("Could not send next payload via sse_sender: {error}");
                        return;
                    }
                }

                // The GraphQLOverSSE spec suggests we just need the event name on the complete
                // event but the SSE spec says that you should drop events with an empty data
                // buffer.  So I'm just putting null in the data buffer for now.
                if let Err(error) = sse_sender.send("complete", "null", None).await {
                    tracing::error!("Could not send complete payload via sse_sender: {error}");
                }
            });

            (Box::pin(response_stream), sender_future)
        }
    }
}
