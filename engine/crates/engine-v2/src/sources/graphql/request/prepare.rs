use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::{EntityDefinition, SubgraphId};
use walker::Walk;

use crate::operation::{
    FieldArgumentsWalker, PlanField, PlanSelectionSet, PlanWalker, QueryInputValueId, SelectionSetType,
};

const VARIABLE_PREFIX: &str = "var";

pub(crate) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: QueryVariables,
}

impl PreparedGraphqlOperation {
    pub(crate) fn build(
        operation_type: OperationType,
        plan: PlanWalker<'_>,
        subgraph_id: SubgraphId,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut ctx = QueryBuilderContext::new(subgraph_id);

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = String::with_capacity(256);

            let selection_set_type = SelectionSetType::Object(match operation_type {
                OperationType::Query => plan.schema().query().id(),
                OperationType::Mutation => plan.schema().mutation().unwrap().id(),
                OperationType::Subscription => plan.schema().subscription().unwrap().id(),
            });

            ctx.write_selection_set(Some(selection_set_type), &mut buffer, plan.selection_set())?;
            buffer
        };

        let mut query = String::with_capacity(selection_set.len() + 14 + ctx.estimated_variable_definitions_string_len);
        match operation_type {
            OperationType::Query => write!(query, "query")?,
            OperationType::Mutation => write!(query, "mutation")?,
            OperationType::Subscription => write!(query, "subscription")?,
        };

        if !ctx.variables.is_empty() {
            query.push('(');
            ctx.write_operation_arguments_without_parenthesis(&mut query)?;
            query.push(')');
        }

        query.push_str(&selection_set);

        Ok(PreparedGraphqlOperation {
            ty: operation_type,
            query,
            variables: ctx.into_query_variables(),
        })
    }
}

pub(crate) struct PreparedFederationEntityOperation {
    pub query: String,
    pub entities_variable_name: String,
    pub variables: QueryVariables,
}

impl PreparedFederationEntityOperation {
    pub(crate) fn build(plan: PlanWalker<'_>, subgraph_id: SubgraphId) -> Result<Self, Error> {
        let mut ctx = QueryBuilderContext::new(subgraph_id);

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = String::with_capacity(256);
            ctx.write_selection_set(None, &mut buffer, plan.selection_set())?;
            buffer
        };

        let entities_variable_name = format!("{VARIABLE_PREFIX}{}", ctx.variables.len());
        let mut query = String::with_capacity(
            // Rough approximation of the final string length counted by hand
            selection_set.len() + 60 + ctx.estimated_variable_definitions_string_len + 2 * entities_variable_name.len(),
        );
        query.push_str("query");
        query.push('(');
        write!(query, "${entities_variable_name}: [_Any!]!")?;

        if !ctx.variables.is_empty() {
            query.push(',');
            ctx.write_operation_arguments_without_parenthesis(&mut query)?;
        }
        query.push(')');

        write!(
            query,
            " {{ _entities(representations: ${entities_variable_name}){selection_set} }}"
        )?;

        Ok(PreparedFederationEntityOperation {
            query,
            entities_variable_name,
            variables: ctx.into_query_variables(),
        })
    }
}

/// All variables associated with a subgraph query. Each one is associated with the variable name
/// "{$VARIABLE_PREFIX}{idx}" with `idx` being the position of the input value in the inner vec.
pub(crate) struct QueryVariables(Vec<QueryInputValueId>);

impl QueryVariables {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, QueryInputValueId)> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, &id)| (format!("{VARIABLE_PREFIX}{}", idx), id))
    }
}

struct QueryVariable {
    idx: usize,
    ty: String,
}

struct QueryBuilderContext {
    subgraph_id: SubgraphId,
    variables: HashMap<QueryInputValueId, QueryVariable>,
    estimated_variable_definitions_string_len: usize,
}

impl QueryBuilderContext {
    fn new(subgraph_id: SubgraphId) -> Self {
        Self {
            subgraph_id,
            variables: HashMap::new(),
            estimated_variable_definitions_string_len: 0,
        }
    }

    fn into_query_variables(self) -> QueryVariables {
        let mut vars = vec![None; self.variables.len()];
        for (input_value_id, var) in self.variables {
            vars[var.idx] = Some(input_value_id);
        }

        QueryVariables(vars.into_iter().map(Option::unwrap).collect())
    }

    fn write_operation_arguments_without_parenthesis(&self, out: &mut String) -> Result<(), Error> {
        write!(
            out,
            "{}",
            self.variables.values().format_with(",", |var, f| {
                // no need to add the default value, we'll always provide the variable.
                f(&format_args!("${VARIABLE_PREFIX}{}: {}", var.idx, var.ty))
            })
        )
    }

    fn write_selection_set(
        &mut self,
        maybe_selection_set_type: Option<SelectionSetType>,
        buffer: &mut String,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        buffer.push_str(" {");
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            buffer.push_str(" __typename");
        }
        self.write_selection_set_fields(maybe_selection_set_type, buffer, selection_set)?;
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            buffer.push_str(" __typename");
        }
        buffer.push_str(" }");
        Ok(())
    }

    fn write_selection_set_fields(
        &mut self,
        selection_set_type: Option<SelectionSetType>,
        buffer: &mut String,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        let subgraph_id = self.subgraph_id;
        let parent_entity_id = selection_set_type.and_then(|t| t.as_entity_id());

        let entity_to_fields = selection_set
            .fields_ordered_by_parent_entity_id_then_position()
            .into_iter()
            .chunk_by(|field| field.definition().parent_entity_id);

        for (entity_id, fields) in entity_to_fields.into_iter() {
            tracing::debug!("{}", entity_id.walk(selection_set.walker().schema()).name());
            let fields = fields.collect_vec();
            let entity = selection_set.walker().schema().walk(entity_id);
            let in_same_entity = parent_entity_id == Some(entity_id);

            let mut add_interface_fragment = false;

            match (selection_set_type, entity) {
                (Some(SelectionSetType::Interface(_)), schema::EntityDefinition::Interface(interface))
                    if interface.is_not_fully_implemented_in(subgraph_id) && !in_same_entity =>
                {
                    let objects = interface
                        .possible_types_ordered_by_typename()
                        .filter(|o| o.resolvable_in(&subgraph_id));

                    for object in objects {
                        if object.subgraph_implements_interface(&subgraph_id, &interface.id()) {
                            add_interface_fragment = true;
                        } else {
                            self.write_type_fields(buffer, object.name(), &fields)?;
                        }
                    }
                }
                (Some(SelectionSetType::Union(union_id)), schema::EntityDefinition::Interface(interface))
                    if interface.is_not_fully_implemented_in(subgraph_id) && !in_same_entity =>
                {
                    let objects = selection_set
                        .walker()
                        .schema()
                        .walk(union_id)
                        .possible_types_ordered_by_typename()
                        .filter(|o| o.resolvable_in(&subgraph_id));

                    for object in objects {
                        if object.subgraph_implements_interface(&subgraph_id, &interface.id()) {
                            add_interface_fragment = true;
                        } else {
                            self.write_type_fields(buffer, object.name(), &fields)?;
                        }
                    }
                }
                _ => {
                    if let Some(SelectionSetType::Interface(interface_id)) = selection_set_type {
                        if let EntityDefinition::Object(ref object) = entity {
                            if !object.subgraph_implements_interface(&subgraph_id, &interface_id) {
                                continue;
                            }
                        }
                    }

                    add_interface_fragment = true;
                }
            }

            if add_interface_fragment {
                self.write_entity_fields(in_same_entity, buffer, entity, &fields)?;
            }
        }

        Ok(())
    }

    fn write_entity_fields(
        &mut self,
        in_same_entity: bool,
        buffer: &mut String,
        entity: EntityDefinition<'_>,
        fields: &[PlanWalker<'_, crate::operation::FieldId>],
    ) -> Result<(), Error> {
        if !in_same_entity {
            write!(buffer, " ... on {} {{", entity.name())?;
        }

        for field in fields {
            self.write_field(buffer, *field)?;
        }

        if !in_same_entity {
            buffer.push_str(" }");
        }

        Ok(())
    }

    fn write_type_fields(
        &mut self,
        buffer: &mut String,
        type_name: &str,
        fields: &[PlanWalker<'_, crate::operation::FieldId>],
    ) -> Result<(), Error> {
        write!(buffer, " ... on {} {{", type_name)?;

        for field in fields {
            self.write_field(buffer, *field)?;
        }

        buffer.push_str(" }");

        Ok(())
    }

    fn write_field(&mut self, buffer: &mut String, field: PlanField<'_>) -> Result<(), Error> {
        let response_key = field.response_key_str();
        let name = field.definition().name();
        buffer.push(' ');
        if response_key == name {
            buffer.push_str(name);
        } else {
            write!(buffer, "{response_key}: {name}")?;
        }
        self.write_arguments(buffer, field.arguments())?;
        if let Some(selection_set) = field.selection_set() {
            self.write_selection_set(
                SelectionSetType::maybe_from(field.definition().ty().definition().id()),
                buffer,
                selection_set,
            )?;
        }
        Ok(())
    }

    fn write_arguments(&mut self, buffer: &mut String, arguments: FieldArgumentsWalker<'_>) -> Result<(), Error> {
        if !arguments.is_empty() {
            write!(
                buffer,
                "({})",
                arguments.into_iter().format_with(", ", |arg, f| {
                    // If the argument is a constant value that would still be present after query
                    // normalization we keep it to avoid adding unnecessary variables.
                    if let Some(value) = arg
                        .value()
                        .and_then(|value| value.to_normalized_query_const_value_str())
                    {
                        f(&format_args!("{}: {}", arg.definition().name(), value))
                    } else {
                        let idx = self.variables.len();
                        let var = self.variables.entry(arg.as_ref().input_value_id).or_insert_with(|| {
                            let ty = arg.definition().ty().to_string();
                            // prefix + ': ' + index (2) + ',' + ty.len()
                            self.estimated_variable_definitions_string_len += VARIABLE_PREFIX.len() + 5 + ty.len();
                            QueryVariable { idx, ty }
                        });
                        f(&format_args!(
                            "{}: ${VARIABLE_PREFIX}{}",
                            arg.definition().name(),
                            var.idx
                        ))
                    }
                })
            )?;
        }
        Ok(())
    }
}
