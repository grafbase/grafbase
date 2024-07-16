use engine_parser::Positioned;
use engine_value::Name;
use id_newtypes::IdRange;
use schema::{Definition, FieldDefinitionId, FieldDefinitionWalker};

use super::{coercion::coerce_query_value, BindError, BindResult, Binder};
use crate::{
    operation::{
        Field, FieldArgument, FieldArgumentId, FieldId, Location, QueryField, QueryInputValue, SelectionSetId,
        SelectionSetType, TypeNameField,
    },
    response::BoundResponseKey,
};

impl<'schema, 'p> Binder<'schema, 'p> {
    pub(super) fn bind_typename_field(
        &mut self,
        parent_selection_set_id: SelectionSetId,
        type_condition: SelectionSetType,
        bound_response_key: BoundResponseKey,
        Positioned { pos, .. }: &'p Positioned<engine_parser::types::Field>,
    ) -> BindResult<FieldId> {
        Ok(self.push_field(Field::TypeName(TypeNameField {
            type_condition,
            bound_response_key,
            location: (*pos).try_into()?,
            parent_selection_set_id,
        })))
    }

    pub(super) fn bind_field(
        &mut self,
        parent_selection_set_id: SelectionSetId,
        bound_response_key: BoundResponseKey,
        definition_id: FieldDefinitionId,
        Positioned { pos, node: field }: &'p Positioned<engine_parser::types::Field>,
        selection_set_id: Option<SelectionSetId>,
    ) -> BindResult<FieldId> {
        let location: Location = (*pos).try_into()?;
        let definition: FieldDefinitionWalker<'_> = self.schema.walk(definition_id);

        // We don't bother processing the selection set if it's not a union/interface/object, so we
        // need to rely on the parsed data rather than selection_set_id.
        let has_selection_set = !field.selection_set.node.items.is_empty();
        match definition.ty().inner().id() {
            Definition::Scalar(_) | Definition::Enum(_) if has_selection_set => {
                return Err(BindError::CannotHaveSelectionSet {
                    name: definition.name().to_string(),
                    ty: definition.ty().to_string(),
                    location,
                })
            }
            Definition::Object(_) | Definition::Interface(_) | Definition::Union(_) if !has_selection_set => {
                return Err(BindError::LeafMustBeAScalarOrEnum {
                    name: definition.name().to_string(),
                    ty: definition.ty().inner().name().to_string(),
                    location,
                });
            }
            _ => {}
        };

        let field_id = FieldId::from(self.fields.len());
        let argument_ids = self.bind_field_arguments(definition, field_id, location, &field.arguments)?;
        let subject_to_response_modifier_rules = self.generate_field_modifiers(field_id, argument_ids, definition);
        self.fields.push(Field::Query(QueryField {
            bound_response_key,
            location,
            definition_id: definition.id(),
            argument_ids,
            selection_set_id,
            parent_selection_set_id,
            subject_to_response_modifier_rules,
        }));

        Ok(field_id)
    }

    pub(super) fn push_field(&mut self, field: Field) -> FieldId {
        let id = FieldId::from(self.fields.len());
        self.fields.push(field);
        id
    }

    fn bind_field_arguments(
        &mut self,
        definition: FieldDefinitionWalker<'_>,
        field_id: FieldId,
        location: Location,
        arguments: &[(Positioned<Name>, Positioned<engine_value::Value>)],
    ) -> BindResult<IdRange<FieldArgumentId>> {
        // Avoid binding multiple times the same arguments (same fragments used at different places)
        if let Some(ids) = self.location_to_field_arguments.get(&location) {
            return Ok(*ids);
        }

        let mut arguments = arguments.to_vec();

        let start = self.field_arguments.len();
        for argument_def in definition.arguments() {
            if let Some(index) = arguments
                .iter()
                .position(|(Positioned { node: name, .. }, _)| name.as_str() == argument_def.name())
            {
                let (name, value) = arguments.swap_remove(index);
                let name_location = Some(name.pos.try_into()?);
                let value_location = value.pos.try_into()?;
                let value = value.node;
                let input_value_id =
                    coerce_query_value(self, field_id, value_location, argument_def.ty().into(), value)?;
                self.field_arguments.push(FieldArgument {
                    name_location,
                    value_location: Some(value_location),
                    input_value_definition_id: argument_def.id(),
                    input_value_id,
                });
            } else if let Some(id) = argument_def.as_ref().default_value {
                self.field_arguments.push(FieldArgument {
                    name_location: None,
                    value_location: None,
                    input_value_definition_id: argument_def.id(),
                    input_value_id: self.input_values.push_value(QueryInputValue::DefaultValue(id)),
                });
            } else if argument_def.ty().wrapping().is_required() {
                return Err(BindError::MissingArgument {
                    field: definition.name().to_string(),
                    name: argument_def.name().to_string(),
                    location,
                });
            }
        }
        let end = self.field_arguments.len();
        Ok((start..end).into())
    }
}
