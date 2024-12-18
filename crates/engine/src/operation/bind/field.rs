use super::{coercion::coerce_query_value, BindError, BindResult, Binder, BoundFieldId, QueryPosition};
use crate::{
    operation::{
        BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundQueryField, BoundSelectionSetId, BoundTypeNameField,
        QueryInputValueRecord, QueryModifierRule,
    },
    response::ResponseKey,
};
use cynic_parser::{
    executable::{Argument, FieldSelection, Iter},
    Span,
};
use id_newtypes::IdRange;
use schema::{CompositeTypeId, DefinitionId, FieldDefinition, FieldDefinitionId};

impl<'schema, 'p> Binder<'schema, 'p> {
    pub(super) fn bind_typename_field(
        &mut self,
        type_condition: CompositeTypeId,
        query_position: QueryPosition,
        key: ResponseKey,
        field: FieldSelection<'p>,
    ) -> BindResult<BoundFieldId> {
        Ok(self.push_field(BoundField::TypeName(BoundTypeNameField {
            type_condition,
            query_position,
            key,
            location: self.parsed_operation.span_to_location(field.name_span()),
        })))
    }

    pub(super) fn bind_field(
        &mut self,
        query_position: QueryPosition,
        key: ResponseKey,
        definition_id: FieldDefinitionId,
        field: FieldSelection<'p>,
        selection_set_id: Option<BoundSelectionSetId>,
        executable_directive_rules: Vec<QueryModifierRule>,
    ) -> BindResult<BoundFieldId> {
        let location = field.name_span();
        let definition: FieldDefinition<'_> = self.schema.walk(definition_id);

        // We don't bother processing the selection set if it's not a union/interface/object, so we
        // need to rely on the parsed data rather than selection_set_id.
        let has_selection_set = field.selection_set().len() != 0;
        match definition.ty().as_ref().definition_id {
            DefinitionId::Scalar(_) | DefinitionId::Enum(_) if has_selection_set => {
                return Err(BindError::CannotHaveSelectionSet {
                    name: definition.name().to_string(),
                    ty: definition.ty().to_string(),
                    span: location,
                })
            }
            DefinitionId::Object(_) | DefinitionId::Interface(_) | DefinitionId::Union(_) if !has_selection_set => {
                return Err(BindError::LeafMustBeAScalarOrEnum {
                    name: definition.name().to_string(),
                    ty: definition.ty().definition().name().to_string(),
                    span: location,
                });
            }
            _ => {}
        };

        let field_id = BoundFieldId::from(self.fields.len());
        let argument_ids = self.bind_field_arguments(definition, location, field.arguments())?;
        self.fields.push(BoundField::Query(BoundQueryField {
            query_position,
            key,
            subgraph_key: key,
            location: self.parsed_operation.span_to_location(location),
            definition_id: definition.id,
            argument_ids,
            selection_set_id,
        }));

        self.generate_field_modifiers(field_id, argument_ids, definition, executable_directive_rules);
        Ok(field_id)
    }

    pub(super) fn push_field(&mut self, field: BoundField) -> BoundFieldId {
        let id = BoundFieldId::from(self.fields.len());
        self.fields.push(field);
        id
    }

    fn bind_field_arguments(
        &mut self,
        definition: FieldDefinition<'schema>,
        span: Span,
        arguments: Iter<'p, Argument<'p>>,
    ) -> BindResult<IdRange<BoundFieldArgumentId>> {
        // Avoid binding multiple times the same arguments (same fragments used at different places)
        if let Some(ids) = self.location_to_field_arguments.get(&span.start) {
            return Ok(*ids);
        }

        let mut arguments = arguments.collect::<Vec<_>>();

        let start = self.field_arguments.len();
        for argument_def in definition.arguments() {
            if argument_def.is_inaccessible() {
                continue;
            }
            if let Some(index) = arguments
                .iter()
                .position(|argument| argument.name() == argument_def.name())
            {
                let argument = arguments.swap_remove(index);
                let value = argument.value();
                let input_value_id = coerce_query_value(self, argument_def.ty(), value)?;
                self.field_arguments.push(BoundFieldArgument {
                    input_value_definition_id: argument_def.id,
                    input_value_id,
                });
            } else if let Some(id) = argument_def.as_ref().default_value_id {
                self.field_arguments.push(BoundFieldArgument {
                    input_value_definition_id: argument_def.id,
                    input_value_id: self.input_values.push_value(QueryInputValueRecord::DefaultValue(id)),
                });
            } else if argument_def.ty().wrapping.is_required() {
                return Err(BindError::MissingArgument {
                    field: definition.name().to_string(),
                    name: argument_def.name().to_string(),
                    span,
                });
            }
        }

        if let Some(first_unknown_argument) = arguments.first() {
            return Err(BindError::UnknownArgument {
                field_name: format!("{}.{}", definition.parent_entity().name(), definition.name()),
                argument_name: first_unknown_argument.name().to_string(),
                span: first_unknown_argument.name_span(),
            });
        }

        let end = self.field_arguments.len();
        Ok((start..end).into())
    }
}
