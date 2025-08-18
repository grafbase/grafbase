use std::cmp::Ordering;

use fxhash::FxHashMap;
use id_newtypes::{BitSet, IdRange, IdToMany};
use query_solver::{
    Edge, Node, QueryFieldId,
    petgraph::{
        Direction,
        graph::NodeIndex,
        visit::{EdgeRef, NodeIndexable as _},
    },
};
use schema::{EntityDefinitionId, ResolverDefinitionId, ResolverDefinitionVariant, SubgraphId, TypeDefinition};
use walker::Walk;

use super::*;
use crate::utils::BufferPool;

impl<'a> Solver<'a> {
    pub(super) fn generate_query_partitions(&mut self) -> SolveResult<(NodeMap, ResponseObjectSetMap)> {
        let mut node_map = NodeMapBuilder {
            node_to_field: vec![None; self.solution.graph.node_bound()],
            query_partition_to_node: Vec::new(),
            query_field_to_data_field: Vec::new(),
        };
        let mut response_object_set_map = ResponseObjectSetMap {
            query_field_id_to_response_object_set: Default::default(),
        };
        Context {
            nested_fields_buffer_pool: BufferPool::default(),
            query_partitions_to_create_stack: Vec::new(),
            derived_entities_roots: Vec::new(),
            map: &mut node_map,
            response_object_set_map: &mut response_object_set_map,
            solver: self,
        }
        .generate_query_partitions()?;
        Ok((node_map.build(), response_object_set_map))
    }
}

struct NodeMapBuilder {
    node_to_field: Vec<Option<PartitionFieldId>>,
    query_field_to_data_field: Vec<(QueryFieldId, DataFieldId)>,
    query_partition_to_node: Vec<(QueryPartitionId, NodeIndex)>,
}

impl NodeMapBuilder {
    pub(super) fn build(self) -> NodeMap {
        NodeMap {
            node_to_field: self.node_to_field,
            query_partition_to_node: self.query_partition_to_node,
            query_field_to_data_field: IdToMany::from(self.query_field_to_data_field),
        }
    }
}

pub(super) struct NodeMap {
    pub node_to_field: Vec<Option<PartitionFieldId>>,
    pub query_partition_to_node: Vec<(QueryPartitionId, NodeIndex)>,
    pub query_field_to_data_field: IdToMany<QueryFieldId, DataFieldId>,
}

pub(super) struct ResponseObjectSetMap {
    pub query_field_id_to_response_object_set: FxHashMap<QueryFieldId, ResponseObjectSetId>,
}

impl ResponseObjectSetMap {
    pub(super) fn get_response_object_set(
        &mut self,
        solver: &mut Solver<'_>,
        source_ix: NodeIndex,
    ) -> ResponseObjectSetId {
        let Node::Field { id, .. } = solver.solution.graph[source_ix] else {
            unreachable!();
        };
        *self.query_field_id_to_response_object_set.entry(id).or_insert_with(|| {
            solver
                .output
                .query_plan
                .response_object_set_definitions
                .push(super::ResponseObjectSetMetadataRecord {
                    ty_id: solver.solution[id]
                        .definition_id
                        .and_then(|def| def.walk(solver.schema).ty().definition_id.as_composite_type())
                        .expect("Could not have a child resolver if it wasn't a composite type"),
                    query_partition_ids: Vec::new(),
                });
            ResponseObjectSetId::from(solver.output.query_plan.response_object_set_definitions.len() - 1)
        })
    }
}

struct Context<'s, 'a> {
    solver: &'s mut Solver<'a>,
    nested_fields_buffer_pool: BufferPool<NestedField>,
    query_partitions_to_create_stack: Vec<QueryPartitionToCreate>,
    derived_entities_roots: Vec<(DataFieldId, NodeIndex, Option<NodeIndex>)>,
    map: &'s mut NodeMapBuilder,
    response_object_set_map: &'s mut ResponseObjectSetMap,
}

struct QueryPartitionToCreate {
    input_id: ResponseObjectSetId,
    source_ix: NodeIndex,
    entity_definition_id: EntityDefinitionId,
    resolver_definition_id: ResolverDefinitionId,
}

enum NestedField {
    Data {
        record: DataFieldRecord,
        node_id: NodeIndex,
        query_field_id: QueryFieldId,
    },
    Typename {
        record: TypenameFieldRecord,
        node_id: NodeIndex,
    },
}

impl<'a> std::ops::Deref for Context<'_, 'a> {
    type Target = Solver<'a>;
    fn deref(&self) -> &Self::Target {
        self.solver
    }
}

impl std::ops::DerefMut for Context<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.solver
    }
}

impl Context<'_, '_> {
    pub(super) fn generate_query_partitions(&mut self) -> SolveResult<()> {
        let root_input_id = self.output.query_plan.root_response_object_set_id;

        for edge in self.solver.solution.graph.edges(self.solution.root_node_id) {
            if let Edge::QueryPartition = edge.weight()
                && let Node::QueryPartition {
                    entity_definition_id,
                    resolver_definition_id,
                } = self.solution.graph[edge.target()]
            {
                self.query_partitions_to_create_stack.push(QueryPartitionToCreate {
                    input_id: root_input_id,
                    source_ix: edge.target(),
                    entity_definition_id,
                    resolver_definition_id,
                });
            }
        }

        while let Some(partition_to_create) = self.query_partitions_to_create_stack.pop() {
            self.generate_query_partition(partition_to_create);
        }

        self.populate_derive_from()?;

        let mut response_data_fields = BitSet::with_capacity(self.output.query_plan.data_fields.len());
        for (i, field) in self.output.query_plan.data_fields.iter().enumerate() {
            // If not explicitly required in the query, we're added later on as a requirement if
            // necessary.
            // If derived, we never need to be part of the query partition as seen by the subgraph.
            if field.query_position.is_some() {
                response_data_fields.insert(i.into());
            }
        }
        self.output.query_plan.response_data_fields = response_data_fields;
        let mut response_typename_fields = BitSet::with_capacity(self.output.query_plan.typename_fields.len());
        for (i, field) in self.output.query_plan.typename_fields.iter().enumerate() {
            if field.query_position.is_some() {
                response_typename_fields.insert(i.into());
            }
        }
        self.output.query_plan.response_typename_fields = response_typename_fields;

        Ok(())
    }

    fn generate_query_partition(
        &mut self,
        QueryPartitionToCreate {
            input_id,
            source_ix,
            entity_definition_id,
            resolver_definition_id,
        }: QueryPartitionToCreate,
    ) {
        let query_partition_id = QueryPartitionId::from(self.output.query_plan.partitions.len());
        let subgraph_id = resolver_definition_id.walk(self.schema).subgraph_id();
        let (_, selection_set_record) = self.generate_selection_set(subgraph_id, query_partition_id, source_ix, true);

        let selection_set_record =
            if let ResolverDefinitionVariant::Lookup(resolver) = resolver_definition_id.walk(self.schema).variant() {
                let definition = resolver.field_definition();
                let lookup_field = LookupFieldRecord {
                    subgraph_key: self.output.operation.response_keys.get_or_intern(definition.name()),
                    location: self.output.query_plan[selection_set_record
                        .data_field_ids_ordered_by_parent_entity_then_key
                        .start]
                        .location,
                    argument_ids: {
                        let start = self.output.query_plan.field_arguments.len();
                        for injection in resolver.injections() {
                            self.output
                                .query_plan
                                .field_arguments
                                .push(PartitionFieldArgumentRecord {
                                    definition_id: injection.definition_id,
                                    value_record: PlanValueRecord::Injection(injection.value),
                                });
                        }
                        IdRange::from(start..self.output.query_plan.field_arguments.len())
                    },
                    definition_id: definition.id,
                    required_fields_record_by_supergraph: Default::default(),
                    shape_ids: IdRange::empty(),
                    output_id: None,
                    selection_set_record,
                    query_partition_id,
                };
                let lookup_field_id: LookupFieldId = self.output.query_plan.lookup_fields.len().into();
                for child_id in lookup_field
                    .selection_set_record
                    .data_field_ids_ordered_by_parent_entity_then_key
                {
                    self.output.query_plan[child_id].parent_field_id = Some(lookup_field_id.into());
                }
                // Confirm with authorization tests...
                self.map.node_to_field[source_ix.index()] = Some(PartitionFieldId::Lookup(lookup_field_id));

                self.output.query_plan.lookup_fields.push(lookup_field);
                PartitionSelectionSetRecord {
                    data_field_ids_ordered_by_parent_entity_then_key: IdRange::empty(),
                    typename_field_ids: IdRange::empty(),
                    lookup_field_ids: IdRange::from_single(lookup_field_id),
                }
            } else {
                selection_set_record
            };

        self.output.query_plan.partitions.push(QueryPartitionRecord {
            entity_definition_id,
            resolver_definition_id,
            selection_set_record,
            input_id,
            // Populated later
            required_fields_record: Default::default(),
            shape_id: RootFieldsShapeId::from(0usize),
        });
        self.map.query_partition_to_node.push((query_partition_id, source_ix));
    }

    fn generate_selection_set(
        &mut self,
        subgraph_id: SubgraphId,
        query_partition_id: QueryPartitionId,
        source_ix: NodeIndex,
        is_root_selection_set: bool,
    ) -> (Option<ResponseObjectSetId>, PartitionSelectionSetRecord) {
        let mut response_object_set_id: Option<ResponseObjectSetId> = None;
        let mut fields_buffer = self.nested_fields_buffer_pool.pop();

        let mut neighbors = self.solution.graph.neighbors(source_ix).detach();
        while let Some((edge_ix, target_id)) = neighbors.next(&self.solution.graph) {
            match self.solution.graph[edge_ix] {
                Edge::QueryPartition => {
                    let Node::QueryPartition {
                        entity_definition_id,
                        resolver_definition_id,
                    } = self.solution.graph[target_id]
                    else {
                        continue;
                    };
                    let new_partition = QueryPartitionToCreate {
                        input_id: *response_object_set_id.get_or_insert_with(|| {
                            let id = self
                                .response_object_set_map
                                .get_response_object_set(self.solver, source_ix);
                            self.output.query_plan[id].query_partition_ids.push(query_partition_id);
                            id
                        }),
                        source_ix: target_id,
                        resolver_definition_id,
                        entity_definition_id,
                    };
                    self.query_partitions_to_create_stack.push(new_partition);
                }
                Edge::Field => {
                    let Node::Field { id: query_field_id, .. } = self.solution.graph[target_id] else {
                        continue;
                    };
                    match self.crate_data_field_or_typename_field(query_partition_id, query_field_id) {
                        None => continue,
                        Some(PartitionFieldRecord::Data(mut record)) => {
                            if is_root_selection_set {
                                debug_assert_eq!(
                                    usize::from(record.argument_ids.end),
                                    self.output.query_plan.field_arguments.len()
                                );
                                if let Some(requires) = record
                                    .definition_id
                                    .walk(self.schema)
                                    .requires_for_subgraph(subgraph_id)
                                {
                                    for injection in requires.injections() {
                                        self.output
                                            .query_plan
                                            .field_arguments
                                            .push(PartitionFieldArgumentRecord {
                                                definition_id: injection.definition_id,
                                                value_record: PlanValueRecord::Injection(injection.value),
                                            });
                                    }
                                }
                                record.argument_ids.end = self.output.query_plan.field_arguments.len().into();
                            }
                            if record
                                .definition_id
                                .walk(self.schema)
                                .ty()
                                .definition()
                                .is_composite_type()
                            {
                                let (nested_response_object_set_id, selection_set) =
                                    self.generate_selection_set(subgraph_id, query_partition_id, target_id, false);
                                record.output_id = nested_response_object_set_id;
                                record.selection_set_record = selection_set;
                            }
                            fields_buffer.push(NestedField::Data {
                                record,
                                node_id: target_id,
                                query_field_id,
                            });
                        }
                        Some(PartitionFieldRecord::Typename(record)) => {
                            fields_buffer.push(NestedField::Typename {
                                record,
                                node_id: target_id,
                            });
                        }
                    }
                }
                Edge::RequiredBySubgraph | Edge::RequiredBySupergraph | Edge::MutationExecutedAfter | Edge::Derive => {}
            }
        }

        let data_fields_start = self.output.query_plan.data_fields.len();
        let typename_fields_start = self.output.query_plan.typename_fields.len();
        fields_buffer.sort_unstable_by(|left, right| match (left, right) {
            (NestedField::Data { record: left, .. }, NestedField::Data { record: right, .. }) => self.schema
                [left.definition_id]
                .parent_entity_id
                .cmp(&self.schema[right.definition_id].parent_entity_id)
                .then(left.key().cmp(&right.key())),
            // __typename fields don't matter
            (NestedField::Data { .. }, NestedField::Typename { .. }) => Ordering::Less,
            (NestedField::Typename { .. }, NestedField::Data { .. }) => Ordering::Greater,
            (NestedField::Typename { .. }, NestedField::Typename { .. }) => Ordering::Equal,
        });

        for field in fields_buffer.drain(..) {
            match field {
                NestedField::Data {
                    record,
                    node_id,
                    query_field_id,
                } => {
                    let field_id = self.output.query_plan.data_fields.len().into();
                    self.map.node_to_field[node_id.index()] = Some(PartitionFieldId::Data(field_id));
                    self.map.query_field_to_data_field.push((query_field_id, field_id));
                    for nested in &mut self.output.query_plan[record
                        .selection_set_record
                        .data_field_ids_ordered_by_parent_entity_then_key]
                    {
                        nested.parent_field_id = Some(field_id.into());
                    }
                    if record
                        .definition_id
                        .walk(self.schema)
                        .derives()
                        .any(|d| d.subgraph_id == subgraph_id)
                    {
                        self.derived_entities_roots.push((
                            field_id,
                            node_id,
                            self.solution
                                .graph
                                .edges_directed(node_id, Direction::Incoming)
                                .filter(|edge| matches!(edge.weight(), Edge::Derive))
                                .map(|edge| edge.source())
                                .next(),
                        ));
                    }
                    self.output.query_plan.data_fields.push(record);
                }
                NestedField::Typename { record, node_id } => {
                    self.map.node_to_field[node_id.index()] = Some(PartitionFieldId::Typename(
                        self.output.query_plan.typename_fields.len().into(),
                    ));
                    self.output.query_plan.typename_fields.push(record);
                }
            }
        }
        self.nested_fields_buffer_pool.push(fields_buffer);

        let selection_set = PartitionSelectionSetRecord {
            data_field_ids_ordered_by_parent_entity_then_key: IdRange::from(
                data_fields_start..self.output.query_plan.data_fields.len(),
            ),
            typename_field_ids: IdRange::from(typename_fields_start..self.output.query_plan.typename_fields.len()),
            lookup_field_ids: IdRange::empty(),
        };

        (response_object_set_id, selection_set)
    }

    fn populate_derive_from(&mut self) -> SolveResult<()> {
        for (root_id, node_ix, batch_node_ix) in std::mem::take(&mut self.derived_entities_roots) {
            let batch_field_id = batch_node_ix.map(|ix| self.map.node_to_field[ix.index()].unwrap().as_data().unwrap());
            self.output.query_plan[root_id].derive = Some(Derive::Root { batch_field_id });
            for nested_field_edge in self
                .solver
                .solution
                .graph
                .edges_directed(node_ix, Direction::Outgoing)
                .filter(|edge| matches!(edge.weight(), Edge::Field))
            {
                let Node::Field { id, .. } = self.solution.graph[nested_field_edge.target()] else {
                    unreachable!();
                };
                if self.solution[id].definition_id.is_none() {
                    continue;
                }
                let Some(PartitionFieldId::Data(from_id)) = self
                    .solution
                    .graph
                    .edges_directed(nested_field_edge.target(), Direction::Incoming)
                    .find(|edge| matches!(edge.weight(), Edge::Derive))
                    .and_then(|edge| self.map.node_to_field[edge.source().index()])
                else {
                    unreachable!("Derived field must have a derived edge");
                };
                let Some(PartitionFieldId::Data(field_id)) = self.map.node_to_field[nested_field_edge.target().index()]
                else {
                    unreachable!("Derived field must be data field");
                };
                self.solver.output.query_plan[field_id].derive = if batch_field_id == Some(from_id) {
                    Some(Derive::ScalarAsField)
                } else {
                    Some(Derive::From(from_id))
                };
            }
        }
        Ok(())
    }

    fn crate_data_field_or_typename_field(
        &mut self,
        query_partition_id: QueryPartitionId,
        id: QueryFieldId,
    ) -> Option<PartitionFieldRecord> {
        let schema = self.solver.schema;
        let output = &mut self.solver.output;
        let field = &self.solver.solution[id];

        let response_key = field.response_key?;
        if let Some(definition_id) = field.definition_id {
            let start = output.query_plan.field_arguments.len();
            match field.argument_ids {
                query_solver::QueryOrSchemaFieldArgumentIds::Query(ids) => {
                    output
                        .query_plan
                        .field_arguments
                        .extend(output.operation[ids].iter().map(|arg| PartitionFieldArgumentRecord {
                            definition_id: arg.definition_id,
                            value_record: PlanValueRecord::Value(arg.value_id.into()),
                        }))
                }
                query_solver::QueryOrSchemaFieldArgumentIds::Schema(ids) => {
                    output.query_plan.field_arguments.extend(schema[ids].iter().map(|arg| {
                        PartitionFieldArgumentRecord {
                            definition_id: arg.definition_id,
                            value_record: PlanValueRecord::Value(arg.value_id.into()),
                        }
                    }))
                }
            };
            let argument_ids = IdRange::from(start..output.query_plan.field_arguments.len());
            Some(PartitionFieldRecord::Data(DataFieldRecord {
                type_condition_ids: field.type_conditions,
                query_partition_id,
                definition_id,
                query_position: field.query_position,
                response_key,
                subgraph_key: field.subgraph_key,
                location: field.location,
                argument_ids,
                // All set later
                selection_set_record: PartitionSelectionSetRecord {
                    data_field_ids_ordered_by_parent_entity_then_key: IdRange::empty(),
                    typename_field_ids: IdRange::empty(),
                    lookup_field_ids: IdRange::empty(),
                },
                required_fields_record: RequiredFieldSetRecord::default(),
                required_fields_record_by_supergraph: Default::default(),
                output_id: None,
                parent_field_id: None,
                selection_set_requires_typename: match definition_id.walk(schema).ty().definition() {
                    // If we may encounter an inaccessible object, we have to detect it
                    TypeDefinition::Union(union) => union.has_inaccessible_member(),
                    TypeDefinition::Interface(interface) => interface.has_inaccessible_implementor(),
                    _ => false,
                },
                shape_ids_ref: IdRange::empty(),
                derive: None,
            }))
        } else {
            Some(PartitionFieldRecord::Typename(TypenameFieldRecord {
                type_condition_ids: field.type_conditions,
                response_key,
                query_position: field.query_position,
                location: field.location,
            }))
        }
    }
}

enum PartitionFieldRecord {
    Data(DataFieldRecord),
    Typename(TypenameFieldRecord),
}
