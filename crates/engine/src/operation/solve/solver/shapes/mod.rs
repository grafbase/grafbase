mod partition;

use fixedbitset::FixedBitSet;
use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{CompositeType, CompositeTypeId, Definition, ObjectDefinitionId, Schema};
use walker::Walk;

use crate::{
    operation::{
        DataField, DataFieldId, FieldShapeRefId, ResponseObjectSetDefinitionId, SelectionSet, SolvedOperation,
        SolvedOperationContext, TypenameField,
    },
    response::{
        ConcreteShapeId, ConcreteShapeRecord, FieldShapeId, FieldShapeRecord, ObjectIdentifier, PolymorphicShapeId,
        PolymorphicShapeRecord, PositionedResponseKey, Shape, Shapes,
    },
    utils::BufferPool,
};

use super::Solver;

impl Solver<'_> {
    pub(super) fn populate_shapes_after_partition_generation(&mut self) {
        let mut query_partitions = std::mem::take(&mut self.operation.query_partitions);
        let mut builder = ShapesBuilder {
            schema: self.schema,
            operation: &self.operation,
            shapes: Shapes::default(),
            data_field_ids_with_selection_set_requiring_typename: Vec::new(),
            field_shapes_buffer_pool: BufferPool::default(),
            data_fields_buffer_pool: BufferPool::default(),
            typename_fields_buffer_pool: BufferPool::default(),
            data_fields_shape_count: vec![0; self.operation.data_fields.len()],
        };
        let ctx = SolvedOperationContext {
            schema: self.schema,
            operation: &self.operation,
        };

        // Create all shapes for the given QueryPartition
        for query_partition in &mut query_partitions {
            query_partition.shape_id = builder.create_root_shape_for(query_partition.selection_set_record.walk(ctx));
        }
        let ShapesBuilder {
            shapes,
            data_field_ids_with_selection_set_requiring_typename,
            data_fields_shape_count,
            ..
        } = builder;
        self.operation.query_partitions = query_partitions;

        // Keep track of all fields for which we need to include a __typename in the selection
        // set we send to the subgraph.
        for id in data_field_ids_with_selection_set_requiring_typename {
            self.operation[id].selection_set_requires_typename = true
        }

        // We keep track of all associated field shapes to a DataField to apply correctly
        // QueryModifierRules. To avoid an expensive sort, as we may generate *a lot* of shapes in
        // some edge cases, we rely on two things:
        // - field shapes needs the DataFieldId anyway
        // - we keep track of the field shape count associated with each data field.
        // So we assign a range to each data field in the field shape refs Vec and populate their
        // range with the right ids. Kind of a counting sort.
        let mut len: usize = 0;
        for (data_field, count) in self.operation.data_fields.iter_mut().zip(data_fields_shape_count) {
            data_field.shape_ids = IdRange::from(len..len);
            len += count as usize;
        }
        let mut field_shape_refs = vec![FieldShapeId::from(0usize); len];
        for (i, field_shape) in shapes.fields.iter().enumerate() {
            let end = &mut self.operation[field_shape.id].shape_ids.end;
            let pos = usize::from(*end);
            field_shape_refs[pos] = FieldShapeId::from(i);
            *end = FieldShapeRefId::from(pos + 1);
        }

        self.operation.shapes = shapes;
        self.operation.field_shape_refs = field_shape_refs;
    }
}

pub(super) struct ShapesBuilder<'ctx> {
    schema: &'ctx Schema,
    operation: &'ctx SolvedOperation,
    shapes: Shapes,
    data_fields_shape_count: Vec<u32>,
    data_field_ids_with_selection_set_requiring_typename: Vec<DataFieldId>,
    field_shapes_buffer_pool: BufferPool<FieldShapeRecord>,
    data_fields_buffer_pool: BufferPool<DataField<'ctx>>,
    typename_fields_buffer_pool: BufferPool<TypenameField<'ctx>>,
}

impl<'ctx> ShapesBuilder<'ctx> {
    fn create_root_shape_for(&mut self, selection_set: SelectionSet<'ctx>) -> ConcreteShapeId {
        let keys = &self.operation.response_keys;

        let data_fields_sorted_by_response_key_str_then_position = {
            let mut fields = self.data_fields_buffer_pool.pop();
            fields.extend(selection_set.data_fields());
            fields.sort_unstable_by(|left, right| {
                let l = left.key;
                let r = right.key;
                keys[l.response_key]
                    .cmp(&keys[r.response_key])
                    .then(l.query_position.cmp(&r.query_position))
            });
            fields
        };

        let typename_fields_sorted_by_response_key_str_then_position = {
            let mut fields = self.typename_fields_buffer_pool.pop();
            fields.extend(selection_set.typename_fields());
            fields.sort_unstable_by(|left, right| {
                let l = left.key;
                let r = right.key;
                keys[l.response_key]
                    .cmp(&keys[r.response_key])
                    .then(l.query_position.cmp(&r.query_position))
            });
            fields
        };

        let included_typename_then_data_fields = {
            let mut included = FixedBitSet::with_capacity(
                data_fields_sorted_by_response_key_str_then_position.len()
                    + typename_fields_sorted_by_response_key_str_then_position.len(),
            );
            included.toggle_range(..included.len());
            included
        };

        let shape_id = self.create_concrete_shape(
            ObjectIdentifier::Anonymous,
            None,
            &typename_fields_sorted_by_response_key_str_then_position,
            &data_fields_sorted_by_response_key_str_then_position,
            included_typename_then_data_fields,
        );
        self.data_fields_buffer_pool
            .push(data_fields_sorted_by_response_key_str_then_position);
        self.typename_fields_buffer_pool
            .push(typename_fields_sorted_by_response_key_str_then_position);

        shape_id
    }

    /// Create the expected shape with known expected fields, applying the GraphQL field collection
    /// logic.
    fn create_concrete_shape(
        &mut self,
        identifier: ObjectIdentifier,
        set_id: Option<ResponseObjectSetDefinitionId>,
        typename_fields_sorted_by_response_key_str_then_position: &[TypenameField<'ctx>],
        data_fields_sorted_by_response_key_str_then_position: &[DataField<'ctx>],
        included_typename_then_data_fields: FixedBitSet,
    ) -> ConcreteShapeId {
        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut distinct_typename_response_keys: Vec<PositionedResponseKey> = Vec::new();
        let mut included = included_typename_then_data_fields.into_ones();

        let mut all_expected_keys_equal_response_keys = true;
        while let Some(i) = included.next() {
            if let Some(field) = typename_fields_sorted_by_response_key_str_then_position.get(i) {
                if distinct_typename_response_keys
                    .last()
                    // fields aren't sorted by the response key but by the string value they point
                    // to. However, response keys are deduplicated so the equality also works here
                    // to ensure we only have distinct values.
                    .map_or(true, |key| key.response_key != field.key.response_key)
                {
                    distinct_typename_response_keys.push(field.key);
                }
            } else {
                // We've exhausted the typename fields, so we know we're in the data fields now.
                let offset = typename_fields_sorted_by_response_key_str_then_position.len();
                let mut first = data_fields_sorted_by_response_key_str_then_position[i - offset];
                self.data_fields_shape_count[usize::from(first.id)] += 1;

                // We'll group data fields together by their response key
                let mut group = self.data_fields_buffer_pool.pop();
                group.push(first);

                for i in included.by_ref() {
                    let field = data_fields_sorted_by_response_key_str_then_position[i - offset];
                    self.data_fields_shape_count[usize::from(field.id)] += 1;
                    if field.key.response_key == first.key.response_key {
                        group.push(field);
                    } else {
                        let field_shape = self.create_data_field_shape(&mut group, first);
                        all_expected_keys_equal_response_keys &= field_shape.expected_key == first.key.response_key;
                        field_shapes_buffer.push(field_shape);
                        first = field;
                        group.clear();
                        group.push(first);
                    }
                }

                let field_shape = self.create_data_field_shape(&mut group, first);
                all_expected_keys_equal_response_keys &= field_shape.expected_key == first.key.response_key;
                field_shapes_buffer.push(field_shape);

                self.data_fields_buffer_pool.push(group);
            }
        }

        debug_assert!(!field_shapes_buffer.is_empty() || !distinct_typename_response_keys.is_empty());
        let shape = ConcreteShapeRecord {
            set_id,
            identifier,
            typename_response_keys: distinct_typename_response_keys,
            field_shape_ids: {
                let start = self.shapes.fields.len();
                let keys = &self.operation.response_keys;
                // If the expected key matches the response key, we don't need to sort
                // again.
                if !all_expected_keys_equal_response_keys {
                    field_shapes_buffer.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
                }
                debug_assert!(field_shapes_buffer.is_sorted_by(|a, b| keys[a.expected_key] < keys[b.expected_key]));
                self.shapes.fields.append(&mut field_shapes_buffer);
                self.field_shapes_buffer_pool.push(field_shapes_buffer);
                IdRange::from(start..self.shapes.fields.len())
            },
        };

        self.push_concrete_shape(shape)
    }

    fn create_data_field_shape(&mut self, group: &mut [DataField<'ctx>], first: DataField<'ctx>) -> FieldShapeRecord {
        let ty = first.definition().ty();
        let shape = match ty.definition() {
            Definition::Scalar(scalar) => Shape::Scalar(scalar.ty),
            Definition::Enum(enm) => Shape::Enum(enm.id),
            Definition::Interface(interface) => self.create_field_composite_type_output_shape(group, interface.into()),
            Definition::Object(object) => self.create_field_composite_type_output_shape(group, object.into()),

            Definition::Union(union) => self.create_field_composite_type_output_shape(group, union.into()),
            Definition::InputObject(_) => unreachable!("Cannot be an output"),
        };

        let required_field_id = group.iter().find_map(|field| field.matching_requirement_id);

        FieldShapeRecord {
            expected_key: first.subgraph_key,
            key: first.key,
            id: first.id,
            required_field_id,
            shape,
            wrapping: ty.wrapping,
        }
    }

    fn create_field_composite_type_output_shape(
        &mut self,
        parent_fields: &[DataField<'ctx>],
        output: CompositeType<'ctx>,
    ) -> Shape {
        //
        // Preparation
        //
        let set_id = parent_fields.iter().find_map(|field| field.output_id);
        let output_possible_types = output.possible_types();

        let (
            data_fields_sorted_by_response_key_str_then_position,
            typename_fields_sorted_by_response_key_str_then_position,
        ) = {
            let mut data_fields = self.data_fields_buffer_pool.pop();
            let mut typename_fields = self.typename_fields_buffer_pool.pop();
            for parent_field in parent_fields {
                data_fields.extend(parent_field.selection_set().data_fields());
                typename_fields.extend(parent_field.selection_set().typename_fields());
            }
            let keys = &self.operation.response_keys;
            typename_fields.sort_unstable_by(|left, right| {
                let l = left.key;
                let r = right.key;
                keys[l.response_key]
                    .cmp(&keys[r.response_key])
                    .then(l.query_position.cmp(&r.query_position))
            });
            data_fields.sort_unstable_by(|left, right| {
                let l = left.key;
                let r = right.key;
                keys[l.response_key]
                    .cmp(&keys[r.response_key])
                    .then(l.query_position.cmp(&r.query_position))
            });
            (data_fields, typename_fields)
        };

        //
        // Partitioning algorithm
        //
        let partition::Partitioning {
            partition_object_count,
            partitions,
        } = self.compute_object_shape_partitions(
            output_possible_types,
            &typename_fields_sorted_by_response_key_str_then_position,
            &data_fields_sorted_by_response_key_str_then_position,
        );

        let requires_typename = parent_fields.iter().any(|field| field.selection_set_requires_typename);

        //
        // Creating the right shape from the partitioning
        //
        let shape = if partitions.is_empty() {
            // There is no partition so all fields are included all the time.
            let included_typename_then_data_fields = {
                let mut included = FixedBitSet::with_capacity(
                    typename_fields_sorted_by_response_key_str_then_position.len()
                        + data_fields_sorted_by_response_key_str_then_position.len(),
                );
                included.toggle_range(..included.len());
                included
            };

            // We may still need to know the type of the object if there is any __typename field.
            let identifier = if output_possible_types.len() == 1 {
                ObjectIdentifier::Known(output_possible_types[0])
            } else if set_id.is_some()
                || !typename_fields_sorted_by_response_key_str_then_position.is_empty()
                || requires_typename
            {
                // The output is part of a ResponseObjectSet or has __typename fields, so we need
                // to know its actual type. We ensure that __typename will be present in the
                // selection set we send to the subgraph and know how to read it.
                self.data_field_ids_with_selection_set_requiring_typename
                    .extend(parent_fields.iter().map(|field| field.id));
                match output {
                    CompositeType::Interface(interface) => ObjectIdentifier::InterfaceTypename(interface.id),
                    CompositeType::Union(union) => ObjectIdentifier::UnionTypename(union.id),
                    CompositeType::Object(object) => ObjectIdentifier::Known(object.id),
                }
            } else {
                // We don't know the object type nor do we care.
                ObjectIdentifier::Anonymous
            };

            Shape::Concrete(self.create_concrete_shape(
                identifier,
                set_id,
                &typename_fields_sorted_by_response_key_str_then_position,
                &data_fields_sorted_by_response_key_str_then_position,
                included_typename_then_data_fields,
            ))
        } else {
            // If even just one partition exists we *always* need to know the type as there are not
            // treated the same. We may request no fields at all for some objects. So like before
            // we ensure we'll request the __typename in the subgraph query.
            self.data_field_ids_with_selection_set_requiring_typename
                .extend(parent_fields.iter().map(|field| field.id));

            let mut possibilities = Vec::with_capacity(partition_object_count);
            let mut fallback = None;
            for partition in partitions {
                match partition {
                    partition::Partition::One { id, fields } => {
                        let shape_id = self.create_concrete_shape(
                            ObjectIdentifier::Anonymous,
                            set_id,
                            &typename_fields_sorted_by_response_key_str_then_position,
                            &data_fields_sorted_by_response_key_str_then_position,
                            fields,
                        );
                        possibilities.push((id, shape_id));
                    }
                    partition::Partition::Many { ids, fields } => {
                        let shape_id = self.create_concrete_shape(
                            ObjectIdentifier::Anonymous,
                            set_id,
                            &typename_fields_sorted_by_response_key_str_then_position,
                            &data_fields_sorted_by_response_key_str_then_position,
                            fields,
                        );
                        possibilities.extend(ids.into_iter().map(|id| (id, shape_id)));
                    }
                    partition::Partition::Remaining { fields } => {
                        let n = typename_fields_sorted_by_response_key_str_then_position.len();
                        let identifier = if set_id.is_some() || fields.contains_any_in_range(..n) || requires_typename {
                            match output {
                                CompositeType::Interface(interface) => {
                                    ObjectIdentifier::InterfaceTypename(interface.id)
                                }
                                CompositeType::Union(union) => ObjectIdentifier::UnionTypename(union.id),
                                CompositeType::Object(object) => ObjectIdentifier::Known(object.id),
                            }
                        } else {
                            ObjectIdentifier::Anonymous
                        };
                        fallback = Some(self.create_concrete_shape(
                            identifier,
                            set_id,
                            &typename_fields_sorted_by_response_key_str_then_position,
                            &data_fields_sorted_by_response_key_str_then_position,
                            fields,
                        ));
                    }
                }
            }
            Shape::Polymorphic(self.push_polymorphic_shape(PolymorphicShapeRecord {
                possibilities,
                fallback,
            }))
        };

        self.data_fields_buffer_pool
            .push(data_fields_sorted_by_response_key_str_then_position);
        self.typename_fields_buffer_pool
            .push(typename_fields_sorted_by_response_key_str_then_position);

        shape
    }

    /// Given this list of fields we generate a partitioning of the output possible types so that
    /// each partition includes objects with the same fields.
    ///
    /// Each partition has a list of object ids and a bitset of all included fields. With typename
    /// fields being first and then data fields. There may be one special "Remaining" partition
    /// which includes everything not present in all other partitions. This is mainly used to avoid
    /// copying the list of possible types for big interfaces like `Node`.
    fn compute_object_shape_partitions(
        &self,
        output_possible_types: &[ObjectDefinitionId],
        typename_fields: &[TypenameField<'ctx>],
        data_fields: &[DataField<'ctx>],
    ) -> partition::Partitioning<ObjectDefinitionId, FixedBitSet> {
        let mut type_condition_and_field_position_in_bitset =
            Vec::with_capacity(typename_fields.len() + data_fields.len());
        for (i, field) in typename_fields.iter().enumerate() {
            type_condition_and_field_position_in_bitset.push((field.type_condition_id, i));
        }
        let offset = typename_fields.len();
        for (i, field) in data_fields.iter().enumerate() {
            type_condition_and_field_position_in_bitset.push((field.definition().parent_entity_id.into(), offset + i));
        }
        type_condition_and_field_position_in_bitset.sort_unstable();

        let type_conditions = type_condition_and_field_position_in_bitset
            .iter()
            .chunk_by(|(ty, _)| ty)
            .into_iter()
            .map(|(ty, chunk)| {
                let possible_types = match ty {
                    CompositeTypeId::Interface(id) => self.schema[*id].possible_type_ids.as_slice(),
                    CompositeTypeId::Union(id) => self.schema[*id].possible_type_ids.as_slice(),
                    CompositeTypeId::Object(id) => std::array::from_ref(id),
                };
                let mut bitset = FixedBitSet::with_capacity(type_condition_and_field_position_in_bitset.len());
                for (_, i) in chunk {
                    bitset.put(*i);
                }
                (possible_types, bitset)
            })
            .collect();

        partition::partition_object_shapes(output_possible_types, type_conditions)
    }

    fn push_concrete_shape(&mut self, shape: ConcreteShapeRecord) -> ConcreteShapeId {
        let id = self.shapes.concrete.len().into();
        self.shapes.concrete.push(shape);
        id
    }

    fn push_polymorphic_shape(&mut self, mut shape: PolymorphicShapeRecord) -> PolymorphicShapeId {
        let id = self.shapes.polymorphic.len().into();
        shape.possibilities.sort_unstable_by_key(|(id, _)| *id);
        self.shapes.polymorphic.push(shape);
        id
    }
}
