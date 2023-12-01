use std::{collections::HashMap, sync::Arc};

use axum::response::IntoResponse;
use common_types::auth::ExecutionAuth;
use dynamodb::{DynamoDBBatchersData, DynamoDBContext};
use engine::{registry::resolvers::graphql, RequestHeaders};
use gateway_core::{RequestContext, StreamingFormat};
use graphql_extensions::{authorization::AuthExtension, runtime_log::RuntimeLogExtension};
use postgres_connector_types::transport::TcpTransport;
use runtime::pg::PgTransportFactory;
use runtime_local::{Bridge, LocalPgTransportFactory, LocalSearchEngine, UdfInvokerImpl};

pub struct Executor {
    #[allow(dead_code)]
    env_vars: HashMap<String, String>,
    bridge: Bridge,
    registry: Arc<engine::Registry>,
    postgres: LocalPgTransportFactory,
}

impl Executor {
    pub(crate) async fn new(
        env_vars: HashMap<String, String>,
        bridge: Bridge,
        registry: Arc<engine::Registry>,
    ) -> Result<Self, crate::Error> {
        let postgres = {
            let mut transports = HashMap::new();
            for (name, definition) in &registry.postgres_databases {
                let transport = TcpTransport::new(definition.connection_string())
                    .await
                    .map_err(|error| crate::Error::Internal(error.to_string()))?;

                transports.insert(name.to_string(), transport);
            }
            LocalPgTransportFactory::new(transports)
        };

        Ok(Self {
            env_vars,
            bridge,
            registry,
            postgres,
        })
    }

    #[allow(clippy::panic, clippy::unused_async)]
    async fn get_db_context(&self, ctx: &crate::Context, auth: &ExecutionAuth) -> DynamoDBContext {
        #[cfg(not(feature = "sqlite"))]
        {
            const AWS_ACCESS_KEY_ID_ENV_VAR: &str = "AWS_ACCESS_KEY_ID";
            const AWS_SECRET_ACCESS_KEY_ENV_VAR: &str = "AWS_SECRET_ACCESS_KEY";
            const DYNAMODB_TABLE_NAME_ENV_VAR: &str = "DYNAMODB_TABLE_NAME";
            const DYNAMODB_REGION: &str = "DYNAMODB_REGION";

            let closest_aws_region =
                self.env_vars
                    .get(DYNAMODB_REGION)
                    .map_or(rusoto_core::Region::EuNorth1, |region: String| {
                        match region.strip_prefix("custom:") {
                            Some(suffix) => rusoto_core::Region::Custom {
                                name: "local".to_string(),
                                endpoint: suffix.to_string(),
                            },
                            None => <rusoto_core::Region as std::str::FromStr>::from_str(&region)
                                .ok()
                                .expect("Cannot parse {DYNAMODB_REGION}"),
                        }
                    });

            return DynamoDBContext::new(
                ctx.ray_id().to_string(),
                self.env_vars
                    .get(AWS_ACCESS_KEY_ID_ENV_VAR)
                    .expect("Missing env var {AWS_ACCESS_KEY_ID_ENV_VAR}"),
                self.env_vars
                    .get(AWS_SECRET_ACCESS_KEY_ENV_VAR)
                    .expect("Missing env var {AWS_SECRET_ACCESS_KEY_ENV_VAR}"),
                closest_aws_region,
                self.env_vars
                    .get(DYNAMODB_TABLE_NAME_ENV_VAR)
                    .expect("Missing env var {DYNAMODB_TABLE_NAME_ENV_VAR}"),
                HashMap::default(),
                auth.clone(),
            );
        }

        #[cfg(feature = "sqlite")]
        return DynamoDBContext::new(
            ctx.ray_id().to_string(),
            String::new(),
            String::new(),
            rusoto_core::Region::EuNorth1,
            String::new(),
            HashMap::default(),
            auth.clone(),
        );
    }

    async fn build_schema(
        &self,
        ctx: &Arc<crate::Context>,
        auth: ExecutionAuth,
    ) -> Result<engine::Schema, crate::Error> {
        let db_context = self.get_db_context(ctx, &auth).await;

        let dynamodb_batchers_data = DynamoDBBatchersData::new(
            &Arc::new(db_context.clone()),
            #[cfg(feature = "sqlite")]
            Some(&Arc::new(dynamodb::LocalContext {
                bridge_port: self.bridge.port().to_string(),
            })),
        );
        let runtime_ctx = runtime::context::Context::new(
            ctx,
            runtime::context::LogContext {
                fetch_log_endpoint_url: Some(format!(
                    "http://{}:{}",
                    std::net::Ipv4Addr::LOCALHOST,
                    self.bridge.port()
                )),
                request_log_event_id: None,
            },
        );

        let resolver_engine = UdfInvokerImpl::create_engine(self.bridge.clone());
        let search_engine = LocalSearchEngine::new(self.bridge.clone());

        Ok(engine::Schema::build(engine::Registry::clone(&self.registry))
            .data(dynamodb_batchers_data)
            .data(graphql::QueryBatcher::new())
            .data(search_engine)
            .data(resolver_engine)
            .data(auth)
            .data(PgTransportFactory::new(Box::new(self.postgres.clone())))
            .data(RequestHeaders::from(&ctx.headers_as_map()))
            .data(runtime_ctx)
            .extension(RuntimeLogExtension::new(Box::new(
                runtime_local::LogEventReceiverImpl::new(self.bridge.clone()),
            )))
            .extension(AuthExtension::new(ctx.ray_id().to_string()))
            .finish())
    }
}

#[async_trait::async_trait]
impl gateway_core::Executor for Executor {
    type Context = crate::Context;
    type Error = crate::Error;
    type Response = crate::Response;

    async fn execute(
        self: Arc<Self>,
        ctx: Arc<crate::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
    ) -> Result<engine::Response, crate::Error> {
        let schema = self.build_schema(&ctx, auth).await?;
        Ok(schema.execute(request).await)
    }

    async fn execute_stream(
        self: Arc<Self>,
        ctx: Arc<crate::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
        streaming_format: StreamingFormat,
    ) -> Result<Self::Response, crate::Error> {
        let schema = self.build_schema(&ctx, auth).await?;
        let payload_stream = Box::pin(schema.execute_stream(request));
        let (headers, bytes_stream) =
            gateway_core::encode_stream_response(ctx.as_ref(), payload_stream, streaming_format).await;
        Ok((headers, axum::body::StreamBody::new(bytes_stream))
            .into_response()
            .into())
    }
}
