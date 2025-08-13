use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use schema::Schema;
use serde::Serialize;
use walker::Walk;

use crate::{
    mcp::McpResponseExtension,
    prepare::{Executable, OperationPlanContext, PlanId, PreparedOperation},
    resolver::{
        ExtensionResolver, FederationEntityResolver, FieldResolverExtension, GraphqlResolver, LookupProxiedResolver,
        Resolver, SelectionSetExtensionResolver,
    },
};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafbase: Option<GrafbaseResponseExtension>,
    #[serde(skip)]
    pub mcp: Option<McpResponseExtension>,
}

impl ResponseExtensions {
    pub(crate) fn is_empty(&self) -> bool {
        self.grafbase.is_none()
    }

    pub(crate) fn merge(self, other: Self) -> Self {
        let grafbase = match (self.grafbase, other.grafbase) {
            (None, None) => None,
            (Some(a), Some(b)) => Some(a.merge(b)),
            (Some(ext), None) | (None, Some(ext)) => Some(ext),
        };
        Self {
            grafbase,
            mcp: self.mcp.or(other.mcp),
        }
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
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            trace_id: self.trace_id.or(other.trace_id),
            query_plan: self.query_plan.or(other.query_plan),
        }
    }
}

impl GrafbaseResponseExtension {
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_query_plan(mut self, schema: &Schema, prepared_operation: &PreparedOperation) -> Self {
        let mut nodes = Vec::with_capacity(prepared_operation.plan.plans.len());
        // at least one edge.
        let mut edges = Vec::with_capacity(prepared_operation.plan.plans.len());

        let ctx = OperationPlanContext {
            schema,
            cached: &prepared_operation.cached,
            plan: &prepared_operation.plan,
        };

        for plan in ctx.plans() {
            nodes.push((ctx, &plan.resolver).into());
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
    Extension(ExtensionNode),
    Lookup(LookupNode),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlResolverNode {
    subgraph_name: String,
    request: GraphqlRequest,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionNode {
    id: extension_catalog::Id,
    directive_name: Option<String>,
    subgraph_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlRequest {
    query: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LookupNode {
    node: Box<QueryPlanNode>,
}

impl From<(OperationPlanContext<'_>, &Resolver)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &Resolver)) -> Self {
        match resolver {
            Resolver::Introspection(_) => QueryPlanNode::IntrospectionResolver,
            Resolver::Graphql(resolver) => (ctx, resolver).into(),
            Resolver::FederationEntity(resolver) => (ctx, resolver).into(),
            Resolver::FieldResolverExtension(resolver) => (ctx, resolver).into(),
            Resolver::Extension(resolver) => (ctx, resolver).into(),
            Resolver::Lookup(resolver) => QueryPlanNode::Lookup(LookupNode {
                node: Box::new(match &resolver.proxied {
                    LookupProxiedResolver::Graphql(resolver) => (ctx, resolver).into(),
                    LookupProxiedResolver::Extension(resolver) => (ctx, resolver).into(),
                    LookupProxiedResolver::SelectionSetExtension(resolver) => (ctx, resolver).into(),
                }),
            }),
            Resolver::SelectionSetExtension(resolver) => QueryPlanNode::Extension(ExtensionNode {
                directive_name: None,
                id: ctx.schema[resolver.definition.extension_id].clone(),
                subgraph_name: resolver.definition.subgraph_id.walk(ctx).name().to_string(),
            }),
        }
    }
}

impl From<(OperationPlanContext<'_>, &GraphqlResolver)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &GraphqlResolver)) -> Self {
        QueryPlanNode::GraphqlResolver(GraphqlResolverNode {
            subgraph_name: resolver.subgraph_id.walk(ctx).name().to_string(),
            request: GraphqlRequest {
                query: resolver.subgraph_operation.query.clone(),
            },
        })
    }
}

impl From<(OperationPlanContext<'_>, &FederationEntityResolver)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &FederationEntityResolver)) -> Self {
        QueryPlanNode::GraphqlResolver(GraphqlResolverNode {
            subgraph_name: resolver.subgraph_id.walk(ctx).name().to_string(),
            request: GraphqlRequest {
                query: resolver.subgraph_operation.query.clone(),
            },
        })
    }
}

impl From<(OperationPlanContext<'_>, &FieldResolverExtension)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &FieldResolverExtension)) -> Self {
        let directive = resolver.directive_id.walk(ctx);
        QueryPlanNode::Extension(ExtensionNode {
            directive_name: Some(directive.name().to_string()),
            id: ctx.schema[directive.extension_id].clone(),
            subgraph_name: directive.subgraph().name().to_string(),
        })
    }
}

impl From<(OperationPlanContext<'_>, &ExtensionResolver)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &ExtensionResolver)) -> Self {
        QueryPlanNode::Extension(ExtensionNode {
            directive_name: None,
            id: ctx.schema[resolver.definition.extension_id].clone(),
            subgraph_name: resolver.definition.subgraph_id.walk(ctx).name().to_string(),
        })
    }
}
impl From<(OperationPlanContext<'_>, &SelectionSetExtensionResolver)> for QueryPlanNode {
    fn from((ctx, resolver): (OperationPlanContext<'_>, &SelectionSetExtensionResolver)) -> Self {
        QueryPlanNode::Extension(ExtensionNode {
            directive_name: None,
            id: ctx.schema[resolver.definition.extension_id].clone(),
            subgraph_name: resolver.definition.subgraph_id.walk(ctx).name().to_string(),
        })
    }
}
