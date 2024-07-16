use grafbase_tracing::span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{FederationEntityResolverWalker, GraphqlEndpointId, GraphqlEndpointWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use crate::{
    execution::{ExecutionContext, PlanWalker, PlanningResult},
    operation::OperationType,
    response::SubgraphResponseMutRef,
    sources::{
        graphql::deserialize::{EntitiesErrorsSeed, GraphqlResponseSeed},
        ExecutionResult, Executor, ExecutorInput, PreparedExecutor,
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

    pub fn new_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: ExecutorInput<'ctx, '_, R>,
    ) -> ExecutionResult<Executor<'ctx, R>> {
        let ExecutorInput {
            ctx,
            plan,
            root_response_objects,
        } = input;

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

        Ok(Executor::FederationEntity(FederationEntityExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            json_body,
            plan,
        }))
    }
}

pub(crate) struct FederationEntityExecutor<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedFederationEntityOperation,
    json_body: String,
    plan: PlanWalker<'ctx>,
}

impl<'ctx, R: Runtime> FederationEntityExecutor<'ctx, R> {
    #[tracing::instrument(skip_all)]
    pub async fn execute<'resp>(self, subgraph_response: SubgraphResponseMutRef<'resp>) -> ExecutionResult<()>
    where
        'ctx: 'resp,
    {
        let span = SubgraphRequestSpan {
            name: self.subgraph.name(),
            operation_type: OperationType::Query.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: self.subgraph.url(),
        }
        .into_span();

        async {
            let bytes = self
                .ctx
                .engine
                .runtime
                .fetcher()
                .post(FetchRequest {
                    url: self.subgraph.url(),
                    json_body: self.json_body,
                    headers: self.ctx.headers_with_rules(self.subgraph.header_rules()),
                })
                .await?
                .bytes;
            tracing::debug!("{}", String::from_utf8_lossy(&bytes));

            let response = subgraph_response.into_shared();
            let status = GraphqlResponseSeed::new(
                EntitiesDataSeed {
                    response: response.clone(),
                    plan: self.plan,
                },
                EntitiesErrorsSeed {
                    response,
                    response_keys: self.plan.response_keys(),
                },
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

            span.record_gql_status(status);

            Ok(())
        }
        .instrument(span.clone())
        .await
    }
}
