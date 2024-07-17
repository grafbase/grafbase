use grafbase_tracing::span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{FederationEntityResolverWalker, GraphqlEndpointId};
use serde::de::DeserializeSeed;
use std::future::Future;
use tracing::Instrument;

use crate::{
    execution::{ExecutionContext, PlanWalker, PlanningResult},
    operation::OperationType,
    response::{ResponseObjectsView, SubgraphResponse},
    sources::{
        graphql::deserialize::{EntitiesErrorsSeed, GraphqlResponseSeed},
        ExecutionResult, PreparedExecutor,
    },
    Runtime,
};

use super::{deserialize::EntitiesDataSeed, query::PreparedFederationEntityOperation, variables::SubgraphVariables};

pub(crate) struct FederationEntityPreparedExecutor {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityPreparedExecutor {
    pub fn prepare(
        resolver: FederationEntityResolverWalker<'_>,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<PreparedExecutor> {
        let subgraph = resolver.endpoint();
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(PreparedExecutor::FederationEntity(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        root_response_objects: ResponseObjectsView<'_>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<impl Future<Output = ExecutionResult<SubgraphResponse>> + Send + 'fut>
    where
        'ctx: 'fut,
    {
        let root_response_objects = root_response_objects.with_extra_constant_fields(vec![(
            "__typename".to_string(),
            serde_json::Value::String(
                ctx.engine
                    .schema
                    .walker()
                    .walk(schema::Definition::from(plan.logical_plan().as_ref().entity_id))
                    .name()
                    .to_string(),
            ),
        )]);
        let variables = SubgraphVariables {
            plan,
            variables: &self.operation.variables,
            inputs: vec![(&self.operation.entities_variable_name, root_response_objects)],
        };

        let subgraph = ctx.engine.schema.walk(self.subgraph_id);
        tracing::debug!(
            "Query {}\n{}\n{}",
            subgraph.name(),
            self.operation.query,
            serde_json::to_string_pretty(&variables).unwrap_or_default()
        );
        let json_body = serde_json::to_string(&serde_json::json!({
            "query": self.operation.query,
            "variables": variables
        }))
        .map_err(|err| format!("Failed to serialize query: {err}"))?;

        let span = SubgraphRequestSpan {
            name: subgraph.name(),
            operation_type: OperationType::Query.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: subgraph.url(),
        }
        .into_span();
        let span_clone = span.clone();

        Ok(async move {
            let bytes = ctx
                .engine
                .runtime
                .fetcher()
                .post(FetchRequest {
                    url: subgraph.url(),
                    json_body,
                    headers: ctx.headers_with_rules(subgraph.header_rules()),
                })
                .await?
                .bytes;
            tracing::debug!("{}", String::from_utf8_lossy(&bytes));

            let response = subgraph_response.as_mut();
            let status = GraphqlResponseSeed::new(
                EntitiesDataSeed {
                    response: response.clone(),
                    plan,
                },
                EntitiesErrorsSeed {
                    response,
                    response_keys: plan.response_keys(),
                },
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

            span.record_gql_status(status);

            Ok(subgraph_response)
        }
        .instrument(span_clone))
    }
}
