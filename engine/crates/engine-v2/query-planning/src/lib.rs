#![allow(unused)]
#![allow(unused_crate_dependencies)]

use grafbase_workspace_hack as _;
use schema::{
    FieldDefinition, FieldDefinitionId, RequiredFieldSet, RequiredFieldSetId, RequiredFieldSetRecord,
    ResolverDefinitionId, Schema, SubgraphId,
};
use walker::Walk;

use std::{borrow::Cow, convert::Infallible, num::NonZero, time::Instant};

use itertools::Itertools;
use petgraph::{
    adj::EdgeReference,
    data::Build,
    dot::{Config, Dot},
    graph::{EdgeIndex, NodeIndex},
    visit::{Bfs, EdgeRef},
    Directed, Direction,
};
use tracing::instrument;

// #[cfg(test)]
// mod tests;

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
    fn new(resolver_definition_id: ResolverDefinitionId, field: FieldDefinition<'_>) -> Self {
        todo!()
    }

    fn child(&self, field_definition: FieldDefinition<'_>) -> Option<FieldResolver> {
        todo!()
    }
}

impl<F> Node<F> {
    fn as_resolver(&self) -> Option<ResolverDefinitionId> {
        match self {
            Node::Resolver(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Edge {
    Resolver(Cost),
    CanResolve(Cost),
    Resolves,
    Field,
    TypenameField,
}

impl Edge {
    fn is_resolver(&self) -> bool {
        matches!(self, Self::Resolver(_))
    }
}

pub type Cost = u16;

pub trait Operation: std::fmt::Debug {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug;

    // Operation utilities
    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;

    // Dot graph utilities
    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str>;
}

pub struct Plan<'ctx, Op: Operation> {
    schema: &'ctx Schema,
    operation: &'ctx Op,
    graph: petgraph::stable_graph::StableGraph<Node<Op::FieldId>, Edge>,
    root: NodeIndex,
    field_nodes: Vec<NodeIndex>,
}

impl<'ctx, Op: Operation> std::ops::Index<Op::FieldId> for Plan<'ctx, Op> {
    type Output = NodeIndex;
    fn index(&self, field_id: Op::FieldId) -> &Self::Output {
        let ix: usize = field_id.into();
        &self.field_nodes[ix]
    }
}

impl<'ctx, Op: Operation> Plan<'ctx, Op> {
    pub fn build(schema: &'ctx Schema, operation: &'ctx Op) -> Self {
        let n = operation.field_ids().len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root = graph.add_node(Node::Root);

        let mut field_nodes = Vec::with_capacity(n);
        for field_id in operation.field_ids() {
            field_nodes.push(graph.add_node(Node::Field(field_id)));
        }

        let mut plan = Plan {
            schema,
            operation,
            field_nodes,
            root,
            graph,
        };

        debug_assert!(plan.ingest_selection_set(plan.root, None, operation.root_selection_set()));

        plan
    }

    fn ingest_selection_set(
        &mut self,
        parent_node: NodeIndex,
        parent_resolver: Option<(SubgraphId, FieldResolver)>,
        selection_set: impl ExactSizeIterator<Item = Op::FieldId>,
    ) -> bool {
        let mut could_resolve_at_least_one_field = false;
        let (parent_subgraph_id, parent_field_resolver) = parent_resolver.unzip();

        // Ingest all fields and their sub-selection first.
        for field_id in selection_set {
            let Some(definition_id) = self.operation.field_defintion(field_id) else {
                self.graph.add_edge(parent_node, self[field_id], Edge::TypenameField);
                continue;
            };
            self.graph.add_edge(parent_node, self[field_id], Edge::Field);

            let field_definition = definition_id.walk(self.schema);

            if let Some(resolver) = parent_field_resolver.as_ref().and_then(|pr| pr.child(field_definition)) {
                let field_resolver_node = self.graph.add_node(Node::FieldResolver(resolver));
                self.graph.add_edge(field_resolver_node, self[field_id], Edge::Resolves);
                self.graph
                    .add_edge(parent_node, field_resolver_node, Edge::CanResolve(0));
            }

            for resolver_definition in field_definition.resolvers() {
                // If within the same subgraph, we
                if Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                    continue;
                };
                let resolver = FieldResolver::new(resolver_definition.id(), field_definition);
                let field_resolver_node = self.graph.add_node(Node::FieldResolver(resolver.clone()));
                // If at least one field is accessible we connect it to the main graph, otherwise
                // we delete what we added.
                if !self.ingest_selection_set(
                    field_resolver_node,
                    Some((resolver_definition.subgraph_id(), resolver)),
                    self.operation.subselection(field_id),
                ) {
                    remove_all(&mut self.graph, field_resolver_node);
                    continue;
                }

                // Adding it to the root graph now that we know it can actually provide something,
                // supposing we can provide the necessary requirements.
                self.graph.add_edge(field_resolver_node, self[field_id], Edge::Resolves);

                let resolver_node = if let Some(edge) = self
                    .graph
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
                    node
                };
                // We don't know the real cost here either, but without any requirements it's 0.
                self.graph
                    .add_edge(resolver_node, field_resolver_node, Edge::CanResolve(0));
            }
        }

        enum HasRequirements<'a> {
            Resolver {
                incoming_edge: EdgeIndex,
                resolver: ResolverDefinitionId,
                required_field_set_id: RequiredFieldSetId,
            },
            FieldResolver {
                incoming_edge: EdgeIndex,
                field_resolver: FieldResolver,
                required_field_set: Cow<'a, RequiredFieldSetRecord>,
            },
        }

        let mut stack = Vec::new();

        // Deal with requirements.
        for edge in self.graph.edges_directed(parent_node, Direction::Outgoing) {
            let Node::Resolver(id) = self.graph[edge.target()] else {
                continue;
            };
            let resolver_definition = id.walk(self.schema);
            if let Some(required_field_set_id) = resolver_definition.required_field_set_id() {
                stack.push(HasRequirements::Resolver {
                    incoming_edge: edge.id(),
                    resolver: id,
                    required_field_set_id,
                });
            };

            for neighbor in self.graph.neighbors_directed(edge.target(), Direction::Outgoing) {
                let Node::FieldResolver(FieldResolver {
                    field_definition_id, ..
                }) = &self.graph[neighbor]
                else {
                    continue;
                };

                let field_definition = field_definition_id.walk(self.schema);
                if let Some(required_field_set_id) =
                    field_defintion.requires_for_subgraph(resolver_definition.subgraph_id())
                {}
            }
        }

        could_resolve_at_least_one_field
    }

    // fn solve(&mut self) {
    //     let start = Instant::now();
    //     let mut terminal_nodes = self.field_nodes.clone();
    //     terminal_nodes.push(self.root);
    //     let result = rustworkx_core::steiner_tree::steiner_tree(&self.graph, &terminal_nodes, |edge| {
    //         Result::<f64, Infallible>::Ok(match edge.weight() {
    //             Edge::Plan => 1f64,
    //             _ => 0f64,
    //         })
    //     })
    //     .expect("Weights without error")
    //     .expect("Not a disconnected graph");
    //
    //     self.graph = self.graph.filter_map(
    //         |ix, node| {
    //             if result.used_node_indices.contains(&ix.index()) {
    //                 Some(*node)
    //             } else {
    //                 None
    //             }
    //         },
    //         |ix, edge| {
    //             let e = &self.graph.raw_edges()[ix.index()];
    //             let a = e.source().index();
    //             let b = e.target().index();
    //             if result.used_edge_endpoints.contains(&(a, b)) || result.used_edge_endpoints.contains(&(b, a)) {
    //                 Some(*edge)
    //             } else {
    //                 None
    //             }
    //         },
    //     );
    //     println!("Solved in {:?}", start.elapsed());
    // }

    /// Check out https://dreampuf.github.io/GraphvizOnline
    pub fn dot_graph(&self) -> String {
        let node_str = |_, node_ref: (NodeIndex, &Node<Op::Resolver, Op::FieldId>)| match node_ref.1 {
            Node::Root => r#"label = "root""#.to_string(),
            Node::Field(id) => format!("label = \"{}\"", self.operation.field_label(*id)),
            Node::FieldResolver(id) => format!("label = \"@{}\"", self.operation.field_label(*id)),
            Node::Resolver(resolver) => format!("label = \"{}\"", self.operation.resolver_label(resolver)),
            // Node::ResolvedField(field_id) => {
            //     format!("label = \"{}\"", self.ctx.field_resolver_label(resolver, *field_id))
            // }
        };
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel],
                &|_, edge| { String::new() },
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
