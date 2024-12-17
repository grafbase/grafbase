mod mutation_order;
mod partition_cycles;
mod response_key;
mod root_typename;

use petgraph::{visit::EdgeRef, Graph};
use schema::Schema;

use crate::{
    query::{Solution, SolutionEdge, SolutionNode},
    solution_space::{SpaceEdge, SpaceNode},
};
use crate::{Query, QueryFieldNode};

use super::SteinerTreeSolution;

pub(crate) struct SolvedQueryWithoutPostProcessing<'schema, 'op> {
    schema: &'schema Schema,
    operation: &'op mut Operation,
    query: SolvedQuery
}

impl<'schema> std::ops::Deref for SolvedQueryWithoutPostProcessing<'schema> {
    type Target = Solution<'schema>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SolvedQueryWithoutPostProcessing<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SolvedQuery {
    pub(crate) fn init<'schema, 'op>(
        schema: &'schema Schema,
        operation: &'op mut operation::Operation,
        mut query: Query<'schema>,
        solution: SteinerTreeSolution,
    ) -> crate::Result<SolvedQueryWithoutPostProcessing<'schema, 'op>> where 'schema: 'op {
        let n = operation.data_fields.len() + operation.typename_fields.len();
        let mut graph = Graph::with_capacity(n, n);
        let root_node_ix = graph.add_node(SolutionNode::Root);

        let mut stack = Vec::new();

        for edge in query.graph.edges(query.root_ix) {
            match edge.weight() {
                SpaceEdge::CreateChildResolver => {
                    stack.push((root_node_ix, edge.target()));
                }
                // For now assign __typename fields to the root node, they will be later be added
                // to an appropriate query partition.
                SpaceEdge::Field => {
                    if let SpaceNode::QueryField(QueryFieldNode { field_id, flags }) = query.graph[edge.target()] {
                        if query[field_id].definition_id.is_none() {
                            let typename_field_ix = graph.add_node(SolutionNode::Field { id: field_id, flags });
                            graph.add_edge(root_node_ix, typename_field_ix, SolutionEdge::Field);
                        }
                    }
                }
                _ => (),
            }
        }

        let mut nodes_with_dependencies = Vec::new();
        let mut edges_to_remove = Vec::new();
        let mut field_to_solution_node = vec![root_node_ix; n];
        while let Some((parent_solution_node_ix, node_ix)) = stack.pop() {
            let new_solution_node_ix = match &query.graph[node_ix] {
                SpaceNode::Resolver(resolver) if solution.node_bitset[node_ix.index()] => {
                    let ix = graph.add_node(SolutionNode::QueryPartition {
                        entity_definition_id: resolver.entity_definition_id,
                        resolver_definition_id: resolver.definition_id,
                    });
                    graph.add_edge(parent_solution_node_ix, ix, SolutionEdge::QueryPartition);
                    ix
                }
                SpaceNode::ProvidableField(_) if solution.node_bitset[node_ix.index()] => {
                    let (field_node_ix, field) = query
                        .graph
                        .edges(node_ix)
                        .find_map(|edge| {
                            if matches!(edge.weight(), SpaceEdge::Provides) {
                                if let SpaceNode::QueryField(field) = &query.graph[edge.target()] {
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
                        id: field.field_id,
                        flags: field.flags,
                    });
                    graph.add_edge(parent_solution_node_ix, field_solution_node_ix, SolutionEdge::Field);
                    field_to_solution_node[usize::from(field.field_id)] = field_solution_node_ix;

                    for edge in query.graph.edges(field_node_ix) {
                        match edge.weight() {
                            SpaceEdge::Requires => {
                                nodes_with_dependencies.push((field_solution_node_ix, field_node_ix));
                            }
                            // Assigning __typename fields to the first resolver that provides the
                            // parent field. There might be multiple with shared root fields.
                            SpaceEdge::Field => {
                                edges_to_remove.push(edge.id());
                                if let SpaceNode::QueryField(QueryFieldNode { field_id, flags }) =
                                    query.graph[edge.target()]
                                {
                                    if query[field_id].definition_id.is_none() {
                                        let typename_field_ix =
                                            graph.add_node(SolutionNode::Field { id: field_id, flags });
                                        graph.add_edge(field_solution_node_ix, typename_field_ix, SolutionEdge::Field);
                                    }
                                }
                            }
                            _ => (),
                        }
                    }

                    for edge in edges_to_remove.drain(..) {
                        query.graph.remove_edge(edge);
                    }

                    field_solution_node_ix
                }
                SpaceNode::QueryField(QueryFieldNode { field_id, flags })
                    if query[*field_id].definition_id.is_none() =>
                {
                    let typename_field_ix = graph.add_node(SolutionNode::Field {
                        id: *field_id,
                        flags: *flags,
                    });
                    graph.add_edge(parent_solution_node_ix, typename_field_ix, SolutionEdge::Field);
                    typename_field_ix
                }
                _ => continue,
            };

            if query
                .graph
                .edges(node_ix)
                .any(|edge| matches!(edge.weight(), SpaceEdge::Requires))
            {
                nodes_with_dependencies.push((new_solution_node_ix, node_ix));
            }

            stack.extend(
                query
                    .graph
                    .edges(node_ix)
                    .filter(|edge| {
                        matches!(
                            edge.weight(),
                            SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide | SpaceEdge::Field
                        )
                    })
                    .map(|edge| (new_solution_node_ix, edge.target())),
            );
        }

        for (new_solution_node_ix, node_ix) in nodes_with_dependencies {
            let weight = match &query.graph[node_ix] {
                SpaceNode::QueryField(_) => SolutionEdge::RequiredBySupergraph,
                _ => SolutionEdge::RequiredBySubgraph,
            };
            for edge in query.graph.edges(node_ix) {
                if !matches!(edge.weight(), SpaceEdge::Requires) {
                    continue;
                }
                let SpaceNode::QueryField(field) = &query.graph[edge.target()] else {
                    continue;
                };

                let dependency = field_to_solution_node[usize::from(field.field_id)];
                debug_assert_ne!(dependency, root_node_ix);

                graph.add_edge(new_solution_node_ix, dependency, weight);
            }
        }

        let solution = SolvedQueryWithoutPostProcessing(Self {
            schema,
            operation,
            root_node_ix,
            graph,
            fields: query.fields,
            shared_type_conditions: query.shared_type_conditions,
            shared_directives: query.shared_directives,
        });

        tracing::debug!("Partial solution:\n{}", solution.to_pretty_dot_graph());

        Ok(solution)
    }
}

impl<'schema> SolvedQueryWithoutPostProcessing<'schema> {
    pub(crate) fn finalize(mut self) -> Solution<'schema> {
        self.adjust_response_keys_to_avoid_collisions();
        if Some(self.operation.root_object_id) == self.schema.graph.root_operation_types_record.mutation_id {
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
}
