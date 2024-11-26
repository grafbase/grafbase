use std::collections::VecDeque;

use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeFiltered;
use petgraph::{visit::EdgeRef, Graph};
use schema::{CompositeTypeId, ResolverDefinitionId, SubgraphId};
use walker::Walk;

use crate::{
    operation::{Edge, Node, OperationGraph},
    solution::{FieldFlags, Solution, SolutionEdge, SolutionNode},
    Operation,
};

use super::SteinerTreeSolution;

pub(crate) struct PartialSolution<'ctx, Op: Operation>(Solution<'ctx, Op>);

impl<'ctx, Op: Operation> std::ops::Deref for PartialSolution<'ctx, Op> {
    type Target = Solution<'ctx, Op>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'ctx, Op: Operation> std::ops::DerefMut for PartialSolution<'ctx, Op> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'ctx, Op: Operation> Solution<'ctx, Op> {
    pub(crate) fn build_partial(
        operation_graph: OperationGraph<'ctx, Op>,
        solution: SteinerTreeSolution,
    ) -> crate::Result<PartialSolution<'ctx, Op>> {
        let n = operation_graph.operation.field_ids().len();
        let mut graph = Graph::with_capacity(n, n);
        let root_node_ix = graph.add_node(SolutionNode::Root);

        let mut stack = Vec::new();

        for edge in operation_graph.graph.edges(operation_graph.root_ix) {
            match edge.weight() {
                Edge::CreateChildResolver => {
                    stack.push((root_node_ix, edge.target()));
                }
                // For now assign __typename fields to the root node, they will be later be added
                // to an appropriate query partition.
                Edge::TypenameField => {
                    if let Node::QueryField(field) = &operation_graph.graph[edge.target()] {
                        let typename_field_ix = graph.add_node(SolutionNode::Field {
                            id: field.id,
                            matching_requirement_id: field.matching_requirement_id,
                            flags: field.flags,
                        });
                        graph.add_edge(root_node_ix, typename_field_ix, SolutionEdge::Field);
                    }
                }
                _ => (),
            }
        }

        let OperationGraph {
            schema,
            operation,
            graph: mut operation_graph,
            ..
        } = operation_graph;

        let mut nodes_with_dependencies = Vec::new();
        let mut edges_to_remove = Vec::new();
        let mut field_to_solution_node = vec![root_node_ix; n];
        while let Some((parent_solution_node_ix, node_ix)) = stack.pop() {
            let new_solution_node_ix = match &operation_graph[node_ix] {
                Node::Resolver(resolver) if solution.node_bitset[node_ix.index()] => {
                    let ix = graph.add_node(SolutionNode::QueryPartition {
                        entity_definition_id: resolver.entity_definition_id,
                        resolver_definition_id: resolver.definition_id,
                    });
                    graph.add_edge(parent_solution_node_ix, ix, SolutionEdge::QueryPartition);
                    ix
                }
                Node::ProvidableField(_) if solution.node_bitset[node_ix.index()] => {
                    let (field_node_ix, field) = operation_graph
                        .edges(node_ix)
                        .find_map(|edge| {
                            if matches!(edge.weight(), Edge::Provides) {
                                if let Node::QueryField(field) = &operation_graph[edge.target()] {
                                    Some((edge.target(), field))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    let field_solution_node_ix = graph.add_node(SolutionNode::Field {
                        id: field.id,
                        matching_requirement_id: field.matching_requirement_id,
                        flags: field.flags,
                    });
                    graph.add_edge(parent_solution_node_ix, field_solution_node_ix, SolutionEdge::Field);
                    field_to_solution_node[field.id.into()] = field_solution_node_ix;

                    for edge in operation_graph.edges(field_node_ix) {
                        match edge.weight() {
                            Edge::Requires => {
                                nodes_with_dependencies.push((field_solution_node_ix, field_node_ix));
                            }
                            // Assigning __typename fields to the first resolver that provides the
                            // parent field
                            Edge::TypenameField => {
                                edges_to_remove.push(edge.id());
                                if let Node::QueryField(field) = &operation_graph[edge.target()] {
                                    let typename_field_ix = graph.add_node(SolutionNode::Field {
                                        id: field.id,
                                        matching_requirement_id: field.matching_requirement_id,
                                        flags: field.flags,
                                    });
                                    graph.add_edge(field_solution_node_ix, typename_field_ix, SolutionEdge::Field);
                                }
                            }
                            _ => (),
                        }
                    }

                    for edge in edges_to_remove.drain(..) {
                        operation_graph.remove_edge(edge);
                    }

                    field_solution_node_ix
                }
                Node::QueryField(field) if field.is_typename() => {
                    let ix = graph.add_node(SolutionNode::Field {
                        id: field.id,
                        matching_requirement_id: field.matching_requirement_id,
                        flags: field.flags,
                    });
                    graph.add_edge(parent_solution_node_ix, ix, SolutionEdge::Field);
                    ix
                }
                _ => continue,
            };

            if operation_graph
                .edges(node_ix)
                .any(|edge| matches!(edge.weight(), Edge::Requires))
            {
                nodes_with_dependencies.push((new_solution_node_ix, node_ix));
            }

            stack.extend(
                operation_graph
                    .edges(node_ix)
                    .filter(|edge| {
                        matches!(
                            edge.weight(),
                            Edge::CreateChildResolver | Edge::CanProvide | Edge::Field | Edge::TypenameField
                        )
                    })
                    .map(|edge| (new_solution_node_ix, edge.target())),
            );
        }

        for (new_solution_node_ix, node_ix) in nodes_with_dependencies {
            let weight = match &operation_graph[node_ix] {
                Node::QueryField(_) => SolutionEdge::RequiredBySupergraph,
                _ => SolutionEdge::RequiredBySubgraph,
            };
            for edge in operation_graph.edges(node_ix) {
                if !matches!(edge.weight(), Edge::Requires) {
                    continue;
                }
                let Node::QueryField(field) = &operation_graph[edge.target()] else {
                    continue;
                };

                let dependency = field_to_solution_node[field.id.into()];
                debug_assert_ne!(dependency, root_node_ix);

                graph.add_edge(new_solution_node_ix, dependency, weight);
            }
        }

        let solution = PartialSolution(Self {
            schema,
            operation,
            root_node_ix,
            graph,
        });

        tracing::debug!("Partial solution:\n{}", solution.to_pretty_dot_graph());

        Ok(solution)
    }
}

impl<'ctx, Op: Operation> PartialSolution<'ctx, Op> {
    pub(crate) fn finalize(mut self) -> Solution<'ctx, Op> {
        self.finalize_extra_fields();
        if Some(self.operation.root_object_id()) == self.schema.graph.root_operation_types_record.mutation_id {
            let root_fields = self.ensure_mutation_execution_order();
            // We already handled query partitions in a more specific way, so we don't want this
            // function to touch them. So it starts from the root field's selection sets instead of
            // the root selection set.
            self.split_query_partition_dependency_cycles(root_fields);
        } else {
            self.split_query_partition_dependency_cycles(vec![self.root_node_ix]);
        }
        self.assign_root_typename_fields();

        tracing::debug!("Solution:\n{}", self.to_pretty_dot_graph());

        self.0
    }

    fn assign_root_typename_fields(&mut self) {
        // There is always at least one field in the query, otherwise validation would fail. So
        // either there is an existing partition or there is only __typename fields and we have to
        // create one.
        let first_partition_ix = self
            .graph
            .neighbors(self.root_node_ix)
            .filter(|neighor| matches!(self.graph[*neighor], SolutionNode::QueryPartition { .. }))
            .min_by_key(|partition_node_ix| {
                self.graph
                    .neighbors(*partition_node_ix)
                    .filter_map(|neighbor| match self.graph[neighbor] {
                        SolutionNode::Field { id, .. } => Some(self.operation.field_query_position(id)),
                        _ => None,
                    })
                    .min()
                    .unwrap_or(usize::MAX)
            })
            .unwrap_or_else(|| {
                let ix = self.0.graph.add_node(SolutionNode::QueryPartition {
                    entity_definition_id: self.operation.root_object_id().into(),
                    resolver_definition_id: self.schema.subgraphs.introspection.resolver_definition_id,
                });
                self.0
                    .graph
                    .add_edge(self.0.root_node_ix, ix, SolutionEdge::QueryPartition);
                ix
            });
        let typename_fields = self
            .graph
            .edges(self.root_node_ix)
            .filter_map(|edge| match edge.weight() {
                SolutionEdge::Field => match self.graph[edge.target()] {
                    SolutionNode::Field { flags, .. } if flags.contains(FieldFlags::TYPENAME) => Some(edge.target()),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        for ix in typename_fields {
            if let Some(id) = self.graph.find_edge(self.root_node_ix, ix) {
                self.graph.remove_edge(id);
            }
            self.graph.add_edge(first_partition_ix, ix, SolutionEdge::Field);
        }
    }

    fn ensure_mutation_execution_order(&mut self) -> Vec<NodeIndex> {
        struct Field {
            position: usize,
            original_partition_node_ix: NodeIndex,
            resolver_definition_id: ResolverDefinitionId,
            field_node_ix: NodeIndex,
        }

        let mut selection_set = Vec::new();
        for partition_node_ix in self.graph.neighbors(self.root_node_ix) {
            if let SolutionNode::QueryPartition {
                resolver_definition_id, ..
            } = self.graph[partition_node_ix]
            {
                for field_node_ix in self.graph.neighbors(partition_node_ix) {
                    if let SolutionNode::Field { id, .. } = self.graph[field_node_ix] {
                        selection_set.push(Field {
                            position: self.operation.field_query_position(id),
                            original_partition_node_ix: partition_node_ix,
                            resolver_definition_id,
                            field_node_ix,
                        });
                    }
                }
            }
        }

        selection_set.sort_unstable_by(|a, b| a.position.cmp(&b.position));
        let selection_set = VecDeque::from(selection_set);

        let mut partitions = Vec::new();
        let mut root_fields = Vec::with_capacity(selection_set.len());

        for Field {
            original_partition_node_ix,
            resolver_definition_id,
            field_node_ix,
            ..
        } in selection_set
        {
            if let Some((last_partition_node_ix, _)) = partitions
                .last()
                .filter(|(_, last_resolver_definition_id)| *last_resolver_definition_id == resolver_definition_id)
            {
                if original_partition_node_ix == *last_partition_node_ix {
                    continue;
                } else {
                    if let Some(id) = self.graph.find_edge(original_partition_node_ix, field_node_ix) {
                        self.graph.remove_edge(id);
                    }
                    self.graph
                        .add_edge(*last_partition_node_ix, field_node_ix, SolutionEdge::Field);
                }
            }

            // If original partition is already used, create a new one.
            if partitions.iter().any(|(id, _)| *id == original_partition_node_ix) {
                let weight = self.graph[original_partition_node_ix];
                let new_partition_ix = self.graph.add_node(weight);
                self.0
                    .graph
                    .add_edge(self.0.root_node_ix, new_partition_ix, SolutionEdge::QueryPartition);
                partitions.push((new_partition_ix, resolver_definition_id));

                if let Some(id) = self.graph.find_edge(original_partition_node_ix, field_node_ix) {
                    self.graph.remove_edge(id);
                }
                self.graph
                    .add_edge(new_partition_ix, field_node_ix, SolutionEdge::Field);
            } else {
                partitions.push((original_partition_node_ix, resolver_definition_id));
            }

            root_fields.push(field_node_ix);
        }

        for ((partition1_ix, _), (partition2_ix, _)) in partitions.into_iter().tuple_windows() {
            self.graph
                .add_edge(partition2_ix, partition1_ix, SolutionEdge::MutationExecutedAfter);
        }

        root_fields
    }

    fn split_query_partition_dependency_cycles(&mut self, starting_nodes: Vec<NodeIndex>) {
        struct Field {
            position: usize,
            original_partition_node_ix: NodeIndex,
            resolver_definition_id: ResolverDefinitionId,
            field_node_ix: NodeIndex,
        }
        let mut partition_fields = Vec::new();
        let mut stack = starting_nodes;
        let mut partitions = Vec::new();

        while let Some(root_node_ix) = stack.pop() {
            partitions.clear();
            debug_assert!(partition_fields.is_empty());
            for edge in self.graph.edges(root_node_ix) {
                if !matches!(edge.weight(), SolutionEdge::Field | SolutionEdge::QueryPartition) {
                    continue;
                }
                match self.graph[edge.target()] {
                    SolutionNode::QueryPartition {
                        resolver_definition_id, ..
                    } => {
                        partitions.push((edge.target(), resolver_definition_id));
                        for second_degree_edge in self.graph.edges(edge.target()) {
                            if !matches!(
                                second_degree_edge.weight(),
                                SolutionEdge::Field | SolutionEdge::QueryPartition
                            ) {
                                continue;
                            }
                            let node_ix = second_degree_edge.target();
                            if let SolutionNode::Field { id, flags, .. } = self.graph[node_ix] {
                                partition_fields.push(Field {
                                    position: self.operation.field_query_position(id),
                                    original_partition_node_ix: edge.target(),
                                    resolver_definition_id,
                                    field_node_ix: node_ix,
                                });
                                if flags.contains(FieldFlags::IS_COMPOSITE_TYPE) {
                                    stack.push(node_ix);
                                }
                            }
                        }
                    }
                    SolutionNode::Field { id, .. } => {
                        if self
                            .operation
                            .field_definition(id)
                            .is_some_and(|def| def.walk(self.schema).ty().definition_id.is_composite_type())
                        {
                            stack.push(edge.target());
                        }
                    }
                    _ => (),
                }
            }

            partition_fields.sort_unstable_by(|a, b| a.position.cmp(&b.position));

            // Removing edges to the parent partitions
            for field in &partition_fields {
                if let Some(id) = self
                    .graph
                    .find_edge(field.original_partition_node_ix, field.field_node_ix)
                {
                    self.graph.remove_edge(id);
                }
            }

            for Field {
                original_partition_node_ix,
                resolver_definition_id,
                field_node_ix,
                ..
            } in partition_fields.drain(..)
            {
                let partition_node_ix = partitions
                    .iter()
                    .filter(|(_, id)| *id == resolver_definition_id)
                    .filter_map(|(partition_node_ix, _)| {
                        let is_connected = self
                            .graph
                            .edges(*partition_node_ix)
                            .filter_map(|edge| {
                                if matches!(edge.weight(), SolutionEdge::Field) {
                                    Some(edge.target())
                                } else {
                                    None
                                }
                            })
                            .any(|partition_field_node_ix| {
                                petgraph::algo::has_path_connecting(
                                    &EdgeFiltered::from_fn(&self.graph, |edge| {
                                        matches!(edge.weight(), SolutionEdge::RequiredBySubgraph)
                                    }),
                                    partition_field_node_ix,
                                    field_node_ix,
                                    None,
                                )
                            });
                        if is_connected {
                            None
                        } else {
                            Some(*partition_node_ix)
                        }
                    })
                    .next()
                    .unwrap_or_else(|| {
                        let weight = self.graph[original_partition_node_ix];
                        let new_partition_ix = self.graph.add_node(weight);
                        self.graph
                            .add_edge(root_node_ix, new_partition_ix, SolutionEdge::QueryPartition);

                        let mut neighbors = self.graph.neighbors(original_partition_node_ix).detach();
                        while let Some((edge_ix, node_ix)) = neighbors.next(&self.graph) {
                            let weight = self.graph[edge_ix];
                            if matches!(
                                weight,
                                SolutionEdge::RequiredBySubgraph | SolutionEdge::MutationExecutedAfter
                            ) {
                                self.graph.add_edge(new_partition_ix, node_ix, weight);
                            }
                        }

                        partitions.push((new_partition_ix, resolver_definition_id));
                        new_partition_ix
                    });

                self.graph
                    .add_edge(partition_node_ix, field_node_ix, SolutionEdge::Field);
            }
        }
    }

    fn finalize_extra_fields(&mut self) {
        let mut existing_fields = Vec::new();
        let mut extra_fields = Vec::new();
        let mut stack = vec![(
            CompositeTypeId::from(self.operation.root_object_id()),
            SubgraphId::Introspection,
            self.root_node_ix,
        )];
        while let Some((parent_type, subgraph_id, node)) = stack.pop() {
            extra_fields.clear();
            existing_fields.clear();
            for edge in self.graph.edges(node) {
                if !matches!(edge.weight(), SolutionEdge::Field | SolutionEdge::QueryPartition) {
                    continue;
                }
                match self.graph[edge.target()] {
                    SolutionNode::Field { id, flags, .. } => {
                        if flags.contains(FieldFlags::EXTRA) {
                            extra_fields.push((subgraph_id, id));
                        } else {
                            existing_fields.push((subgraph_id, id));
                        }
                        if let Some(parent_type) = self
                            .operation
                            .field_definition(id)
                            .walk(self.schema)
                            .and_then(|def| def.ty().definition_id.as_composite_type())
                        {
                            stack.push((parent_type, subgraph_id, edge.target()));
                        }
                    }
                    SolutionNode::QueryPartition {
                        resolver_definition_id, ..
                    } => {
                        let subgraph_id = resolver_definition_id.walk(self.schema).subgraph_id();
                        for second_degree_edge in self.graph.edges(edge.target()) {
                            if !matches!(
                                second_degree_edge.weight(),
                                SolutionEdge::Field | SolutionEdge::QueryPartition
                            ) {
                                continue;
                            }
                            if let SolutionNode::Field { id, flags, .. } = self.graph[second_degree_edge.target()] {
                                if flags.contains(FieldFlags::EXTRA) {
                                    extra_fields.push((subgraph_id, id));
                                } else {
                                    existing_fields.push((subgraph_id, id));
                                }
                                if let Some(parent_type) = self
                                    .operation
                                    .field_definition(id)
                                    .walk(self.schema)
                                    .and_then(|def| def.ty().definition_id.as_composite_type())
                                {
                                    stack.push((parent_type, subgraph_id, second_degree_edge.target()));
                                }
                            }
                        }
                    }
                    SolutionNode::Root => (),
                }
            }
            self.operation
                .finalize_selection_set(parent_type, &mut extra_fields, &mut existing_fields);
        }
    }
}
