use std::collections::HashMap;

use engine_value::ConstValue;
use schema::SubgraphResolver;
use serde::de::DeserializeSeed;

use super::{ExecutionContext, Executor, ExecutorError, ExecutorInput};
use crate::response::{ResponseObjectRoot, ResponsePartBuilder};

mod deserialize;
mod query;

#[derive(Debug)]
pub struct GraphqlExecutor<'a> {
    endpoint_name: String,
    url: String,
    payload: Payload<'a>,
    response_object_root: ResponseObjectRoot,
}

#[derive(Debug, serde::Serialize)]
pub struct Payload<'a> {
    query: String,
    variables: HashMap<String, &'a ConstValue>,
}

impl<'a> GraphqlExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx, 'ctx>,
        resolver: &SubgraphResolver,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &ctx.engine.schema[*subgraph_id];
        let query::Query { query, variables } =
            query::QueryBuilder::build(ctx.operation, ctx.plan_id, ctx.variables(), ctx.selection_set())
                .map_err(|err| ExecutorError::InternalError(format!("Failed to build query: {err}")))?;
        Ok(Executor::GraphQL(Self {
            endpoint_name: ctx.engine.schema[subgraph.name].to_string(),
            url: ctx.engine.schema[subgraph.url].clone(),
            payload: Payload { query, variables },
            response_object_root: input.root_response_objects.root(),
        }))
    }

    async fn send_request(&self) -> Result<bytes::Bytes, ExecutorError> {
        let response = reqwest::Client::new()
            .post(&self.url)
            .json(&self.payload)
            .send()
            .await
            .map_err(|err| format!("Request to '{}' failed with: {err}", self.endpoint_name))?;
        response
            .bytes()
            .await
            .map_err(|_err| "Failed to read response".to_string().into())
    }

    pub(super) async fn execute(
        self,
        ctx: ExecutionContext<'_, '_>,
        output: &mut ResponsePartBuilder,
    ) -> Result<(), ExecutorError> {
        #[cfg(feature = "cf-workers")]
        let bytes = match &ctx.engine.self_domain_configuration {
            Some((self_domain, service))
                if self
                    .url
                    .parse::<url::Url>()
                    .map_err(|_err| "invalid URL".to_string())?
                    .domain()
                    .is_some_and(|parsed_domain| {
                        parsed_domain
                            .rsplit('.')
                            .zip(self_domain.rsplit('.'))
                            .all(|(lhs, rhs)| lhs == rhs)
                    }) =>
            {
                let url = self.url.clone();
                Box::pin(send_wrapper::SendWrapper::new(async move {
                    use serde::Serialize;

                    let mut init = worker::RequestInit::new();
                    init.with_method(worker::Method::Post);
                    init.with_body(Some(
                        worker::js_sys::JSON::stringify(
                            &self
                                .payload
                                .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                                .expect("necessarily serializable"),
                        )
                        .expect("must succeed")
                        .into(),
                    ));
                    service
                        .fetch_request(worker::Request::new_with_init(&url, &init).map_err(|err| err.to_string())?)
                        .await
                        .map_err(|err| err.to_string())?
                        .bytes()
                        .await
                        .map_err(|err| err.to_string())
                        .map(From::from)
                }))
                .await?
            }
            _ => self.send_request().await?,
        };

        #[cfg(not(feature = "cf-workers"))]
        let bytes = self.send_request().await?;

        deserialize::UniqueRootSeed {
            ctx: &ctx,
            output,
            root: &self.response_object_root,
        }
        .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))
        .map_err(|err| format!("Deserialization failure: {err}"))?;

        Ok(())
    }
}
