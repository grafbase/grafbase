#![allow(unused)]
#![allow(unused_crate_dependencies)]

use grafbase_workspace_hack as _;
use id_newtypes::BitSet;
use schema::{
    Definition, FieldDefinition, FieldDefinitionId, RequiredField, RequiredFieldId, RequiredFieldSet,
    RequiredFieldSetId, RequiredFieldSetRecord, ResolverDefinitionId, Schema, SubgraphId, TypeSystemDirective,
};
use walker::Walk;

use std::{borrow::Cow, collections::HashSet, convert::Infallible, num::NonZero, time::Instant};

use itertools::Itertools;
use petgraph::{
    adj::EdgeReference,
    data::Build,
    dot::{Config, Dot},
    graph::{EdgeIndex, NodeIndex},
    visit::{Bfs, EdgeRef, IntoEdgeReferences, IntoEdgesDirected, IntoNodeReferences, NodeRef},
    Directed, Direction,
};
use tracing::instrument;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct ResolverDecisionId(NonZero<u16>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct EdgeId(NonZero<u16>);

#[derive(id_derives::IndexedFields)]
struct PlanningTree {
    #[indexed_by(ResolverDecisionId)]
    decisions: Vec<ResolverDecision>,
    root: ResolverDecisionTree,
}

struct ResolverDecisionTree {
    fields: Vec<ResolverDecision>,
}

struct ResolverDecision {
    possibilities: Vec<(usize, ResolverDecisionTree)>,
}

pub type Cost = u16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Node<F> {
    Root,
    Field(F),
    Resolver(ResolverDefinitionId),
    FieldResolver(FieldResolver),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FieldResolver {
    resolver_definition_id: ResolverDefinitionId,
    field_definition_id: FieldDefinitionId,
}

impl FieldResolver {
    fn new(resolver_definition_id: ResolverDefinitionId, field_definition: FieldDefinition<'_>) -> Self {
        FieldResolver {
            resolver_definition_id,
            field_definition_id: field_definition.id(),
        }
    }

    fn child(&self, schema: &Schema, field_definition_id: FieldDefinitionId) -> Option<FieldResolver> {
        let resolver_definition = self.resolver_definition_id.walk(schema);
        if resolver_definition.can_provide(field_definition_id) {
            Some(FieldResolver {
                resolver_definition_id: self.resolver_definition_id,
                field_definition_id,
            })
        } else {
            None
        }
    }
}

impl<F> Node<F> {
    fn as_resolver(&self) -> Option<ResolverDefinitionId> {
        match self {
            Node::Resolver(id) => Some(*id),
            _ => None,
        }
    }

    fn as_field_resolver(&self) -> Option<&FieldResolver> {
        match self {
            Node::FieldResolver(r) => Some(r),
            _ => None,
        }
    }

    fn as_field(&self) -> Option<F>
    where
        F: Copy,
    {
        match self {
            Node::Field(field_id) => Some(*field_id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Edge {
    Resolver(Cost),
    CanResolveField(Cost),
    Resolves,
    Field,
    TypenameField,
    Requires,
}

impl Edge {
    fn is_resolver(&self) -> bool {
        matches!(self, Self::Resolver(_))
    }
}

pub trait Operation: std::fmt::Debug {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug + Ord;

    // Operation utilities
    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn field_satisfies(&self, field_id: Self::FieldId, requirement: RequiredField<'_>) -> bool;
    fn create_extra_field(&mut self, requirement: RequiredField<'_>) -> Self::FieldId;

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;

    // Dot graph utilities
    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str>;
}

pub struct Plan<'ctx, Op: Operation> {
    schema: &'ctx Schema,
    operation: &'ctx mut Op,
    graph: petgraph::stable_graph::StableGraph<Node<Op::FieldId>, Edge>,
    root: NodeIndex,
    field_nodes: Vec<NodeIndex>,
    scalar_nodes: Vec<NodeIndex>,
    fields_stack: Vec<Field<Op::FieldId>>,
    requirements_stack: Vec<Requirement<'ctx>>,
}

impl<'ctx, Op: Operation> std::ops::Index<Op::FieldId> for Plan<'ctx, Op> {
    type Output = NodeIndex;
    fn index(&self, field_id: Op::FieldId) -> &Self::Output {
        let ix: usize = field_id.into();
        &self.field_nodes[ix]
    }
}

struct Requirement<'ctx> {
    parent_field_node: NodeIndex,
    petitioner_node: NodeIndex,
    required_field_set: RequiredFieldSet<'ctx>,
}

struct ParentResolver {
    field_resolver_node: NodeIndex,
    subgraph_id: SubgraphId,
}

struct Field<Id> {
    parent_field_node: NodeIndex,
    parent_resolver: Option<ParentResolver>,
    field_id: Id,
}

impl<'ctx, Op: Operation> Plan<'ctx, Op> {
    pub fn build(schema: &'ctx Schema, operation: &'ctx mut Op) -> Self {
        let n = operation.field_ids().len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root = graph.add_node(Node::Root);

        let mut plan = Plan {
            schema,
            operation,
            root,
            graph,
            scalar_nodes: Vec::new(),
            field_nodes: Vec::new(),
            fields_stack: Vec::new(),
            requirements_stack: Vec::new(),
        };

        plan.ingest_operation();
        tracing::debug!("After operation ingestion:\n{}", plan.dot_graph());
        plan.prune_resolvers_not_providing_any_scalars();
        tracing::debug!("After pruning resolvers:\n{}", plan.dot_graph());

        plan
    }

    fn ingest_operation(&mut self) {
        self.field_nodes = self
            .operation
            .field_ids()
            .map(|field_id| self.graph.add_node(Node::Field(field_id)))
            .collect();

        self.scalar_nodes = self
            .operation
            .field_ids()
            .filter_map(|field_id| {
                if self
                    .operation
                    .field_defintion(field_id)
                    .walk(self.schema)
                    .is_some_and(|definition| matches!(definition.ty().definition(), Definition::Scalar(_)))
                {
                    Some(self[field_id])
                } else {
                    None
                }
            })
            .collect();

        self.fields_stack = self
            .operation
            .root_selection_set()
            .map(|field_id| Field {
                parent_field_node: self.root,
                parent_resolver: None,
                field_id,
            })
            .collect();

        loop {
            if let Some(field) = self.fields_stack.pop() {
                self.create_field_resolvers(field)
            } else if let Some(requirement) = self.requirements_stack.pop() {
                self.handle_requirements(requirement)
            } else {
                break;
            }
        }
    }

    /// field resolver -> resolver -> nested field resolver -> nested field
    /// field resolver -> nested field resolver -> nested field
    fn create_field_resolvers(
        &mut self,
        Field {
            parent_field_node,
            parent_resolver,
            field_id,
        }: Field<Op::FieldId>,
    ) {
        let field_node = self[field_id];
        let Some(definition_id) = self.operation.field_defintion(field_id) else {
            self.graph.add_edge(parent_field_node, field_node, Edge::TypenameField);
            return;
        };
        let field_definition = definition_id.walk(self.schema);

        /// if it's the first time we see this field, we add any requirements from type system
        /// directives.
        if self.graph.edges(field_node).next().is_none() {
            for required_field_set in field_definition.directives().filter_map(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_) => None,
                TypeSystemDirective::Authorized(directive) => directive.fields(),
            }) {
                self.requirements_stack.push(Requirement {
                    parent_field_node,
                    petitioner_node: field_node,
                    required_field_set,
                })
            }
        }
        self.graph.add_edge(parent_field_node, field_node, Edge::Field);

        if let Some((parent_resolver_node, resolver)) = parent_resolver.as_ref().and_then(
            |ParentResolver {
                 field_resolver_node, ..
             }| {
                self.graph[*field_resolver_node]
                    .as_field_resolver()
                    .unwrap()
                    .child(self.schema, field_definition.id())
                    .map(|r| (*field_resolver_node, r))
            },
        ) {
            let resolver_definition = resolver.resolver_definition_id.walk(self.schema);
            let field_resolver_node = self.graph.add_node(Node::FieldResolver(resolver));
            self.graph.add_edge(field_resolver_node, field_node, Edge::Resolves);
            self.graph
                .add_edge(parent_resolver_node, field_resolver_node, Edge::CanResolveField(0));
            for nested_field_id in self.operation.subselection(field_id) {
                self.fields_stack.push(Field {
                    parent_field_node: field_node,
                    parent_resolver: Some(ParentResolver {
                        field_resolver_node,
                        subgraph_id: resolver_definition.subgraph_id(),
                    }),
                    field_id: nested_field_id,
                })
            }
        }

        let parent_node = parent_resolver
            .as_ref()
            .map(
                |ParentResolver {
                     field_resolver_node, ..
                 }| *field_resolver_node,
            )
            .unwrap_or(parent_field_node);
        let parent_subgraph_id = parent_resolver
            .as_ref()
            .map(|ParentResolver { subgraph_id, .. }| *subgraph_id);
        for resolver_definition in field_definition.resolvers() {
            // If within the same subgraph, we skip it. Resolvers are entrypoints.
            if Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                continue;
            };
            let resolver = FieldResolver::new(resolver_definition.id(), field_definition);
            let field_resolver_node = self.graph.add_node(Node::FieldResolver(resolver.clone()));

            if let Some(required_field_set) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id())
            {
                self.requirements_stack.push(Requirement {
                    parent_field_node,
                    petitioner_node: field_resolver_node,
                    required_field_set,
                })
            }

            let resolver_node = if let Some(edge) =
                self.graph
                    .edges_directed(parent_node, Direction::Outgoing)
                    .find(|edge| {
                        self.graph[edge.target()]
                            .as_resolver()
                            .is_some_and(|id| id == resolver_definition.id())
                    }) {
                edge.target()
            } else {
                let node = self.graph.add_node(Node::Resolver(resolver_definition.id()));
                // We don't know the real cost yet, but it's at least one as it'll need extra
                // work.
                self.graph.add_edge(parent_node, node, Edge::Resolver(1));
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.requirements_stack.push(Requirement {
                        parent_field_node,
                        petitioner_node: node,
                        required_field_set,
                    });
                };

                node
            };

            // We don't know the real cost here either, but without any requirements it's 0.
            self.graph
                .add_edge(resolver_node, field_resolver_node, Edge::CanResolveField(0));
            self.graph.add_edge(field_resolver_node, field_node, Edge::Resolves);

            for nested_field_id in self.operation.subselection(field_id) {
                self.fields_stack.push(Field {
                    parent_field_node: field_node,
                    parent_resolver: Some(ParentResolver {
                        field_resolver_node,
                        subgraph_id: resolver_definition.subgraph_id(),
                    }),
                    field_id: nested_field_id,
                })
            }
        }
    }

    fn handle_requirements(
        &mut self,
        Requirement {
            parent_field_node,
            petitioner_node,
            required_field_set,
        }: Requirement<'ctx>,
    ) {
        for item in required_field_set.items() {
            let existing_field = self
                .graph
                .edges_directed(parent_field_node, Direction::Outgoing)
                .filter_map(|edge| {
                    if matches!(edge.weight(), Edge::Field) {
                        self.graph[edge.target()]
                            .as_field()
                            .map(|field_id| (edge.target(), field_id))
                    } else {
                        None
                    }
                })
                .filter(|(_, field_id)| self.operation.field_satisfies(*field_id, item.field()))
                // not sure if necessary but provides consistency
                .min_by_key(|(_, field_id)| *field_id);

            let required_node = existing_field.map(|(node, _)| node).unwrap_or_else(|| {
                let field_id = self.operation.create_extra_field(item.field());
                let field_node = self.graph.add_node(Node::Field(field_id));
                self.graph.add_edge(parent_field_node, field_node, Edge::Field);
                self.fields_stack.extend(
                    self.graph
                        .edges_directed(parent_field_node, Direction::Incoming)
                        .filter_map(|edge| {
                            if matches!(edge.weight(), Edge::Resolves) {
                                let node = edge.source();
                                self.graph[node].as_field_resolver().map(|r| (node, r))
                            } else {
                                None
                            }
                        })
                        .map(|(field_resolver_node, field_resolver)| {
                            tracing::debug!("creating field");
                            Field {
                                parent_field_node,
                                parent_resolver: Some(ParentResolver {
                                    field_resolver_node,
                                    subgraph_id: field_resolver.resolver_definition_id.walk(self.schema).subgraph_id(),
                                }),
                                field_id,
                            }
                        }),
                );
                self.field_nodes.push(field_node);
                if matches!(item.field().definition().ty().definition(), Definition::Scalar(_)) {
                    self.scalar_nodes.push(field_node);
                }
                field_node
            });
            self.graph.add_edge(petitioner_node, required_node, Edge::Requires);

            if item.subselection().items().len() != 0 {
                self.requirements_stack.push(Requirement {
                    parent_field_node: required_node,
                    petitioner_node,
                    required_field_set: item.subselection(),
                })
            }
        }
    }

    fn prune_resolvers_not_providing_any_scalars(&mut self) {
        let mut touches_scalar = HashSet::new();
        let mut stack = self.scalar_nodes.clone();

        while let Some(node) = stack.pop() {
            if touches_scalar.contains(&node) {
                continue;
            };
            stack.extend(
                self.graph
                    .edges_directed(node, Direction::Incoming)
                    .filter(|edge| {
                        matches!(
                            edge.weight(),
                            Edge::Resolves | Edge::CanResolveField(_) | Edge::Resolver(_)
                        )
                    })
                    .map(|edge| edge.source()),
            );
            touches_scalar.insert(node);
        }

        let mut to_remove_stack = self
            .graph
            .node_indices()
            .filter(|node| matches!(self.graph[*node], Node::Resolver(_)) && !touches_scalar.contains(node))
            .collect::<Vec<_>>();

        for node in &to_remove_stack {
            let edges = self
                .graph
                .edges_directed(*node, Direction::Incoming)
                .map(|edge| edge.id())
                .collect::<Vec<_>>();
            for edge in edges {
                self.graph.remove_edge(edge);
            }
        }

        while let Some(node) = to_remove_stack.pop() {
            if self.graph.edges_directed(node, Direction::Incoming).next().is_none() {
                for neighbor in self.graph.neighbors_directed(node, Direction::Outgoing) {
                    to_remove_stack.push(neighbor);
                }
                self.graph.remove_node(node);
            }
        }
    }

    /// cost depends on how many different resolvers one depends on.
    /// No... if fields are all available at 0 zero cost, the plan only costs 1.
    /// so for each requirement update cost.
    /// maybe topological sort?
    /// graph cost:
    ///     resolver depends on fields, each one provided by one or multiple plans.
    ///     so max(field) and each of those is min(plan)
    /// During iteration will change resolver weight so need a quick resolver -> impacted resolvers
    fn estimate_cost(&mut self) {
        let mut updated_fields = self
            .graph
            .edge_references()
            .filter_map(|edge| match edge.weight() {
                Edge::Requires => Some(edge.target()),
                _ => None,
            })
            .collect::<Vec<_>>();
        while let Some(field) = updated_fields.pop() {}
        struct Case {
            resolution_edge: EdgeIndex,
            depedent_edges: Vec<EdgeIndex>,
            requirement_fields: Vec<NodeIndex>,
        }
        type Cost = u16;
        enum CostEdge {
            RequiredField,
            ResolvedBy,
        }
        let mut cost_graph = petgraph::csr::Csr::<Cost, CostEdge>::new();
        let mut to_cost = Vec::new();
        for (ix, node) in self.graph.node_references() {
            match node {
                Node::Root => continue,
                Node::Field(_) => {
                    if self
                        .graph
                        .edges_directed(ix, Direction::Incoming)
                        .any(|edge| matches!(edge.weight(), Edge::Requires))
                    {
                        to_cost.push((ix, cost_graph.add_node(0)))
                    }
                }
                Node::Resolver(_) | Node::FieldResolver(_) => {
                    to_cost.push((ix, cost_graph.add_node(1)));
                }
            }
        }

        // Likely unnecessary
        to_cost.sort_unstable();

        for &(graph_ix, cost_ix) in &to_cost {
            for edge in self.graph.edges_directed(graph_ix, Direction::Outgoing) {
                match edge.weight() {}
                if matches!(edge.weight(), Edge::Requires) {
                    let target = to_cost
                        .binary_search_by_key(edge.target(), |(ix, _)| ix)
                        .expect("Should have been added.");
                    cost_graph.add_edge(graph_ix, to_cost[graph_ix], CostEdge::RequiredField);
                }
            }
        }
    }

    /// Check out https://dreampuf.github.io/GraphvizOnline
    fn dot_graph(&self) -> String {
        let node_str = |_, node_ref: (NodeIndex, &Node<Op::FieldId>)| match node_ref.1 {
            Node::Root => r#"label = "root""#.to_string(),
            Node::Field(id) => format!("label = \"{}\"", self.operation.field_label(*id)),
            Node::FieldResolver(field_resolver) => format!(
                "label = \"{}@{}\",shape=box,style=dashed,color=blue",
                field_resolver.field_definition_id.walk(self.schema).name(),
                field_resolver.resolver_definition_id.walk(self.schema).name()
            ),
            Node::Resolver(resolver_definition_id) => {
                format!(
                    "label = \"{}\",shape=box,color=blue",
                    resolver_definition_id.walk(self.schema).name()
                )
            }
        };
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel],
                &|_, edge| {
                    match edge.weight() {
                        Edge::Resolver(_) | Edge::CanResolveField(_) | Edge::Resolves => "color=blue".to_string(),
                        Edge::Field | Edge::TypenameField => String::new(),
                        Edge::Requires => "color=green".to_string(),
                    }
                },
                &node_str
            )
        )
    }
}

fn remove_all<N, E>(graph: &mut petgraph::stable_graph::StableGraph<N, E>, node: NodeIndex) {
    let mut bfs = Bfs::new(&*graph, node);
    let mut nodes = vec![node];
    while let Some(nx) = bfs.next(&*graph) {
        nodes.push(nx)
    }
    for nx in nodes {
        graph.remove_node(nx);
    }
}
