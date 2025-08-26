mod partition;

use std::borrow::Cow;

use fixedbitset::FixedBitSet;
use id_newtypes::IdRange;
use im::HashSet;
use itertools::Itertools;
use operation::{QueryPosition, ResponseKey};
use schema::{
    CompositeType, CompositeTypeId, ObjectDefinitionId, Schema, SubgraphId, TypeDefinition, TypeDefinitionId,
};
use walker::Walk;

use crate::{
    prepare::{
        BatchFieldShape, DataOrLookupFieldId, DefaultFieldShapeRecord, Derive, DerivedEntityShapeId,
        DerivedEntityShapeRecord, LookupFieldId, OnRootFieldsError, RootFieldsShapeId, RootFieldsShapeRecord,
        TypenameShapeRecord,
        cached::{
            CachedOperationContext, ConcreteShapeId, ConcreteShapeRecord, DataField, DataFieldId, FieldShapeId,
            FieldShapeRecord, FieldShapeRefId, ObjectIdentifier, PartitionSelectionSet, PolymorphicShapeId,
            PolymorphicShapeRecord, ResponseObjectSetId, Shape, Shapes, TypenameField,
        },
    },
    utils::BufferPool,
};

use super::Solver;

impl Solver<'_> {
    pub(super) fn populate_shapes_after_query_plan(&mut self) {
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
            typename_shapes_buffer_pool: BufferPool::default(),
            data_fields_buffer_pool: BufferPool::default(),
            typename_fields_buffer_pool: BufferPool::default(),
            data_fields_shape_count: vec![0; self.output.query_plan.data_fields.len()],
            current_subgraph_id: SubgraphId::Introspection,
        };

        // Create all shapes for the given QueryPartition
        for query_partition in &mut query_partitions {
            builder.current_subgraph_id = query_partition.resolver_definition_id.walk(self.schema).subgraph_id();
            let mut shape_id = builder.create_root_shape_for(query_partition.selection_set_record.walk(ctx));
            let shape_id = loop {
                let field_shape_ids = builder.shapes[shape_id].field_shape_ids;
                if builder.shapes[field_shape_ids].iter().all(|fs| fs.id.is_data()) {
                    break shape_id;
                }
                let field_shape = &builder.shapes[field_shape_ids.start];
                if field_shape_ids.len() != 1 || !field_shape.id.is_lookup() {
                    break shape_id;
                }
                let Shape::Concrete(id) = field_shape.shape else {
                    unreachable!();
                };
                shape_id = id;
            };
            query_partition.shape_id = builder.build_root_fields_shape(shape_id);
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
            data_field.shape_ids_ref = IdRange::from(len..len);
            len += count as usize;
        }
        let mut field_shape_refs = vec![FieldShapeId::from(0usize); len];
        for (i, field_shape) in shapes.fields.iter().enumerate() {
            let DataOrLookupFieldId::Data(field_id) = field_shape.id else {
                continue;
            };
            let end = &mut self.output.query_plan[field_id].shape_ids_ref.end;
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
    data_field_ids_with_selection_set_requiring_typename: Vec<DataFieldId>,
    field_shapes_buffer_pool: BufferPool<FieldShapeRecord>,
    typename_shapes_buffer_pool: BufferPool<TypenameShapeRecord>,
    data_fields_buffer_pool: BufferPool<DataField<'ctx>>,
    typename_fields_buffer_pool: BufferPool<TypenameField<'ctx>>,
    current_subgraph_id: SubgraphId,
}

impl<'ctx> ShapesBuilder<'ctx> {
    fn build_root_fields_shape(&mut self, shape_id: ConcreteShapeId) -> RootFieldsShapeId {
        let shape = &self.shapes[shape_id];
        let mut propagate_null = false;
        let mut default_fields = Vec::new();
        let mut location_and_key = None;

        for field_shape in &self.shapes[shape.field_shape_ids] {
            let Some(query_position) = field_shape.query_position_before_modifications else {
                continue;
            };
            let field = field_shape
                .id
                .as_data()
                .walk(self.ctx)
                .expect("We shouldn't generate errors for lookup fields");
            location_and_key.get_or_insert_with(|| (field.location, field.response_key));
            if propagate_null | field_shape.wrapping.is_non_null() {
                propagate_null = true;
                continue;
            }
            default_fields.push(DefaultFieldShapeRecord {
                query_position_before_modifications: query_position,
                response_key: field_shape.response_key,
                id: field.id.into(),
                value: None,
            })
        }

        let typename_field_shapes = &self.shapes[shape.typename_shape_ids];
        if let Some(first_typename_shape) = typename_field_shapes.first() {
            if let ObjectIdentifier::Known(object_id) = shape.identifier {
                let name_id = object_id.walk(self.ctx).name_id;
                default_fields.extend(typename_field_shapes.iter().filter_map(|shape| {
                    shape
                        .query_position_before_modifications
                        .map(|qp| DefaultFieldShapeRecord {
                            query_position_before_modifications: qp,
                            response_key: shape.response_key,
                            id: shape.id.into(),
                            value: Some(name_id),
                        })
                }));
            } else {
                propagate_null = true;
                if location_and_key.is_none() {
                    location_and_key = Some((first_typename_shape.location, first_typename_shape.response_key));
                }
            }
        }

        let shape = if let Some(error_location_and_key) = location_and_key {
            if propagate_null {
                RootFieldsShapeRecord {
                    concrete_shape_id: shape_id,
                    on_error: OnRootFieldsError::PropagateNull { error_location_and_key },
                }
            } else {
                default_fields.sort_unstable_by(|a, b| {
                    let a = a
                        .response_key
                        .with_position(Some(a.query_position_before_modifications));
                    let b = b
                        .response_key
                        .with_position(Some(b.query_position_before_modifications));
                    a.cmp(&b)
                });
                let start = self.shapes.default_fields.len();
                self.shapes.default_fields.append(&mut default_fields);
                RootFieldsShapeRecord {
                    concrete_shape_id: shape_id,
                    on_error: OnRootFieldsError::Default {
                        fields_sorted_by_key: (start..self.shapes.default_fields.len()).into(),
                        error_location_and_key,
                    },
                }
            }
        } else {
            RootFieldsShapeRecord {
                concrete_shape_id: shape_id,
                on_error: OnRootFieldsError::Skip,
            }
        };

        let id = self.shapes.root_fields.len().into();
        self.shapes.root_fields.push(shape);
        id
    }

    fn create_root_shape_for(&mut self, selection_set: PartitionSelectionSet<'ctx>) -> ConcreteShapeId {
        if !selection_set.lookup_field_ids.is_empty() {
            debug_assert!(
                selection_set
                    .data_field_ids_ordered_by_parent_entity_then_key
                    .is_empty()
                    && selection_set.typename_field_ids.is_empty()
            );
            return self.create_lookup_fields_set_shape(selection_set.lookup_field_ids);
        }

        let keys = &self.ctx.cached.operation.response_keys;
        let data_fields_sorted_by_response_key_str_then_position_extra_last = {
            let mut fields = self.data_fields_buffer_pool.pop();
            fields.extend(selection_set.data_fields());
            fields.sort_unstable_by(|left, right| {
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
            });
            fields
        };

        let typename_fields_sorted_by_response_key_str_then_position_extra_last = {
            let mut fields = self.typename_fields_buffer_pool.pop();
            fields.extend(selection_set.typename_fields());
            fields.sort_unstable_by(|left, right| {
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
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

    fn create_lookup_fields_set_shape(&mut self, field_ids: IdRange<LookupFieldId>) -> ConcreteShapeId {
        let mut buffer = self.field_shapes_buffer_pool.pop();
        for field_id in field_ids {
            buffer.push(self.create_lookup_field_shape(field_id));
        }

        let field_shape_ids = {
            let start = self.shapes.fields.len();
            buffer.sort_unstable_by(|a, b| a.id.cmp(&b.id));
            debug_assert!(
                buffer
                    .iter()
                    .map(|f| f.expected_key)
                    .collect::<HashSet<ResponseKey>>()
                    .len()
                    == buffer.len()
            );
            self.shapes.fields.append(&mut buffer);
            self.field_shapes_buffer_pool.push(buffer);
            IdRange::from(start..self.shapes.fields.len())
        };

        let shape = ConcreteShapeRecord {
            set_id: None,
            identifier: ObjectIdentifier::Anonymous,
            typename_shape_ids: Default::default(),
            field_shape_ids,
            derived_field_shape_ids_start: field_shape_ids.end,
        };

        self.push_concrete_shape(shape)
    }

    fn create_lookup_field_shape(&mut self, field_id: LookupFieldId) -> FieldShapeRecord {
        let field = field_id.walk(self.ctx);
        FieldShapeRecord {
            expected_key: field.subgraph_key,
            query_position_before_modifications: None,
            response_key: field.subgraph_key,
            id: field.id.into(),
            shape: Shape::Concrete(self.create_root_shape_for(field.selection_set())),
            wrapping: field.definition().ty().wrapping,
        }
    }

    /// Create the expected shape with known expected fields, applying the GraphQL field collection
    /// logic.
    fn create_concrete_shape(
        &mut self,
        identifier: ObjectIdentifier,
        set_id: Option<ResponseObjectSetId>,
        typename_fields_sorted_by_response_key_str_then_position_extra_last: &[TypenameField<'ctx>],
        data_fields_sorted_by_response_key_str_then_position_extra_last: &[DataField<'ctx>],
        included_typename_then_data_fields: FixedBitSet,
    ) -> ConcreteShapeId {
        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut distinct_typename_shapes = self.typename_shapes_buffer_pool.pop();
        let mut included = included_typename_then_data_fields.into_ones();

        while let Some(i) = included.next() {
            if let Some(field) = typename_fields_sorted_by_response_key_str_then_position_extra_last.get(i) {
                if distinct_typename_shapes
                    .last()
                    // fields aren't sorted by the response key but by the string value they point
                    // to. However, response keys are deduplicated so the equality also works here
                    // to ensure we only have distinct values.
                    .is_none_or(|shape| shape.response_key != field.response_key)
                {
                    distinct_typename_shapes.push(TypenameShapeRecord {
                        query_position_before_modifications: field.query_position,
                        response_key: field.response_key,
                        id: field.id,
                        location: field.location,
                    });
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

                group.clear();
                self.data_fields_buffer_pool.push(group);
            }
        }

        debug_assert!(!field_shapes_buffer.is_empty() || !distinct_typename_shapes.is_empty());
        let field_shape_ids = {
            let start = self.shapes.fields.len();
            field_shapes_buffer.sort_unstable_by(|a, b| {
                a.shape
                    .is_derive_entity()
                    .cmp(&b.shape.is_derive_entity())
                    .then(a.id.cmp(&b.id))
            });
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
        };

        let shape = ConcreteShapeRecord {
            set_id,
            identifier,
            typename_shape_ids: {
                let start = self.shapes.typename_fields.len();
                self.shapes.typename_fields.append(&mut distinct_typename_shapes);
                self.typename_shapes_buffer_pool.push(distinct_typename_shapes);
                IdRange::from(start..self.shapes.typename_fields.len())
            },
            field_shape_ids,
            derived_field_shape_ids_start: self.shapes[field_shape_ids]
                .iter()
                .position(|field| field.shape.is_derive_entity())
                .map(|offset| (usize::from(field_shape_ids.start) + offset).into())
                .unwrap_or(field_shape_ids.end),
        };

        self.push_concrete_shape(shape)
    }

    fn create_data_field_shape(&mut self, group: &mut [DataField<'ctx>], first: DataField<'ctx>) -> FieldShapeRecord {
        let ty = first.definition().ty();
        let shape = if let Some(Derive::Root { batch_field_id }) = first.derive {
            Shape::DeriveEntity(self.create_derived_entity(batch_field_id, group, ty.definition_id.as_object()))
        } else {
            match ty.definition() {
                TypeDefinition::Scalar(scalar) => Shape::Scalar(scalar.ty),
                TypeDefinition::Enum(enm) => Shape::Enum(enm.id),
                TypeDefinition::Interface(interface) => {
                    self.create_field_composite_type_output_shape(group, interface.into())
                }
                TypeDefinition::Object(object) => self.create_field_composite_type_output_shape(group, object.into()),

                TypeDefinition::Union(union) => self.create_field_composite_type_output_shape(group, union.into()),
                TypeDefinition::InputObject(_) => unreachable!("Cannot be an output"),
            }
        };

        FieldShapeRecord {
            expected_key: first.subgraph_key.unwrap_or(first.response_key),
            query_position_before_modifications: first.query_position,
            response_key: first.response_key,
            id: first.id.into(),
            shape,
            wrapping: ty.wrapping,
        }
    }

    fn create_derived_entity(
        &mut self,
        batch_field_id: Option<DataFieldId>,
        parent_fields: &[DataField<'ctx>],
        object_definition_id: Option<ObjectDefinitionId>,
    ) -> DerivedEntityShapeId {
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
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
            });
            typename_fields.sort_unstable_by(|left, right| {
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
            });
            (data_fields, typename_fields)
        };

        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut distinct_typename_shapes = self.typename_shapes_buffer_pool.pop();

        for field in typename_fields_sorted_by_response_key_str_then_position_extra_last {
            if distinct_typename_shapes
                .last()
                // fields aren't sorted by the response key but by the string value they point
                // to. However, response keys are deduplicated so the equality also works here
                // to ensure we only have distinct values.
                .is_none_or(|shape| shape.response_key != field.response_key)
            {
                distinct_typename_shapes.push(TypenameShapeRecord {
                    query_position_before_modifications: field.query_position,
                    response_key: field.response_key,
                    id: field.id,
                    location: field.location,
                });
            }
        }

        for field in data_fields_sorted_by_response_key_str_then_position_extra_last {
            if field_shapes_buffer
                .last()
                // fields aren't sorted by the response key but by the string value they point
                // to. However, response keys are deduplicated so the equality also works here
                // to ensure we only have distinct values.
                .is_none_or(|shape| shape.response_key != field.response_key)
            {
                let ty = field.definition().ty();
                let (expected_key, shape) = match field.derive {
                    Some(Derive::From(id)) => {
                        let derive_from = id.walk(self.ctx);
                        let shape = match ty.definition_id {
                            TypeDefinitionId::Scalar(_) | TypeDefinitionId::Enum(_) => {
                                Shape::DeriveFrom(derive_from.query_position)
                            }
                            _ => unreachable!("Nested object are not supported yet for derived."),
                        };
                        (derive_from.response_key, shape)
                    }
                    Some(Derive::ScalarAsField) => {
                        // Expected key doesn't matter here.
                        (field.response_key, Shape::DeriveFromScalar)
                    }
                    _ => {
                        unreachable!(
                            "Derived fields should always have a From variant, found: {:?}",
                            field.derive
                        );
                    }
                };
                self.data_fields_shape_count[usize::from(field.id)] += 1;
                field_shapes_buffer.push(FieldShapeRecord {
                    expected_key,
                    query_position_before_modifications: field.query_position,
                    response_key: field.response_key,
                    id: field.id.into(),
                    shape,
                    wrapping: ty.wrapping,
                });
            }
        }

        let shape = DerivedEntityShapeRecord {
            set_id,
            object_definition_id,
            batch_field_shape: batch_field_id.walk(self.ctx).map(|field| BatchFieldShape {
                key: field.key(),
                wrapping: field.definition().ty().wrapping,
            }),
            typename_shape_ids: {
                let start = self.shapes.typename_fields.len();
                self.shapes.typename_fields.append(&mut distinct_typename_shapes);
                self.typename_shapes_buffer_pool.push(distinct_typename_shapes);
                IdRange::from(start..self.shapes.typename_fields.len())
            },
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
        self.push_derived_entity_shape(shape)
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
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
            });
            typename_fields.sort_unstable_by(|left, right| {
                keys[left.response_key]
                    .cmp(&keys[right.response_key])
                    .then_with(|| QueryPosition::cmp_with_none_last(left.query_position, right.query_position))
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
        typename_fields: &[TypenameField<'ctx>],
        data_fields: &[DataField<'ctx>],
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

    fn push_derived_entity_shape(&mut self, shape: DerivedEntityShapeRecord) -> DerivedEntityShapeId {
        let id = self.shapes.derived_entities.len().into();
        self.shapes.derived_entities.push(shape);
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
