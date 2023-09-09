use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dynamodb::{DynamoDBBatchersData, DynamoDBContext};
use gateway_protocol::{
    ExecutionEngine, ExecutionError, ExecutionHealthRequest, ExecutionHealthResponse, ExecutionRequest,
    ExecutionResult, LocalSpecificConfig, VersionedRegistry,
};
use grafbase_engine::{registry::resolvers::graphql, RequestHeaders, Response};
use grafbase_local::{Bridge, LocalSearchEngine, UdfInvokerImpl};
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
}

#[allow(unused_variables, clippy::expect_fun_call)]
fn get_db_context(
    execution_request: &ExecutionRequest<gateway_protocol::LocalSpecificConfig>,
    env: &HashMap<String, String>,
) -> DynamoDBContext {
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
            .var_get(VarType::Var, BRIDGE_PORT_ENV_VAR)
            .expect(&format!("Missing env var {BRIDGE_PORT_ENV_VAR}"));

        let registry = env
            .var_get(VarType::Var, REGISTRY_ENV_VAR)
            .expect(&format!("Missing env var {REGISTRY_ENV_VAR}"));

        let mut local_env = HashMap::from([
            (BRIDGE_PORT_ENV_VAR.to_string(), bridge_port),
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

        Ok(Self { env: local_env })
    }
}

#[async_trait(? Send)]
impl ExecutionEngine for LocalExecution {
    type ConfigType = LocalSpecificConfig;
    type ExecutionResponse = Response;

    #[allow(clippy::expect_fun_call)]
    async fn execute(
        self: Arc<Self>,
        mut execution_request: ExecutionRequest<gateway_protocol::LocalSpecificConfig>,
    ) -> ExecutionResult<Response> {
        use worker::js_sys;

        let db_context = get_db_context(&execution_request, &self.env);

        let bridge_port = self
            .env
            .get(BRIDGE_PORT_ENV_VAR)
            .expect(&format!("Missing env var {BRIDGE_PORT_ENV_VAR}"));
        let dynamodb_batchers_data = DynamoDBBatchersData::new(
            &Arc::new(db_context.clone()),
            #[cfg(feature = "sqlite")]
            &Arc::new(dynamodb::LocalContext {
                bridge_port: bridge_port.to_string(),
            }),
        );
        let bridge_port = bridge_port
            .parse()
            .expect(&format!("{BRIDGE_PORT_ENV_VAR} should be an integer"));

        let fetch_log_endpoint = format!("http://{}:{}", std::net::Ipv4Addr::LOCALHOST, bridge_port);
        let global: worker::wasm_bindgen::JsValue = js_sys::global().into();
        js_sys::Reflect::set(&global, &"fetchLogEndpoint".into(), &fetch_log_endpoint.into()).unwrap();

        let search_engine = LocalSearchEngine::new(bridge_port);
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

        let bridge = Bridge::new(bridge_port);
        let resolver_engine = UdfInvokerImpl::create_engine(bridge.clone());
        let gql_request_exec_context = grafbase_runtime::GraphqlRequestExecutionContext {
            ray_id: ray_id.clone(),
            headers: execution_request.execution_headers.clone(),
        };

        let schema = grafbase_engine::Schema::build(registry)
            .data(dynamodb_batchers_data)
            .data(graphql::QueryBatcher::new())
            .data(search_engine)
            .data(resolver_engine)
            .data(gql_request_exec_context)
            .data(RequestHeaders::from(&execution_request.execution_headers))
            .extension(graphql_extensions::runtime_log::RuntimeLogExtension::new(Box::new(
                grafbase_local::LogEventReceiverImpl::new(bridge),
            )))
            .extension(graphql_extensions::authorization::AuthExtension::new(ray_id.clone()))
            .finish();

        // decorate the graphql request context with auth data for extension
        execution_request.request.data.insert(execution_request.auth);

        Ok(schema.execute(execution_request.request).await)
    }

    async fn health(
        self: Arc<Self>,
        _req: ExecutionHealthRequest<gateway_protocol::LocalSpecificConfig>,
    ) -> ExecutionResult<ExecutionHealthResponse> {
        Ok(ExecutionHealthResponse {
            deployment_id: "local".to_string(),
            ready: true,
            udf_results: vec![],
        })
    }
}
