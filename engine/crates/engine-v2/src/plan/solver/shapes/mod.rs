mod partition;

use id_newtypes::IdRange;
use itertools::Itertools;
use partition::{partition_shapes, Partition};
use schema::{CompositeTypeId, DefinitionId, EntityDefinitionId, ObjectDefinitionId, Schema};
use walker::Walk;

use super::{
    DataField, DataFieldId, OperationSolution, OperationSolutionBuilder, OperationSolutionContext,
    ResponseObjectSetDefinitionId, SelectionSet, TypenameField,
};
use crate::{
    response::{
        ConcreteObjectShapeId, ConcreteObjectShapeRecord, FieldShapeId, FieldShapeRecord, ObjectIdentifier,
        PolymorphicObjectShapeId, PolymorphicObjectShapeRecord, SafeResponseKey, Shape, Shapes,
    },
    utils::BufferPool,
};

impl OperationSolutionBuilder<'_> {
    pub(super) fn populate_shapes_after_partition_generation(&mut self) {
        let mut plans = std::mem::take(&mut self.operation.query_partitions);
        let mut builder = ShapesBuilder {
            schema: self.schema,
            operation: &self.operation,
            shapes: Shapes::default(),
            field_id_to_field_shape_ids: Vec::new(),
            data_field_ids_with_selection_set_requiring_typename: Vec::new(),
            field_shapes_buffer_pool: BufferPool::default(),
            data_fields_buffer_pool: BufferPool::default(),
            typename_fields_buffer_pool: BufferPool::default(),
        };
        let ctx = OperationSolutionContext {
            schema: self.schema,
            operation_solution: &self.operation,
        };
        for plan in &mut plans {
            plan.shape_id =
                builder.create_root_shape_for(plan.entity_definition_id, plan.selection_set_record.walk(ctx));
        }
        let data_field_ids_with_selection_set_requiring_typename =
            builder.data_field_ids_with_selection_set_requiring_typename;
        let mut field_id_to_field_shape_ids = builder.field_id_to_field_shape_ids;

        let shapes = builder.shapes;
        self.operation.query_partitions = plans;
        self.operation.shapes = shapes;

        for id in data_field_ids_with_selection_set_requiring_typename {
            self.operation[id].selection_set_requires_typename = true
        }

        field_id_to_field_shape_ids.sort_unstable();
        self.operation
            .field_shape_refs
            .reserve(field_id_to_field_shape_ids.len());
        for (data_field_id, field_shape_ids) in field_id_to_field_shape_ids
            .into_iter()
            .chunk_by(|(id, _)| *id)
            .into_iter()
        {
            let start = self.operation.field_shape_refs.len();
            self.operation
                .field_shape_refs
                .extend(field_shape_ids.into_iter().map(|(_, id)| id));
            self.operation[data_field_id].shape_ids = IdRange::from(start..self.operation.field_shape_refs.len());
        }
    }
}

pub(super) struct ShapesBuilder<'ctx> {
    schema: &'ctx Schema,
    operation: &'ctx OperationSolution,
    shapes: Shapes,
    field_id_to_field_shape_ids: Vec<(DataFieldId, FieldShapeId)>,
    data_field_ids_with_selection_set_requiring_typename: Vec<DataFieldId>,
    field_shapes_buffer_pool: BufferPool<(FieldShapeRecord, Vec<DataFieldId>)>,
    data_fields_buffer_pool: BufferPool<DataField<'ctx>>,
    typename_fields_buffer_pool: BufferPool<TypenameField<'ctx>>,
}

impl<'ctx> ShapesBuilder<'ctx> {
    fn create_root_shape_for(
        &mut self,
        entity_definition_id: EntityDefinitionId,
        selection_set: SelectionSet<'ctx>,
    ) -> ConcreteObjectShapeId {
        let exemplar = match entity_definition_id {
            EntityDefinitionId::Object(id) => id,
            EntityDefinitionId::Interface(id) => self.schema[id].possible_type_ids[0],
        };

        let mut data_fields = self.data_fields_buffer_pool.pop();
        data_fields.extend(selection_set.data_fields());
        data_fields.sort_unstable_by(|left, right| {
            let l = left.key;
            let r = right.key;
            l.response_key
                .cmp(&r.response_key)
                .then(l.query_position.cmp(&r.query_position))
        });

        let mut typename_fields = self.typename_fields_buffer_pool.pop();
        typename_fields.extend(selection_set.typename_fields());
        typename_fields.sort_unstable_by(|left, right| {
            let l = left.key;
            let r = right.key;
            l.response_key
                .cmp(&r.response_key)
                .then(l.query_position.cmp(&r.query_position))
        });

        let id = self.create_concrete_shape(exemplar, None, &data_fields, &typename_fields);
        self.data_fields_buffer_pool.push(data_fields);
        self.typename_fields_buffer_pool.push(typename_fields);

        id
    }

    fn create_concrete_shape(
        &mut self,
        exemplar_object_id: ObjectDefinitionId,
        maybe_response_object_set_id: Option<ResponseObjectSetDefinitionId>,
        data_fields_sorted_by_response_key_then_position: &[DataField<'ctx>],
        typename_fields_sorted_by_response_key_then_position: &[TypenameField<'ctx>],
    ) -> ConcreteObjectShapeId {
        let schema = self.schema;
        tracing::trace!("Creating shape for exemplar {}", schema.walk(exemplar_object_id).name());

        let typename_response_edges = typename_fields_sorted_by_response_key_then_position
            .iter()
            .filter(|field| match field.type_condition_id {
                CompositeTypeId::Object(id) => id == exemplar_object_id,
                CompositeTypeId::Interface(id) => {
                    schema[id].possible_type_ids.binary_search(&exemplar_object_id).is_ok()
                }
                CompositeTypeId::Union(id) => schema[id].possible_type_ids.binary_search(&exemplar_object_id).is_ok(),
            })
            .dedup_by(|a, b| a.key.response_key == b.key.response_key)
            .map(|field| field.key)
            .collect();

        let mut fields_buffer = self.data_fields_buffer_pool.pop();
        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();

        let mut start = 0;
        while let Some(response_key) = data_fields_sorted_by_response_key_then_position
            .get(start)
            .map(|field| field.key.response_key)
        {
            let mut end = start + 1;
            while data_fields_sorted_by_response_key_then_position
                .get(end)
                .map(|field| field.key.response_key == response_key)
                .unwrap_or_default()
            {
                end += 1;
            }

            fields_buffer.clear();
            fields_buffer.extend(
                data_fields_sorted_by_response_key_then_position[start..end]
                    .iter()
                    .filter(|field| match field.definition().parent_entity_id {
                        EntityDefinitionId::Object(id) => id == exemplar_object_id,
                        EntityDefinitionId::Interface(id) => {
                            schema[id].possible_type_ids.binary_search(&exemplar_object_id).is_ok()
                        }
                    }),
            );

            if let Some(field) = fields_buffer.first().copied() {
                let shape = self.create_data_field_shape(response_key, &mut fields_buffer, field);
                field_shapes_buffer.push((shape, fields_buffer.iter().map(|field| field.id).collect()));
            }
            start = end;
        }

        let shape = ConcreteObjectShapeRecord {
            set_id: maybe_response_object_set_id,
            identifier: ObjectIdentifier::Anonymous,
            typename_response_edges,
            field_shape_ids: {
                let start = self.shapes.fields.len();
                let keys = &self.operation.response_keys;
                field_shapes_buffer.sort_unstable_by(|a, b| keys[a.0.expected_key].cmp(&keys[b.0.expected_key]));
                for (shape, mut field_ids) in field_shapes_buffer.drain(..) {
                    self.shapes.fields.push(shape);
                    let field_shape_id = FieldShapeId::from(self.shapes.fields.len() - 1);
                    for field_id in field_ids.drain(..) {
                        self.field_id_to_field_shape_ids.push((field_id, field_shape_id));
                    }
                }
                self.field_shapes_buffer_pool.push(field_shapes_buffer);

                IdRange::from(start..self.shapes.fields.len())
            },
        };

        self.push_concrete_shape(shape)
    }

    fn create_data_field_shape(
        &mut self,
        response_key: SafeResponseKey,
        fields: &mut [DataField<'ctx>],
        field: DataField<'ctx>,
    ) -> FieldShapeRecord {
        let ty = field.definition().ty();
        let shape = match ty.definition_id {
            DefinitionId::Scalar(id) => Shape::Scalar(id.walk(self.schema).ty),
            DefinitionId::Enum(id) => Shape::Enum(id),
            DefinitionId::Interface(id) => self.create_field_output_shape(fields, id.into()),
            DefinitionId::Object(id) => self.create_field_output_shape(fields, id.into()),

            DefinitionId::Union(id) => self.create_field_output_shape(fields, id.into()),
            DefinitionId::InputObject(_) => unreachable!("Cannot be an output"),
        };

        let required_field_id = fields.iter().find_map(|field| field.matching_requirement_id);

        FieldShapeRecord {
            expected_key: response_key,
            key: field.key,
            id: field.id,
            required_field_id,
            definition_id: field.definition().id,
            shape,
            wrapping: ty.wrapping,
        }
    }

    fn create_field_output_shape(&mut self, parent_fields: &[DataField<'ctx>], output_id: CompositeTypeId) -> Shape {
        let maybe_response_object_set_id = parent_fields.iter().find_map(|field| field.output_id);
        let mut data_fields = self.data_fields_buffer_pool.pop();
        let mut typename_fields = self.typename_fields_buffer_pool.pop();
        for parent_field in parent_fields {
            data_fields.extend(parent_field.selection_set().data_fields());
            typename_fields.extend(parent_field.selection_set().typename_fields());
        }
        let shape = self.collect_object_shapes(
            output_id,
            maybe_response_object_set_id,
            &mut data_fields,
            &mut typename_fields,
        );
        self.data_fields_buffer_pool.push(data_fields);
        self.typename_fields_buffer_pool.push(typename_fields);
        match shape {
            Shape::Scalar(_) | Shape::Enum(_) => {}
            Shape::ConcreteObject(id) => {
                if matches!(
                    self.shapes[id].identifier,
                    ObjectIdentifier::UnionTypename(_) | ObjectIdentifier::InterfaceTypename(_)
                ) {
                    self.data_field_ids_with_selection_set_requiring_typename
                        .extend(parent_fields.iter().map(|field| field.id));
                }
            }
            Shape::PolymorphicObject(_) => {
                self.data_field_ids_with_selection_set_requiring_typename
                    .extend(parent_fields.iter().map(|field| field.id));
            }
        }
        shape
    }

    fn collect_object_shapes(
        &mut self,
        ty: CompositeTypeId,
        maybe_response_object_set_id: Option<ResponseObjectSetDefinitionId>,
        data_fields: &mut [DataField<'ctx>],
        typename_fields: &mut [TypenameField<'ctx>],
    ) -> Shape {
        let output: &[ObjectDefinitionId] = match &ty {
            CompositeTypeId::Object(id) => std::array::from_ref(id),
            CompositeTypeId::Interface(id) => &self.schema[*id].possible_type_ids,
            CompositeTypeId::Union(id) => &self.schema[*id].possible_type_ids,
        };
        let shape_partitions = self.compute_shape_partitions(output, data_fields, typename_fields);

        data_fields.sort_unstable_by(|left, right| {
            let l = left.key;
            let r = right.key;
            l.response_key
                .cmp(&r.response_key)
                .then(l.query_position.cmp(&r.query_position))
        });
        let data_fields_sorted_by_response_key_then_position = data_fields;
        typename_fields.sort_unstable_by(|left, right| {
            let l = left.key;
            let r = right.key;
            l.response_key
                .cmp(&r.response_key)
                .then(l.query_position.cmp(&r.query_position))
        });
        let typename_fields_sorted_by_response_key_then_position = typename_fields;

        if let Some(partitions) = shape_partitions {
            let mut possibilities = Vec::new();
            for partition in partitions {
                match partition {
                    Partition::One(id) => {
                        let shape_id = self.create_concrete_shape(
                            id,
                            maybe_response_object_set_id,
                            data_fields_sorted_by_response_key_then_position,
                            typename_fields_sorted_by_response_key_then_position,
                        );
                        possibilities.push((id, shape_id))
                    }
                    Partition::Many(ids) => {
                        let shape_id = self.create_concrete_shape(
                            ids[0],
                            maybe_response_object_set_id,
                            data_fields_sorted_by_response_key_then_position,
                            typename_fields_sorted_by_response_key_then_position,
                        );
                        possibilities.extend(ids.into_iter().map(|id| (id, shape_id)))
                    }
                }
            }
            Shape::PolymorphicObject(self.push_polymorphic_shape(PolymorphicObjectShapeRecord { possibilities }))
        } else {
            let shape_id = self.create_concrete_shape(
                output[0],
                maybe_response_object_set_id,
                data_fields_sorted_by_response_key_then_position,
                typename_fields_sorted_by_response_key_then_position,
            );
            let shape = &mut self.shapes[shape_id];
            if output.len() == 1 {
                shape.identifier = ObjectIdentifier::Known(output[0]);
            } else if shape.set_id.is_some() || !shape.typename_response_edges.is_empty() {
                shape.identifier = match ty {
                    CompositeTypeId::Interface(id) => ObjectIdentifier::InterfaceTypename(id),
                    CompositeTypeId::Union(id) => ObjectIdentifier::UnionTypename(id),
                    CompositeTypeId::Object(_) => unreachable!(),
                }
            }
            Shape::ConcreteObject(shape_id)
        }
    }

    fn compute_shape_partitions(
        &self,
        output: &[ObjectDefinitionId],
        data_fields: &[DataField<'ctx>],
        typename_fields: &[TypenameField<'ctx>],
    ) -> Option<Vec<Partition<ObjectDefinitionId>>> {
        let mut type_conditions = Vec::new();
        for field in typename_fields {
            type_conditions.push(field.type_condition_id);
        }
        for field in data_fields {
            type_conditions.push(field.definition().parent_entity_id.into());
        }

        let mut single_object_ids = Vec::new();
        let mut other_type_conditions = Vec::new();

        type_conditions.sort_unstable();
        for type_condition in type_conditions.into_iter().dedup() {
            match type_condition {
                CompositeTypeId::Object(id) => single_object_ids.push(id),
                CompositeTypeId::Interface(id) => {
                    other_type_conditions.push(self.schema[id].possible_type_ids.as_slice())
                }
                CompositeTypeId::Union(id) => other_type_conditions.push(self.schema[id].possible_type_ids.as_slice()),
            }
        }

        partition_shapes(output, single_object_ids, other_type_conditions)
    }

    fn push_concrete_shape(&mut self, shape: ConcreteObjectShapeRecord) -> ConcreteObjectShapeId {
        let id = self.shapes.concrete.len().into();
        self.shapes.concrete.push(shape);
        id
    }

    fn push_polymorphic_shape(&mut self, mut shape: PolymorphicObjectShapeRecord) -> PolymorphicObjectShapeId {
        let id = self.shapes.polymorphic.len().into();
        let schema = self.schema;
        shape.possibilities.sort_unstable_by(|a, b| {
            let a = &schema[schema[a.0].name_id];
            let b = &schema[schema[b.0].name_id];
            a.cmp(b)
        });
        self.shapes.polymorphic.push(shape);
        id
    }
}
