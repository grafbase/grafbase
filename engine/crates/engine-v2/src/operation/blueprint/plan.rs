use id_newtypes::IdRange;
use im::HashMap;
use itertools::Itertools;
use schema::{EntityDefinitionId, FieldDefinition, ObjectDefinitionId, Schema};

use crate::{
    operation::{
        ExtraField, Field, FieldId, FieldWalker, LogicalPlanId, LogicalPlanResponseBlueprint, QueryField,
        SelectionSetId, SelectionSetType, TypeNameField,
    },
    response::{
        ConcreteObjectShape, ConcreteObjectShapeId, FieldShape, FieldShapeId, ObjectIdentifier, PolymorphicObjectShape,
        PolymorphicObjectShapeId, ResponseKey, ResponseObjectSetId, Shape,
    },
};

use super::{
    partition::{partition_shapes, Partition},
    ResponseBlueprintBuilder, ToBuild,
};

pub(super) struct LogicalPlanResponseBlueprintBuilder<'schema, 'op, 'builder> {
    builder: &'builder mut ResponseBlueprintBuilder<'schema, 'op>,
    logical_plan_id: LogicalPlanId,
}

impl<'schema, 'op, 'builder> std::ops::Deref for LogicalPlanResponseBlueprintBuilder<'schema, 'op, 'builder> {
    type Target = ResponseBlueprintBuilder<'schema, 'op>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'schema, 'op, 'builder> std::ops::DerefMut for LogicalPlanResponseBlueprintBuilder<'schema, 'op, 'builder> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.builder
    }
}

impl<'schema, 'op, 'builder> LogicalPlanResponseBlueprintBuilder<'schema, 'op, 'builder>
where
    'schema: 'op,
{
    pub(super) fn build(
        builder: &'builder mut ResponseBlueprintBuilder<'schema, 'op>,
        ToBuild {
            logical_plan_id,
            input_id,
            root_field_ids,
        }: &ToBuild,
    ) -> LogicalPlanResponseBlueprint {
        let start = builder.blueprint.response_object_sets_to_type.len();
        let mut builder = LogicalPlanResponseBlueprintBuilder {
            builder,
            logical_plan_id: *logical_plan_id,
        };
        let concrete_shape_id = builder.create_root_shape_for(builder.plan[*logical_plan_id].entity_id, root_field_ids);
        LogicalPlanResponseBlueprint {
            input_id: *input_id,
            output_ids: IdRange::from(start..builder.blueprint.response_object_sets_to_type.len()),
            concrete_shape_id,
        }
    }

    fn create_root_shape_for(
        &mut self,
        entity_id: EntityDefinitionId,
        root_field_ids: &[FieldId],
    ) -> ConcreteObjectShapeId {
        let exemplar = match entity_id {
            EntityDefinitionId::Object(id) => id,
            EntityDefinitionId::Interface(id) => self.schema[id].possible_type_ids[0],
        };

        let walker = self.walker();
        let mut root_fields = root_field_ids.iter().map(|id| walker.walk(*id)).collect::<Vec<_>>();
        root_fields.sort_unstable_by(|left, right| {
            left.response_key()
                .cmp(&right.response_key())
                .then(left.as_ref().query_position().cmp(&right.as_ref().query_position()))
        });

        self.create_shape_for(exemplar, None, &root_fields, &mut Vec::new())
    }

    fn create_object_shape(&mut self, ty: SelectionSetType, merged_selection_set_ids: Vec<SelectionSetId>) -> Shape {
        let mut plan_field_ids: Vec<FieldId> = Vec::new();
        let mut children_plan: HashMap<LogicalPlanId, Vec<FieldId>> = HashMap::new();
        for id in &merged_selection_set_ids {
            for field in self.walker().walk(*id).fields() {
                let plan_id = self.plan.field_to_logical_plan_id[usize::from(field.id())];
                if plan_id == self.logical_plan_id {
                    plan_field_ids.push(field.id());
                } else {
                    children_plan.entry(plan_id).or_default().push(field.id());
                }
            }
        }
        let maybe_response_object_set_id = if !children_plan.is_empty() {
            let id = self.next_response_object_set_id(ty);
            self.to_build_stack
                .extend(children_plan.into_iter().map(|(plan_id, root_fields)| ToBuild {
                    input_id: id,
                    logical_plan_id: plan_id,
                    root_field_ids: root_fields,
                }));
            Some(id)
        } else if merged_selection_set_ids
            .iter()
            .any(|&id| self.plan.selection_set_to_objects_must_be_tracked[id])
        {
            Some(self.next_response_object_set_id(ty))
        } else {
            None
        };

        if let Some(set_id) = maybe_response_object_set_id {
            for &id in &merged_selection_set_ids {
                self.selection_set_to_response_object_set[usize::from(id)] = Some(set_id);
            }
        }

        let shape = self.collect_object_shapes(ty, maybe_response_object_set_id, plan_field_ids);
        match shape {
            Shape::Scalar(_) => {}
            Shape::ConcreteObject(id) => {
                if matches!(
                    self.blueprint[id].identifier,
                    ObjectIdentifier::UnionTypename(_) | ObjectIdentifier::InterfaceTypename(_)
                ) {
                    self.blueprint
                        .selection_set_to_requires_typename
                        .set(merged_selection_set_ids[0], true);
                }
            }
            Shape::PolymorphicObject(_) => {
                self.blueprint
                    .selection_set_to_requires_typename
                    .set(merged_selection_set_ids[0], true);
            }
        }
        shape
    }

    fn collect_object_shapes(
        &mut self,
        ty: SelectionSetType,
        maybe_response_object_set_id: Option<ResponseObjectSetId>,
        field_ids: Vec<FieldId>,
    ) -> Shape {
        let output: &[ObjectDefinitionId] = match &ty {
            SelectionSetType::Object(id) => std::array::from_ref(id),
            SelectionSetType::Interface(id) => &self.schema[*id].possible_type_ids,
            SelectionSetType::Union(id) => &self.schema[*id].possible_type_ids,
        };
        let shape_partitions = self.compute_shape_partitions(output, &field_ids);

        let walker = self.walker();
        let mut fields_sorted_by_response_key_then_position =
            field_ids.into_iter().map(|id| walker.walk(id)).collect::<Vec<_>>();
        fields_sorted_by_response_key_then_position.sort_unstable_by(|left, right| {
            left.response_key()
                .cmp(&right.response_key())
                .then(left.as_ref().query_position().cmp(&right.as_ref().query_position()))
        });
        let mut buffer = Vec::new();

        if let Some(partitions) = shape_partitions {
            let mut possibilities = Vec::new();
            for partition in partitions {
                match partition {
                    Partition::One(id) => {
                        let shape_id = self.create_shape_for(
                            id,
                            maybe_response_object_set_id,
                            &fields_sorted_by_response_key_then_position,
                            &mut buffer,
                        );
                        possibilities.push((id, shape_id))
                    }
                    Partition::Many(ids) => {
                        let shape_id = self.create_shape_for(
                            ids[0],
                            maybe_response_object_set_id,
                            &fields_sorted_by_response_key_then_position,
                            &mut buffer,
                        );
                        possibilities.extend(ids.into_iter().map(|id| (id, shape_id)))
                    }
                }
            }
            Shape::PolymorphicObject(self.push_polymorphic_shape(PolymorphicObjectShape { possibilities }))
        } else {
            let shape_id = self.create_shape_for(
                output[0],
                maybe_response_object_set_id,
                &fields_sorted_by_response_key_then_position,
                &mut buffer,
            );
            let shape = &mut self.blueprint[shape_id];
            if output.len() == 1 {
                shape.identifier = ObjectIdentifier::Known(output[0]);
            } else if shape.set_id.is_some() || !shape.typename_response_edges.is_empty() {
                shape.identifier = match ty {
                    SelectionSetType::Interface(id) => ObjectIdentifier::InterfaceTypename(id),
                    SelectionSetType::Union(id) => ObjectIdentifier::UnionTypename(id),
                    SelectionSetType::Object(_) => unreachable!(),
                }
            }
            Shape::ConcreteObject(shape_id)
        }
    }

    fn compute_shape_partitions(
        &self,
        output: &[ObjectDefinitionId],
        field_ids: &[FieldId],
    ) -> Option<Vec<Partition<ObjectDefinitionId>>> {
        let mut type_conditions = Vec::new();
        for field_id in field_ids {
            match &self.operation[*field_id] {
                Field::TypeName(TypeNameField { type_condition, .. }) => type_conditions.push(*type_condition),
                Field::Query(QueryField { definition_id, .. }) | Field::Extra(ExtraField { definition_id, .. }) => {
                    type_conditions.push(self.schema.walk(*definition_id).as_ref().parent_entity_id.into())
                }
            }
        }
        type_conditions.sort_unstable();
        let mut single_object_ids = Vec::new();
        let mut other_type_conditions = Vec::new();
        for type_condition in type_conditions.into_iter().dedup() {
            match type_condition {
                SelectionSetType::Object(id) => single_object_ids.push(id),
                SelectionSetType::Interface(id) => {
                    other_type_conditions.push(self.schema[id].possible_type_ids.as_slice())
                }
                SelectionSetType::Union(id) => other_type_conditions.push(self.schema[id].possible_type_ids.as_slice()),
            }
        }

        partition_shapes(output, single_object_ids, other_type_conditions)
    }

    fn create_shape_for<'a>(
        &mut self,
        exemplar_object_id: ObjectDefinitionId,
        maybe_response_object_set_id: Option<ResponseObjectSetId>,
        fields_sorted_by_response_key_then_position: &'a [FieldWalker<'op>],
        fields_buffer: &mut Vec<&'a FieldWalker<'op>>,
    ) -> ConcreteObjectShapeId {
        let schema = self.schema;
        tracing::trace!("Creating shape for exemplar {}", schema.walk(exemplar_object_id).name());

        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut typename_response_keys = Vec::new();

        let mut start = 0;
        while let Some(response_key) = fields_sorted_by_response_key_then_position
            .get(start)
            .map(|field| field.response_key())
        {
            let mut end = start + 1;
            while fields_sorted_by_response_key_then_position
                .get(end)
                .map(|field| field.response_key() == response_key)
                .unwrap_or_default()
            {
                end += 1;
            }
            fields_buffer.clear();
            for field in &fields_sorted_by_response_key_then_position[start..end] {
                if is_field_of(schema, field, exemplar_object_id) {
                    fields_buffer.push(field);
                }
            }
            if let Some(field) = fields_buffer.first() {
                tracing::trace!(
                    "Creating shape for {}.{}",
                    schema.walk(exemplar_object_id).name(),
                    field.name()
                );
                if let Some(definition) = field.definition() {
                    field_shapes_buffer.push((
                        self.create_field_shape(response_key, definition, fields_buffer),
                        fields_buffer.iter().map(|field| field.id()).collect(),
                    ));
                } else {
                    typename_response_keys.push(field.response_edge());
                }
            }
            start = end;
        }

        let shape = ConcreteObjectShape {
            set_id: maybe_response_object_set_id,
            identifier: ObjectIdentifier::Anonymous,
            typename_response_edges: typename_response_keys,
            field_shape_ids: {
                let start = self.blueprint.shapes.fields.len();
                let n = field_shapes_buffer.len();
                let keys = &self.operation.response_keys;
                field_shapes_buffer.sort_unstable_by(|a, b| keys[a.0.expected_key].cmp(&keys[b.0.expected_key]));
                for (i, (_, field_ids)) in field_shapes_buffer.iter().enumerate() {
                    let field_shape_id = FieldShapeId::from(start + i);
                    for field_id in field_ids {
                        self.field_id_to_field_shape_ids_builder
                            .push((*field_id, field_shape_id));
                    }
                }
                self.blueprint
                    .shapes
                    .fields
                    .extend(&mut field_shapes_buffer.drain(..).map(|(field_shape, _)| field_shape));
                IdRange::from_start_and_length((start, n))
            },
        };
        self.field_shapes_buffer_pool.push(field_shapes_buffer);
        self.push_concrete_shape(shape)
    }

    fn create_field_shape(
        &mut self,
        response_key: ResponseKey,
        definition: FieldDefinition<'_>,
        fields: &[&FieldWalker<'_>],
    ) -> FieldShape {
        let field = fields
            .iter()
            .min_by_key(|field| field.response_edge())
            .expect("At least one field");
        let ty = definition.ty();
        FieldShape {
            expected_key: self.operation.response_keys.ensure_safety(response_key),
            edge: field.response_edge(),
            id: field.id(),
            required_field_id: fields
                .iter()
                .find_map(|field| self.plan.field_to_solved_requirement[usize::from(field.id())]),
            definition_id: definition.id(),
            shape: match ty.definition().scalar_type() {
                Some(scalar) => Shape::Scalar(scalar),
                None => self.create_object_shape(
                    SelectionSetType::maybe_from(ty.as_ref().definition_id).unwrap(),
                    fields.iter().map(|field| field.selection_set().unwrap().id()).collect(),
                ),
            },
            wrapping: ty.wrapping,
        }
    }

    fn push_concrete_shape(&mut self, shape: ConcreteObjectShape) -> ConcreteObjectShapeId {
        let id = self.blueprint.shapes.concrete.len().into();
        self.blueprint.shapes.concrete.push(shape);
        id
    }

    fn push_polymorphic_shape(&mut self, mut shape: PolymorphicObjectShape) -> PolymorphicObjectShapeId {
        let id = self.blueprint.shapes.polymorphic.len().into();
        let schema = self.schema;
        shape.possibilities.sort_unstable_by(|a, b| {
            let a = &schema[schema[a.0].name_id];
            let b = &schema[schema[b.0].name_id];
            a.cmp(b)
        });
        self.blueprint.shapes.polymorphic.push(shape);
        id
    }
}

fn is_field_of(schema: &Schema, field: &FieldWalker<'_>, object_id: ObjectDefinitionId) -> bool {
    match field.as_ref() {
        Field::TypeName(TypeNameField { type_condition, .. }) => match type_condition {
            SelectionSetType::Object(id) => *id == object_id,
            SelectionSetType::Interface(id) => schema[*id].possible_type_ids.binary_search(&object_id).is_ok(),
            SelectionSetType::Union(id) => schema[*id].possible_type_ids.binary_search(&object_id).is_ok(),
        },
        Field::Query(QueryField { definition_id, .. }) | Field::Extra(ExtraField { definition_id, .. }) => {
            match schema[*definition_id].parent_entity_id {
                EntityDefinitionId::Object(id) => id == object_id,
                EntityDefinitionId::Interface(id) => schema[id].possible_type_ids.binary_search(&object_id).is_ok(),
            }
        }
    }
}
