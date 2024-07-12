use id_newtypes::IdRange;
use im::HashMap;
use itertools::Itertools;
use schema::{EntityId, FieldDefinitionWalker, ObjectId, Schema};

use crate::{
    execution::PlanInput,
    operation::{
        ConditionResult, ExtraField, Field, FieldId, FieldWalker, LogicalPlanId, QueryField, SelectionSetId,
        SelectionSetType, TypeNameField,
    },
    response::{
        ConcreteObjectShape, ConcreteObjectShapeId, FieldError, FieldShape, ObjectIdentifier, PolymorphicObjectShape,
        PolymorphicObjectShapeId, ResponseKey, ResponseObjectSetId, Shape,
    },
    Runtime,
};

use super::{
    partition::{partition_shapes, Partition},
    ExecutionPlanBuilder, PlanningResult,
};

impl<'ctx, 'op, 'planner, R: Runtime> ExecutionPlanBuilder<'ctx, 'op, 'planner, R>
where
    'ctx: 'op,
{
    pub(super) fn create_root_shape_for(
        &mut self,
        input: &PlanInput,
        root_field_ids: &[FieldId],
    ) -> PlanningResult<ConcreteObjectShapeId> {
        let exemplar = match input.entity_id {
            EntityId::Object(id) => id,
            EntityId::Interface(id) => self.schema()[id].possible_types[0],
        };

        let walker = self.walker();
        let mut root_fields = root_field_ids.iter().map(|id| walker.walk(*id)).collect::<Vec<_>>();
        root_fields.sort_unstable_by(|left, right| {
            left.response_key()
                .cmp(&right.response_key())
                .then(left.as_ref().query_position().cmp(&right.as_ref().query_position()))
        });
        let shape_id = self.create_shape_for(exemplar, None, &root_fields, &mut Vec::new())?;

        Ok(shape_id)
    }

    fn create_object_shape(
        &mut self,
        ty: SelectionSetType,
        merged_selection_set_ids: Vec<SelectionSetId>,
    ) -> PlanningResult<Shape> {
        let mut plan_field_ids: Vec<FieldId> = Vec::new();
        let mut children_plan: HashMap<LogicalPlanId, Vec<FieldId>> = HashMap::new();
        for id in &merged_selection_set_ids {
            for field in self.walker().walk(*id).fields() {
                let plan_id = self.operation.plan_id_for(field.id());
                if plan_id == self.logical_plan_id {
                    plan_field_ids.push(field.id());
                } else {
                    children_plan.entry(plan_id).or_default().push(field.id());
                }
            }
        }
        let maybe_response_object_set_id = if children_plan.is_empty() {
            None
        } else {
            let id = self.next_response_object_set_id();
            self.tracked_output_ids.push(id);
            let to_be_planned = children_plan
                .into_iter()
                .map(|(plan_id, root_fields)| super::ToBePlanned {
                    selection_set_ty: ty,
                    input_id: id,
                    logical_plan_id: plan_id,
                    root_fields,
                })
                .collect::<Vec<_>>();
            self.to_be_planned.extend(to_be_planned);
            Some(id)
        };

        let shape = self.collect_object_shapes(ty, maybe_response_object_set_id, plan_field_ids)?;
        match shape {
            Shape::Scalar(_) => {}
            Shape::ConcreteObject(id) => {
                if matches!(
                    self.plans.shapes[id].identifier,
                    ObjectIdentifier::UnionTypename(_) | ObjectIdentifier::InterfaceTypename(_)
                ) {
                    self.requires_typename_for.push(merged_selection_set_ids[0]);
                }
            }
            Shape::PolymorphicObject(_) => {
                self.requires_typename_for.push(merged_selection_set_ids[0]);
            }
        }
        Ok(shape)
    }

    fn collect_object_shapes(
        &mut self,
        ty: SelectionSetType,
        maybe_response_object_set_id: Option<ResponseObjectSetId>,
        field_ids: Vec<FieldId>,
    ) -> PlanningResult<Shape> {
        let output: &[ObjectId] = match &ty {
            SelectionSetType::Object(id) => std::array::from_ref(id),
            SelectionSetType::Interface(id) => &self.schema()[*id].possible_types,
            SelectionSetType::Union(id) => &self.schema()[*id].possible_types,
        };
        let shape_partitions = self.compute_shape_partitions(output, &field_ids);

        let walker = self.walker();
        let mut fields = field_ids.into_iter().map(|id| walker.walk(id)).collect::<Vec<_>>();
        fields.sort_unstable_by(|left, right| {
            left.response_key()
                .cmp(&right.response_key())
                .then(left.as_ref().query_position().cmp(&right.as_ref().query_position()))
        });
        let mut buffer = Vec::new();

        if let Some(partitions) = shape_partitions {
            let mut shapes = Vec::new();
            for partition in partitions {
                match partition {
                    Partition::One(id) => {
                        let shape_id = self.create_shape_for(id, maybe_response_object_set_id, &fields, &mut buffer)?;
                        shapes.push((id, shape_id))
                    }
                    Partition::Many(ids) => {
                        let shape_id =
                            self.create_shape_for(ids[0], maybe_response_object_set_id, &fields, &mut buffer)?;
                        shapes.extend(ids.into_iter().map(|id| (id, shape_id)))
                    }
                }
            }
            Ok(Shape::PolymorphicObject(
                self.push_polymorphic_shape(PolymorphicObjectShape { shapes }),
            ))
        } else {
            let shape_id = self.create_shape_for(output[0], maybe_response_object_set_id, &fields, &mut buffer)?;
            let shape = &mut self.plans.shapes[shape_id];
            if output.len() == 1 {
                shape.identifier = ObjectIdentifier::Known(output[0]);
            } else if shape.set_id.is_some() || !shape.typename_response_edges.is_empty() {
                shape.identifier = match ty {
                    SelectionSetType::Interface(id) => ObjectIdentifier::InterfaceTypename(id),
                    SelectionSetType::Union(id) => ObjectIdentifier::UnionTypename(id),
                    SelectionSetType::Object(_) => unreachable!(),
                }
            }
            Ok(Shape::ConcreteObject(shape_id))
        }
    }

    fn compute_shape_partitions(&self, output: &[ObjectId], field_ids: &[FieldId]) -> Option<Vec<Partition<ObjectId>>> {
        let mut type_conditions = Vec::new();
        for field_id in field_ids {
            match &self.operation[*field_id] {
                Field::TypeName(TypeNameField { type_condition, .. }) => type_conditions.push(*type_condition),
                Field::Query(QueryField { definition_id, .. }) | Field::Extra(ExtraField { definition_id, .. }) => {
                    type_conditions.push(self.schema().walk(*definition_id).parent_entity().id().into())
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
                    other_type_conditions.push(self.schema()[id].possible_types.as_slice())
                }
                SelectionSetType::Union(id) => other_type_conditions.push(self.schema()[id].possible_types.as_slice()),
            }
        }

        partition_shapes(output, single_object_ids, other_type_conditions)
    }

    fn create_shape_for<'a>(
        &mut self,
        exemplar_object_id: ObjectId,
        maybe_response_object_set_id: Option<ResponseObjectSetId>,
        fields_sorted_by_response_key_then_position: &'a [FieldWalker<'op>],
        buffer: &mut Vec<&'a FieldWalker<'op>>,
    ) -> PlanningResult<ConcreteObjectShapeId> {
        let schema = self.schema();
        tracing::trace!("Creating shape for exemplar {}", schema.walk(exemplar_object_id).name());

        let mut field_shapes_buffer = self.field_shapes_buffer_pool.pop();
        let mut field_errors_buffer = self.field_errors_buffer_pool.pop();
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
            buffer.clear();
            for field in &fields_sorted_by_response_key_then_position[start..end] {
                if type_condition_applies(schema, field, exemplar_object_id) {
                    buffer.push(field);
                }
            }
            if let Some(field) = buffer.first() {
                tracing::trace!(
                    "Creating shape for {}.{}",
                    schema.walk(exemplar_object_id).name(),
                    field.name()
                );
                if let Some(definition) = field.definition() {
                    match field
                        .as_ref()
                        .condition()
                        .map(|id| &self.condition_results[usize::from(id)])
                    {
                        Some(ConditionResult::Errors(errors)) => {
                            field_errors_buffer.push(FieldError {
                                edge: field.response_edge(),
                                errors: errors.clone(),
                                is_required: definition.ty().wrapping().is_required(),
                            });
                        }
                        Some(ConditionResult::Include) | None => {
                            field_shapes_buffer.push(self.create_field_shape(response_key, definition, buffer)?)
                        }
                    }
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
                let start = self.plans.shapes.fields.len();
                let n = field_shapes_buffer.len();
                let keys = &self.operation.response_keys;
                field_shapes_buffer.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
                self.plans.shapes.fields.extend(&mut field_shapes_buffer.drain(..));
                IdRange::from_start_and_length((start, n))
            },
            field_error_ids: {
                let start = self.plans.shapes.errors.len();
                let n = field_errors_buffer.len();
                self.plans.shapes.errors.extend(&mut field_errors_buffer.drain(..));
                IdRange::from_start_and_length((start, n))
            },
        };
        self.field_shapes_buffer_pool.push(field_shapes_buffer);
        self.field_errors_buffer_pool.push(field_errors_buffer);
        Ok(self.push_concrete_shape(shape))
    }

    fn create_field_shape(
        &mut self,
        response_key: ResponseKey,
        definition: FieldDefinitionWalker<'_>,
        fields: &[&FieldWalker<'_>],
    ) -> PlanningResult<FieldShape> {
        let field = fields
            .iter()
            .min_by_key(|field| field.response_edge())
            .expect("At least one field");
        let ty = definition.ty();
        Ok(FieldShape {
            expected_key: self.operation.response_keys.ensure_safety(response_key),
            edge: field.response_edge(),
            id: field.id(),
            definition_id: definition.id(),
            shape: match ty.inner().scalar_type() {
                Some(scalar) => Shape::Scalar(scalar),
                None => self.create_object_shape(
                    SelectionSetType::maybe_from(ty.inner().id()).unwrap(),
                    fields.iter().map(|field| field.selection_set().unwrap().id()).collect(),
                )?,
            },
            wrapping: ty.wrapping(),
        })
    }

    fn push_concrete_shape(&mut self, shape: ConcreteObjectShape) -> ConcreteObjectShapeId {
        let id = self.plans.shapes.concrete.len().into();
        self.plans.shapes.concrete.push(shape);
        id
    }

    fn push_polymorphic_shape(&mut self, mut shape: PolymorphicObjectShape) -> PolymorphicObjectShapeId {
        let id = self.plans.shapes.polymorphic.len().into();
        let schema = self.schema();
        shape.shapes.sort_unstable_by(|a, b| {
            let a = &schema[schema[a.0].name];
            let b = &schema[schema[b.0].name];
            a.cmp(b)
        });
        self.plans.shapes.polymorphic.push(shape);
        id
    }
}

fn type_condition_applies(schema: &Schema, field: &FieldWalker<'_>, object_id: ObjectId) -> bool {
    match field.as_ref() {
        Field::TypeName(TypeNameField { type_condition, .. }) => match type_condition {
            SelectionSetType::Object(id) => *id == object_id,
            SelectionSetType::Interface(id) => schema[*id].possible_types.binary_search(&object_id).is_ok(),
            SelectionSetType::Union(id) => schema[*id].possible_types.binary_search(&object_id).is_ok(),
        },
        Field::Query(QueryField { definition_id, .. }) | Field::Extra(ExtraField { definition_id, .. }) => {
            match schema[*definition_id].parent_entity {
                EntityId::Object(id) => id == object_id,
                EntityId::Interface(id) => schema[id].possible_types.binary_search(&object_id).is_ok(),
            }
        }
    }
}
