use cynic_parser::{
    Span,
    common::OperationType,
    executable::{Argument, Directive, FieldSelection, Iter, Selection},
};
use id_newtypes::IdRange;
use schema::{CompositeType, FieldDefinition, ObjectDefinitionId, TypeDefinition, TypeRecord, Wrapping};
use std::collections::HashSet;
use walker::Walk;

use crate::{
    ExecutableDirectiveId, FieldArgumentId, IncludeDirectiveRecord, InlineFragmentId, InlineFragmentRecord,
    QueryInputValueRecord, SelectionSetRecord, SkipDirectiveRecord, VariableDefinitionRecord,
};

use super::{
    BindError, BindResult, OperationBinder,
    coercion::{coerce_query_value, coerce_variable_default_value},
};

impl<'schema, 'p> OperationBinder<'schema, 'p> {
    pub(super) fn bind_root(&mut self) -> BindResult<(ObjectDefinitionId, SelectionSetRecord)> {
        let operation = self.parsed_operation.operation();
        let root_object_id = match operation.operation_type() {
            OperationType::Query => self.schema.query().id,
            OperationType::Mutation => self.schema.mutation().ok_or(BindError::NoMutationDefined)?.id,
            OperationType::Subscription => self.schema.subscription().ok_or(BindError::NoSubscriptionDefined)?.id,
        };
        // Must be executed before binding selection sets
        self.bind_variable_definitions(operation.variable_definitions())?;
        let root_selection_set_record = self.bind_selection_set(
            CompositeType::Object(root_object_id.walk(self.schema)),
            operation.selection_set(),
        )?;

        // If there are no errors, we can check if all variables are used.
        if self.errors.is_empty() {
            self.validate_all_variables_used()?;
        }

        Ok((root_object_id, root_selection_set_record))
    }

    fn bind_selection_set(
        &mut self,
        parent_output_type: CompositeType<'schema>,
        selection_set: Iter<'p, Selection<'p>>,
    ) -> BindResult<crate::SelectionSetRecord> {
        let mut buffer = self.selection_buffers.pop().unwrap_or_default();
        for selection in selection_set {
            let id = match selection {
                Selection::Field(field) => if field.name() == "__typename" {
                    match self.bind_typename_field(field) {
                        Ok(field_id) => crate::FieldId::from(field_id),
                        Err(err) => {
                            self.errors.push(err);
                            continue;
                        }
                    }
                } else {
                    match self.bind_field(parent_output_type, field) {
                        Ok(field_id) => crate::FieldId::from(field_id),
                        Err(err) => {
                            // We can continue if there nested errors.
                            self.errors.push(err);
                            continue;
                        }
                    }
                }
                .into(),
                Selection::FragmentSpread(spread) => match self.bind_fragment_spread(parent_output_type, spread) {
                    Ok(spread_id) => spread_id.into(),
                    Err(err) => {
                        self.errors.push(err);
                        continue;
                    }
                },
                Selection::InlineFragment(fragment) => match self.bind_inline_fragment(parent_output_type, fragment) {
                    Ok(fragment_id) => fragment_id.into(),
                    Err(err) => {
                        self.errors.push(err);
                        continue;
                    }
                },
            };
            buffer.push(id);
        }

        let start = self.shared_selection_ids.len();
        self.shared_selection_ids.append(&mut buffer);
        self.selection_buffers.push(buffer);
        Ok((start..self.shared_selection_ids.len()).into())
    }

    fn bind_typename_field(&mut self, field: FieldSelection<'p>) -> BindResult<crate::TypenameFieldId> {
        let directive_ids = self.bind_executable_directive(field.directives());
        let response_key = self.response_keys.get_or_intern(field.alias().unwrap_or(field.name()));
        self.typename_fields.push(crate::TypenameFieldRecord {
            response_key,
            location: self.parsed_operation.span_to_location(field.name_span()),
            directive_ids,
        });
        Ok((self.typename_fields.len() - 1).into())
    }

    fn bind_field(
        &mut self,
        parent_output_type: CompositeType<'schema>,
        field: FieldSelection<'p>,
    ) -> BindResult<crate::DataFieldId> {
        let definition = match parent_output_type {
            CompositeType::Object(object) => object.find_field_by_name(field.name()),
            CompositeType::Interface(interface) => interface.find_field_by_name(field.name()),
            CompositeType::Union(union) => {
                return Err(BindError::UnionHaveNoFields {
                    name: field.name().to_string(),
                    ty: union.name().to_string(),
                    span: field.name_span(),
                    site_id: union.id,
                });
            }
        }
        .filter(|field_definition| !field_definition.is_inaccessible())
        .ok_or_else(|| BindError::UnknownField {
            ty: parent_output_type.name().to_string(),
            name: field.name().to_string(),
            span: field.name_span(),
            site_id: parent_output_type.as_entity().unwrap().id(),
        })?;

        let selection_set_record = match definition.ty().definition().as_composite_type() {
            Some(output_type) => {
                let error_count = self.errors.len();
                let selection_set_record = self.bind_selection_set(output_type, field.selection_set())?;
                // If empty and we didn't have any errors in this selection set.
                if selection_set_record.is_empty() && error_count == self.errors.len() {
                    return Err(BindError::LeafMustBeAScalarOrEnum {
                        name: definition.name().to_string(),
                        ty: definition.ty().definition().name().to_string(),
                        span: field.name_span(),
                        site_id: output_type.id(),
                    });
                }
                selection_set_record
            }
            None => {
                if field.selection_set().len() > 0 {
                    return Err(BindError::CannotHaveSelectionSet {
                        name: definition.name().to_string(),
                        ty: definition.ty().to_string(),
                        span: field.name_span(),
                        site_id: definition.ty().definition_id,
                    });
                }
                crate::SelectionSetRecord::empty()
            }
        };

        let argument_ids = self.bind_field_arguments(definition, field.name_span(), field.arguments());
        let directive_ids = self.bind_executable_directive(field.directives());
        let response_key = self.response_keys.get_or_intern(field.alias().unwrap_or(field.name()));

        self.data_fields.push(crate::DataFieldRecord {
            definition_id: definition.id,
            directive_ids,
            response_key,
            location: self.parsed_operation.span_to_location(field.name_span()),
            argument_ids,
            selection_set_record,
        });

        Ok((self.data_fields.len() - 1).into())
    }

    fn bind_field_arguments(
        &mut self,
        definition: FieldDefinition<'schema>,
        span: Span,
        arguments: Iter<'p, Argument<'p>>,
    ) -> IdRange<FieldArgumentId> {
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
                let value_id = coerce_query_value(self, argument_def.ty(), value);
                self.field_arguments.push(crate::FieldArgumentRecord {
                    definition_id: argument_def.id,
                    value_id,
                });
            } else if let Some(id) = argument_def.as_ref().default_value_id {
                self.field_arguments.push(crate::FieldArgumentRecord {
                    definition_id: argument_def.id,
                    value_id: self
                        .query_input_values
                        .push_value(QueryInputValueRecord::DefaultValue(id)),
                });
            } else if argument_def.ty().wrapping.is_required() {
                self.errors.push(BindError::MissingArgument {
                    field: definition.name().to_string(),
                    name: argument_def.name().to_string(),
                    span,
                    site_id: definition.id,
                });
            }
        }

        if let Some(first_unknown_argument) = arguments.first() {
            self.errors.push(BindError::UnknownArgument {
                field_name: format!("{}.{}", definition.parent_entity().name(), definition.name()),
                argument_name: first_unknown_argument.name().to_string(),
                span: first_unknown_argument.name_span(),
                site_id: definition.id,
            });
        }

        let end = self.field_arguments.len();
        // We iterate over input fields in order which is a range, so it should be sorted by the
        // id.
        debug_assert!(self.field_arguments[start..end].is_sorted_by_key(|arg| arg.definition_id));
        (start..end).into()
    }

    fn bind_inline_fragment(
        &mut self,
        parent_output_type: CompositeType<'schema>,
        fragment: cynic_parser::executable::InlineFragment<'p>,
    ) -> BindResult<InlineFragmentId> {
        let type_condition = fragment
            .type_condition()
            .map(|name| self.bind_type_condition(parent_output_type, name, fragment.type_condition_span().unwrap()))
            .transpose()?;
        let selection_set_record =
            self.bind_selection_set(type_condition.unwrap_or(parent_output_type), fragment.selection_set())?;
        let directive_ids = self.bind_executable_directive(fragment.directives());

        self.inline_fragments.push(InlineFragmentRecord {
            type_condition_id: type_condition.map(|ty| ty.id()),
            directive_ids,
            selection_set_record,
        });

        Ok((self.inline_fragments.len() - 1).into())
    }

    fn bind_fragment_spread(
        &mut self,
        parent_output_type: CompositeType<'schema>,
        spread: cynic_parser::executable::FragmentSpread<'p>,
    ) -> BindResult<crate::FragmentSpreadId> {
        let fragment_id = match self.fragment_name_to_id.get(spread.fragment_name()) {
            Some(&id) => {
                let ty = self[id].type_condition_id.walk(self.schema);
                if !parent_output_type.has_non_empty_intersection_with(ty) {
                    return Err(BindError::DisjointTypeCondition {
                        parent: parent_output_type.name().to_string(),
                        name: ty.name().to_string(),
                        span: spread.fragment_name_span(),
                        site_id: parent_output_type.id(),
                    });
                }
                id
            }
            None => {
                let fragment = spread.fragment().ok_or_else(|| BindError::UnknownFragment {
                    name: spread.fragment_name().to_string(),
                    span: spread.fragment_name_span(),
                })?;
                let id = self.bind_fragment(parent_output_type, fragment)?;
                self.fragment_name_to_id.insert(spread.fragment_name(), id);
                id
            }
        };
        let directive_ids = self.bind_executable_directive(spread.directives());
        self.fragment_spreads.push(crate::FragmentSpreadRecord {
            fragment_id,
            directive_ids,
        });
        Ok((self.fragment_spreads.len() - 1).into())
    }

    fn bind_fragment(
        &mut self,
        parent_output_type: CompositeType<'schema>,
        fragment: cynic_parser::executable::FragmentDefinition<'p>,
    ) -> BindResult<crate::FragmentId> {
        let type_condition = self.bind_type_condition(
            parent_output_type,
            fragment.type_condition(),
            fragment.type_condition_span(),
        )?;
        let selection_set_record = self.bind_selection_set(type_condition, fragment.selection_set())?;

        self.fragments.push(crate::FragmentRecord {
            type_condition_id: type_condition.id(),
            selection_set_record,
        });

        Ok((self.fragments.len() - 1).into())
    }

    fn bind_type_condition(
        &self,
        parent_output_type: CompositeType<'schema>,
        name: &'p str,
        span: Span,
    ) -> BindResult<CompositeType<'schema>> {
        let definition = self
            .schema
            .type_definition_by_name(name)
            .filter(|def| !def.is_inaccessible())
            .ok_or_else(|| BindError::UnknownType {
                name: name.to_string(),
                span,
            })?;
        let type_condition =
            definition
                .as_composite_type()
                .ok_or_else(|| BindError::InvalidTypeConditionTargetType {
                    name: name.to_string(),
                    span,
                    site_id: definition.id(),
                })?;

        if parent_output_type.has_non_empty_intersection_with(type_condition) {
            return Ok(type_condition);
        }

        Err(BindError::DisjointTypeCondition {
            parent: parent_output_type.name().to_string(),
            name: name.to_string(),
            span,
            site_id: parent_output_type.id(),
        })
    }

    fn bind_executable_directive(&mut self, directives: Iter<'p, Directive<'p>>) -> Vec<ExecutableDirectiveId> {
        let mut out = Vec::new();
        for directive in directives {
            if matches!(directive.name(), "skip" | "include") {
                match self.bind_skip_or_include_executable_directive(directive) {
                    Ok(directive_id) => out.push(directive_id),
                    Err(err) => {
                        self.errors.push(err);
                        continue;
                    }
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    fn bind_skip_or_include_executable_directive(
        &mut self,
        directive: Directive<'p>,
    ) -> BindResult<ExecutableDirectiveId> {
        let argument = directive
            .arguments()
            .next()
            .ok_or(BindError::MissingDirectiveArgument {
                name: "if",
                span: directive.name_span(),
                directive: directive.name().to_string(),
            })?;

        let ty = TypeRecord {
            definition_id: self.schema.type_definition_by_name("Boolean").expect("must exist").id(),
            wrapping: schema::Wrapping::required(),
        }
        .walk(self.schema);

        let condition = coerce_query_value(self, ty, argument.value());

        Ok(if directive.name() == "skip" {
            ExecutableDirectiveId::Skip(SkipDirectiveRecord { condition })
        } else {
            ExecutableDirectiveId::Include(IncludeDirectiveRecord { condition })
        })
    }

    fn bind_variable_definitions(
        &mut self,
        variables: cynic_parser::executable::Iter<'_, cynic_parser::executable::VariableDefinition<'_>>,
    ) -> BindResult<()> {
        let mut seen_names = HashSet::new();

        for variable in variables {
            let name = variable.name().to_string();
            let name_location = self.parsed_operation.span_to_location(variable.name_span());

            if seen_names.contains(&name) {
                return Err(BindError::DuplicateVariable {
                    name,
                    location: name_location,
                });
            }
            seen_names.insert(name.clone());

            let mut ty = match self.convert_type(&name, variable.ty()) {
                Ok(ty) => ty,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };

            match variable.default_value() {
                Some(value) if !value.is_null() => {
                    if ty.wrapping.is_list() {
                        ty.wrapping = ty.wrapping.wrap_list_non_null();
                    } else {
                        ty.wrapping = Wrapping::required()
                    }
                }
                _ => (),
            }

            let ty = ty.walk(self.schema);
            let default_value_id = variable
                .default_value()
                .map(|value| coerce_variable_default_value(self, ty, value));

            self.variable_definition_in_use.push(false);
            self.variable_definitions.push(VariableDefinitionRecord {
                name,
                name_location,
                default_value_id,
                ty_record: ty.into(),
            });
        }

        Ok(())
    }

    fn validate_all_variables_used(&self) -> BindResult<()> {
        for (variable, in_use) in self.variable_definitions.iter().zip(&self.variable_definition_in_use) {
            if !in_use {
                return Err(BindError::UnusedVariable {
                    name: variable.name.clone(),
                    operation: self.error_operation_name.clone(),
                    location: variable.name_location,
                });
            }
        }

        Ok(())
    }

    fn convert_type(
        &self,
        variable_name: &str,
        ty: cynic_parser::executable::Type<'_>,
    ) -> BindResult<schema::TypeRecord> {
        use cynic_parser::common::WrappingType;

        let location = ty.span();

        let definition = self
            .schema
            .type_definition_by_name(ty.name())
            .ok_or_else(|| BindError::UnknownType {
                name: ty.name().to_string(),
                span: location,
            })?;

        if !matches!(
            definition,
            TypeDefinition::Enum(_) | TypeDefinition::Scalar(_) | TypeDefinition::InputObject(_)
        ) {
            return Err(BindError::InvalidVariableType {
                name: variable_name.to_string(),
                ty: definition.name().to_string(),
                span: location,
            });
        }

        let mut wrapping = schema::Wrapping::default();
        let wrappers = ty.wrappers().collect::<Vec<_>>();
        // from innermost to outermost
        for wrapper in wrappers.into_iter().rev() {
            wrapping = match wrapper {
                WrappingType::NonNull => wrapping.wrap_non_null(),
                WrappingType::List => wrapping.wrap_list(),
            };
        }

        Ok(schema::TypeRecord {
            definition_id: definition.id(),
            wrapping,
        })
    }
}
