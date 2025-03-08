mod partition;

use std::borrow::Cow;

use fixedbitset::FixedBitSet;
use id_newtypes::IdRange;
use im::HashSet;
use itertools::Itertools;
use operation::{PositionedResponseKey, ResponseKey};
use schema::{CompositeType, CompositeTypeId, Definition, ObjectDefinitionId, Schema};
use walker::Walk;

use crate::{
    prepare::cached::{
        CachedOperationContext, ConcreteShapeId, ConcreteShapeRecord, FieldShapeId, FieldShapeRecord, FieldShapeRefId,
        ObjectIdentifier, PartitionDataField, PartitionDataFieldId, PartitionSelectionSet, PartitionTypenameField,
        PolymorphicShapeId, PolymorphicShapeRecord, ResponseObjectSetDefinitionId, Shape, Shapes,
    },
    utils::BufferPool,
};

use super::Solver;

impl Solver<'_> {
    pub(super) fn populate_shapes_after_partition_generation(&mut self) {
        let mut query_partitions = std::mem::take(&mut self.output.query_plan.partitions);
        let ctx = CachedOperationContext {
            schema: self.schema,
            cached: &self.output,
        };
        let mut builder = ShapesBuilder {
            ctx,
            shapes: Shapes::default(),
            data_field_ids_with_selection_set_requiring_typename: Vec::new(),
            field_shapes_buffer_pool: BufferPool::default(),
            data_fields_buffer_pool: BufferPool::default(),
            typename_fields_buffer_pool: BufferPool::default(),
            data_fields_shape_count: vec![0; self.output.query_plan.data_fields.len()],
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
        self.output.query_plan.partitions = query_partitions;

        // Keep track of all fields for which we need to include a __typename in the selection
        // set we send to the subgraph.
        for id in data_field_ids_with_selection_set_requiring_typename {
            self.output.query_plan[id].selection_set_requires_typename = true
        }

        // We keep track of all associated field shapes to a DataField to apply correctly
        // QueryModifierRules. To avoid an expensive sort, as we may generate *a lot* of shapes in
        // some edge cases, we rely on two things:
        // - field shapes needs the DataFieldId anyway
        // - we keep track of the field shape count associated with each data field.
        // So we assign a range to each data field in the field shape refs Vec and populate their
        // range with the right ids. Kind of a counting sort.
        let mut len: usize = 0;
        for (data_field, count) in self
            .output
            .query_plan
            .data_fields
            .iter_mut()
            .zip(data_fields_shape_count)
        {
            data_field.shape_ids = IdRange::from(len..len);
            len += count as usize;
        }
        let mut field_shape_refs = vec![FieldShapeId::from(0usize); len];
        for (i, field_shape) in shapes.fields.iter().enumerate() {
            let end = &mut self.output.query_plan[field_shape.id].shape_ids.end;
            let pos = usize::from(*end);
            field_shape_refs[pos] = FieldShapeId::from(i);
            *end = FieldShapeRefId::from(pos + 1);
        }

        self.output.shapes = shapes;
        self.output.query_plan.field_shape_refs = field_shape_refs;
    }
}

pub(super) struct ShapesBuilder<'ctx> {
    ctx: CachedOperationContext<'ctx>,
    shapes: Shapes,
    data_fields_shape_count: Vec<u32>,
    data_field_ids_with_selection_set_requiring_typename: Vec<PartitionDataFieldId>,
    field_shapes_buffer_pool: BufferPool<FieldShapeRecord>,
    data_fields_buffer_pool: BufferPool<PartitionDataField<'ctx>>,
    typename_fields_buffer_pool: BufferPool<PartitionTypenameField<'ctx>>,
}

impl<'ctx> ShapesBuilder<'ctx> {
    fn create_root_shape_for(&mut self, selection_set: PartitionSelectionSet<'ctx>) -> ConcreteShapeId {
        let keys = &self.ctx.cached.operation.response_keys;

        let data_fields_sorted_by_response_key_str_then_position_extra_last = {
            let mut fields = self.data_fields_buffer_pool.pop();
            fields.extend(selection_set.data_fields());
            fields.sort_unstable_by(|left, right| {
                keys[left.response_key].cmp(&keys[right.response_key]).then(
                    left.query_position
                        .map(u16::from)
                        .unwrap_or(u16::MAX)
                        .cmp(&right.query_position.map(u16::from).unwrap_or(u16::MAX)),
                )
            });
            fields
        };

        let typename_fields_sorted_by_response_key_str_then_position_extra_last = {
            let mut fields = self.typename_fields_buffer_pool.pop();
            fields.extend(selection_set.typename_fields());
            fields.sort_unstable_by(|left, right| {
                keys[left.response_key].cmp(&keys[right.response_key]).then(
                    left.query_position
                        .map(u16::from)
                        .unwrap_or(u16::MAX)
                        .cmp(&right.query_position.map(u16::from).unwrap_or(u16::MAX)),
                )
            });
            fields
        };

        let included_typename_then_data_fields = {
            let mut included = FixedBitSet::with_capacity(
                data_fields_sorted_by_response_key_str_then_position_extra_last.len()
                    + typename_fields_sorted_by_response_key_str_then_position_extra_last.len(),
            );
            included.toggle_range(..included.len());
            included
        };

        let shape_id = self.create_concrete_shape(
            ObjectIdentifier::Anonymous,
            None,
            &typename_fields_sorted_by_response_key_str_then_position_extra_last,
            &data_fields_sorted_by_response_key_str_then_position_extra_last,
            included_typename_then_data_fields,
        );
        self.data_fields_buffer_pool
            .push(data_fields_sorted_by_response_key_str_then_position_extra_last);
        self.typename_fields_buffer_pool
            .push(typename_fields_sorted_by_response_key_str_then_position_extra_last);

        shape_id
    }

    /// Create the expected shape with known expected fields, applying the GraphQL field collection
    /// logic.
    fn create_concrete_shape(
        &mut self,
        identifier: ObjectIdentifier,
        set_id: Option<ResponseObjectSetDefinitionId>,
        typename_fields_sorted_by_response_key_str_then_position_extra_last: &[PartitionTypenameField<'ctx>],
        data_fields_sorted_by_response_key_str_then_position_extra_last: &[PartitionDataField<'ctx>],
        included_typename_then_data_fields: FixedBitSet,
    ) -> ConcreteShapeId {
        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut distinct_typename_response_keys: Vec<PositionedResponseKey> = Vec::new();
        let mut included = included_typename_then_data_fields.into_ones();

        while let Some(i) = included.next() {
            if let Some(field) = typename_fields_sorted_by_response_key_str_then_position_extra_last.get(i) {
                if distinct_typename_response_keys
                    .last()
                    // fields aren't sorted by the response key but by the string value they point
                    // to. However, response keys are deduplicated so the equality also works here
                    // to ensure we only have distinct values.
                    .is_none_or(|key| key.response_key != field.response_key)
                {
                    distinct_typename_response_keys.push(field.response_key.with_position(field.query_position));
                }
            } else {
                // We've exhausted the typename fields, so we know we're in the data fields now.
                let offset = typename_fields_sorted_by_response_key_str_then_position_extra_last.len();
                let mut first = data_fields_sorted_by_response_key_str_then_position_extra_last[i - offset];
                self.data_fields_shape_count[usize::from(first.id)] += 1;

                // We'll group data fields together by their response key
                let mut group = self.data_fields_buffer_pool.pop();
                group.push(first);

                for i in included.by_ref() {
                    let field = data_fields_sorted_by_response_key_str_then_position_extra_last[i - offset];
                    self.data_fields_shape_count[usize::from(field.id)] += 1;
                    if field.response_key == first.response_key {
                        group.push(field);
                    } else {
                        let field_shape = self.create_data_field_shape(&mut group, first);
                        field_shapes_buffer.push(field_shape);
                        first = field;
                        group.clear();
                        group.push(first);
                    }
                }

                let field_shape = self.create_data_field_shape(&mut group, first);
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
                field_shapes_buffer.sort_unstable_by(|a, b| a.id.cmp(&b.id));
                debug_assert!(
                    field_shapes_buffer
                        .iter()
                        .map(|f| f.expected_key)
                        .collect::<HashSet<ResponseKey>>()
                        .len()
                        == field_shapes_buffer.len()
                );
                self.shapes.fields.append(&mut field_shapes_buffer);
                self.field_shapes_buffer_pool.push(field_shapes_buffer);
                IdRange::from(start..self.shapes.fields.len())
            },
        };

        self.push_concrete_shape(shape)
    }

    fn create_data_field_shape(
        &mut self,
        group: &mut [PartitionDataField<'ctx>],
        first: PartitionDataField<'ctx>,
    ) -> FieldShapeRecord {
        let ty = first.definition().ty();
        let shape = match ty.definition() {
            Definition::Scalar(scalar) => Shape::Scalar(scalar.ty),
            Definition::Enum(enm) => Shape::Enum(enm.id),
            Definition::Interface(interface) => self.create_field_composite_type_output_shape(group, interface.into()),
            Definition::Object(object) => self.create_field_composite_type_output_shape(group, object.into()),

            Definition::Union(union) => self.create_field_composite_type_output_shape(group, union.into()),
            Definition::InputObject(_) => unreachable!("Cannot be an output"),
        };

        FieldShapeRecord {
            expected_key: first.subgraph_key.unwrap_or(first.response_key),
            key: first.response_key.with_position(first.query_position),
            id: first.id,
            shape,
            wrapping: ty.wrapping,
        }
    }

    fn create_field_composite_type_output_shape(
        &mut self,
        parent_fields: &[PartitionDataField<'ctx>],
        output: CompositeType<'ctx>,
    ) -> Shape {
        //
        // Preparation
        //
        let set_id = parent_fields.iter().find_map(|field| field.output_id);

        let (
            data_fields_sorted_by_response_key_str_then_position_extra_last,
            typename_fields_sorted_by_response_key_str_then_position_extra_last,
        ) = {
            let mut data_fields = self.data_fields_buffer_pool.pop();
            let mut typename_fields = self.typename_fields_buffer_pool.pop();
            for parent_field in parent_fields {
                data_fields.extend(parent_field.selection_set().data_fields());
                typename_fields.extend(parent_field.selection_set().typename_fields());
            }
            let keys = &self.ctx.cached.operation.response_keys;
            data_fields.sort_unstable_by(|left, right| {
                keys[left.response_key].cmp(&keys[right.response_key]).then(
                    left.query_position
                        .map(u16::from)
                        .unwrap_or(u16::MAX)
                        .cmp(&right.query_position.map(u16::from).unwrap_or(u16::MAX)),
                )
            });
            typename_fields.sort_unstable_by(|left, right| {
                keys[left.response_key].cmp(&keys[right.response_key]).then(
                    left.query_position
                        .map(u16::from)
                        .unwrap_or(u16::MAX)
                        .cmp(&right.query_position.map(u16::from).unwrap_or(u16::MAX)),
                )
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
            output,
            &typename_fields_sorted_by_response_key_str_then_position_extra_last,
            &data_fields_sorted_by_response_key_str_then_position_extra_last,
        );

        let requires_typename = parent_fields.iter().any(|field| field.selection_set_requires_typename);

        //
        // Creating the right shape from the partitioning
        //
        let shape = if partitions.is_empty() {
            // There is no partition so all fields are included all the time.
            let included_typename_then_data_fields = {
                let mut included = FixedBitSet::with_capacity(
                    typename_fields_sorted_by_response_key_str_then_position_extra_last.len()
                        + data_fields_sorted_by_response_key_str_then_position_extra_last.len(),
                );
                included.toggle_range(..included.len());
                included
            };

            // We may still need to know the type of the object if there is any __typename field.
            let identifier = if let [id] = output.possible_type_ids() {
                ObjectIdentifier::Known(*id)
            } else if set_id.is_some()
                || !typename_fields_sorted_by_response_key_str_then_position_extra_last.is_empty()
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
                &typename_fields_sorted_by_response_key_str_then_position_extra_last,
                &data_fields_sorted_by_response_key_str_then_position_extra_last,
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
                            &typename_fields_sorted_by_response_key_str_then_position_extra_last,
                            &data_fields_sorted_by_response_key_str_then_position_extra_last,
                            fields,
                        );
                        possibilities.push((id, shape_id));
                    }
                    partition::Partition::Many { ids, fields } => {
                        let shape_id = self.create_concrete_shape(
                            ObjectIdentifier::Anonymous,
                            set_id,
                            &typename_fields_sorted_by_response_key_str_then_position_extra_last,
                            &data_fields_sorted_by_response_key_str_then_position_extra_last,
                            fields,
                        );
                        possibilities.extend(ids.into_iter().map(|id| (id, shape_id)));
                    }
                    partition::Partition::Remaining { fields } => {
                        let n = typename_fields_sorted_by_response_key_str_then_position_extra_last.len();
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
                            &typename_fields_sorted_by_response_key_str_then_position_extra_last,
                            &data_fields_sorted_by_response_key_str_then_position_extra_last,
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
            .push(data_fields_sorted_by_response_key_str_then_position_extra_last);
        self.typename_fields_buffer_pool
            .push(typename_fields_sorted_by_response_key_str_then_position_extra_last);

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
        output: CompositeType<'ctx>,
        typename_fields: &[PartitionTypenameField<'ctx>],
        data_fields: &[PartitionDataField<'ctx>],
    ) -> partition::Partitioning<ObjectDefinitionId, FixedBitSet> {
        let mut type_condition_and_field_position_in_bitset =
            Vec::with_capacity(typename_fields.len() + data_fields.len());
        for (i, field) in typename_fields.iter().enumerate() {
            type_condition_and_field_position_in_bitset
                .push((&self.ctx.cached.query_plan[field.type_condition_ids], i));
        }
        let offset = typename_fields.len();
        for (i, field) in data_fields.iter().enumerate() {
            type_condition_and_field_position_in_bitset
                .push((&self.ctx.cached.query_plan[field.type_condition_ids], offset + i));
        }
        type_condition_and_field_position_in_bitset.sort_unstable();

        let type_conditions = type_condition_and_field_position_in_bitset
            .iter()
            .chunk_by(|(type_conditions, _)| type_conditions)
            .into_iter()
            .map(|(type_conditions, chunk)| {
                let possible_types =
                    compute_possible_types(self.ctx.schema, output.possible_type_ids(), type_conditions);
                let mut bitset = FixedBitSet::with_capacity(type_condition_and_field_position_in_bitset.len());
                for (_, i) in chunk {
                    bitset.put(*i);
                }
                (possible_types, bitset)
            })
            .collect();

        partition::partition_object_shapes(output.possible_type_ids(), type_conditions)
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

fn compute_possible_types<'s>(
    schema: &'s Schema,
    output_possible_types: &'s [ObjectDefinitionId],
    type_conditions: &'s [CompositeTypeId],
) -> Cow<'s, [ObjectDefinitionId]> {
    let Some(first) = type_conditions.first() else {
        return Cow::Borrowed(output_possible_types);
    };
    let mut intersection = {
        let first = first.walk(schema);
        let first_possible_types = first.possible_type_ids();
        let mut intersection = Vec::with_capacity(first_possible_types.len().min(output_possible_types.len()));
        let mut l = 0;
        let mut r = 0;
        while let Some((left, right)) = output_possible_types.get(l).zip(first_possible_types.get(r)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => l += 1,
                std::cmp::Ordering::Greater => r += 1,
                std::cmp::Ordering::Equal => {
                    intersection.push(*left);
                    l += 1;
                    r += 1;
                }
            }
        }
        intersection
    };

    for ty in &type_conditions[1..] {
        let ty = ty.walk(schema);
        let possible_types = ty.possible_type_ids();
        let mut n = 0;
        let mut l = 0;
        let mut r = 0;
        while let Some((left, right)) = intersection.get(l).zip(possible_types.get(r)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => l += 1,
                std::cmp::Ordering::Greater => r += 1,
                std::cmp::Ordering::Equal => {
                    intersection.swap(l, n);
                    l += 1;
                    r += 1;
                    n += 1;
                }
            }
        }
        intersection.truncate(n);
    }

    Cow::Owned(intersection)
}
