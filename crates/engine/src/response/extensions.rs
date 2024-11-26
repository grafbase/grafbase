use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use schema::Schema;
use serde::Serialize;
use walker::Walk;

use crate::{
    operation::{Executable, OperationPlanContext, PlanId},
    prepare::PreparedOperation,
    resolver::Resolver,
};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafbase: Option<GrafbaseResponseExtension>,
}

impl ResponseExtensions {
    pub fn is_emtpy(&self) -> bool {
        self.grafbase.is_none()
    }
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GrafbaseResponseExtension {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_trace_id")]
    trace_id: Option<TraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_plan: Option<QueryPlan>,
}

impl GrafbaseResponseExtension {
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_query_plan(mut self, schema: &Schema, operation: &PreparedOperation) -> Self {
        let mut nodes = Vec::with_capacity(operation.plan.plans.len());
        // at least one edge.
        let mut edges = Vec::with_capacity(operation.plan.plans.len());

        let ctx = OperationPlanContext {
            schema,
            solved_operation: &operation.cached.solved,
            operation_plan: &operation.plan,
        };

        for plan in ctx.plans() {
            nodes.push(match &plan.resolver {
                Resolver::Introspection(_) => QueryPlanNode::IntrospectionResolver,
                Resolver::Graphql(resolver) => QueryPlanNode::GraphqlResolver(GraphqlResolverNode {
                    subgraph_name: resolver.endpoint_id.walk(ctx).subgraph_name().to_string(),
                    request: GraphqlRequest {
                        query: resolver.subgraph_operation.query.clone(),
                    },
                }),
                Resolver::FederationEntity(resolver) => QueryPlanNode::GraphqlResolver(GraphqlResolverNode {
                    subgraph_name: resolver.endpoint_id.walk(ctx).subgraph_name().to_string(),
                    request: GraphqlRequest {
                        query: resolver.subgraph_operation.query.clone(),
                    },
                }),
            });
            for child in plan.children() {
                if let Executable::Plan(child) = child {
                    edges.push((usize::from(plan.id), usize::from(child.id)))
                }
            }
        }

        self.query_plan = Some(QueryPlan { nodes, edges });
        self
    }
}

fn serialize_trace_id<S>(trace_id: &Option<TraceId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(trace_id) = trace_id {
        serializer.serialize_str(&format!("{trace_id:x}"))
    } else {
        serializer.serialize_none()
    }
}

#[derive(Debug, Serialize, id_derives::IndexedFields)]
#[serde(rename_all = "camelCase")]
struct QueryPlan {
    #[indexed_by(PlanId)]
    nodes: Vec<QueryPlanNode>,
    edges: Vec<(usize, usize)>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "__typename", rename_all = "PascalCase")]
enum QueryPlanNode {
    IntrospectionResolver,
    GraphqlResolver(GraphqlResolverNode),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlResolverNode {
    subgraph_name: String,
    request: GraphqlRequest,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlRequest {
    query: String,
}
