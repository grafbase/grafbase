use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

use grafbase_telemetry::graphql::OperationType;
use itertools::Itertools;
use operation::QueryOrSchemaInputValueId;
use schema::{CompositeType, EntityDefinition, SubgraphId};

use crate::prepare::{PartitionFieldArguments, PlanQueryPartition, SubgraphField, SubgraphSelectionSet};

const VARIABLE_PREFIX: &str = "var";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: QueryVariables,
}

impl PreparedGraphqlOperation {
    pub(crate) fn build(
        operation_type: OperationType,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut ctx = QueryBuilderContext::new(plan_query_partition.resolver_definition().subgraph_id());

        // Generating the selection set first as this will define all the operation arguments
        let mut selection_set = String::with_capacity(256);
        ctx.write_selection_set(
            ParentType::CompositeType(plan_query_partition.entity_definition().into()),
            &mut selection_set,
            plan_query_partition.selection_set(),
        )?;

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedFederationEntityOperation {
    pub query: String,
    pub entities_variable_name: String,
    pub variables: QueryVariables,
}

impl PreparedFederationEntityOperation {
    pub(crate) fn build(plan_query_partition: PlanQueryPartition<'_>) -> Result<Self, Error> {
        let mut ctx = QueryBuilderContext::new(plan_query_partition.resolver_definition().subgraph_id());

        // Generating the selection set first as this will define all the operation arguments
        let mut selection_set = String::with_capacity(256);
        ctx.write_selection_set(
            ParentType::Any,
            &mut selection_set,
            plan_query_partition.selection_set(),
        )?;

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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryVariables(Vec<QueryOrSchemaInputValueId>);

impl QueryVariables {
    pub fn iter(&self) -> impl Iterator<Item = (String, QueryOrSchemaInputValueId)> + '_ {
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
    variables: HashMap<QueryOrSchemaInputValueId, QueryVariable>,
    estimated_variable_definitions_string_len: usize,
}

enum ParentType<'a> {
    Any,
    CompositeType(CompositeType<'a>),
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
            self.variables.values().format_with(", ", |var, f| {
                // no need to add the default value, we'll always provide the variable.
                f(&format_args!("${VARIABLE_PREFIX}{}: {}", var.idx, var.ty))
            })
        )
    }

    fn write_selection_set(
        &mut self,
        parent_type: ParentType<'_>,
        buffer: &mut String,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> Result<(), Error> {
        buffer.push_str(" {");
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            buffer.push_str(" __typename");
        }
        self.write_selection_set_fields(parent_type, buffer, selection_set)?;
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            buffer.push_str(" __typename @skip(if: true)");
        }
        buffer.push_str(" }");
        Ok(())
    }

    fn write_selection_set_fields(
        &mut self,
        parent_type: ParentType<'_>,
        buffer: &mut String,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> Result<(), Error> {
        let subgraph_id = self.subgraph_id;

        let ParentType::CompositeType(parent_type) = parent_type else {
            let entity_fields = selection_set
                .fields_ordered_by_type_condition_then_key()
                .chunk_by(|field| field.definition().parent_entity());

            // Parent is Any or any other type that will accept anything.
            for (entity, fields) in entity_fields.into_iter() {
                self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
            }
            return Ok(());
        };

        if let CompositeType::Object(_) = parent_type {
            // From here one, it doesn't matter from where fields are coming, the
            // subgraph object must expose all the fields so we just request them directly
            // without any type conditions.
            self.write_fields(buffer, selection_set.fields())?;
            return Ok(());
        }

        let entity_fields = selection_set
            .fields_ordered_by_type_condition_then_key()
            .chunk_by(|field| field.definition().parent_entity());

        let maybe_parent_interface_id = parent_type.as_interface().map(|interface| interface.id);
        let parent_is_fully_implemented = parent_type.is_fully_implemented_in_subgraph(subgraph_id);
        for (entity, fields) in entity_fields.into_iter() {
            let interface = match entity {
                EntityDefinition::Object(object) => {
                    match parent_type {
                        CompositeType::Interface(interface)
                            if object.implements_interface_in_subgraph(&subgraph_id, &interface.id) =>
                        {
                            self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                        }
                        CompositeType::Union(union) if union.has_member_in_subgraph(subgraph_id, object.id) => {
                            self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                        }
                        _ => (),
                    }
                    continue;
                }
                EntityDefinition::Interface(interface) => interface,
            };

            // If it's the same interface as the parent, there's nothing to do. It's either fully implemented
            // and we're good. Or it isn't and we can't retrieve missing objects from the parent anyway because
            // it's the same interface from our perspective.
            if Some(interface.id) == maybe_parent_interface_id {
                self.write_fields(buffer, fields)?;
                continue;
            }

            // If fully implemented, it's consistent with the our, the super-graph's, view.
            if interface.is_fully_implemented_in(subgraph_id) {
                self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                continue;
            }

            // From here on we know that some objects that implement the interface on our side
            // don't in the subgraphs. But we still want to retrieve them if they're part the
            // parent type.

            let fields = fields.collect::<Vec<_>>();
            let possible_subgraph_objects = interface
                .possible_types()
                .filter(|o| o.exists_in_subgraph(&subgraph_id));

            let mut add_interface_fragment = false;
            for object in possible_subgraph_objects {
                // For objects that do implement the interface, we add an interface fragment
                // instead to reduce query size.
                if object.implements_interface_in_subgraph(&subgraph_id, &interface.id) {
                    add_interface_fragment = true;
                } else if parent_is_fully_implemented
                    || parent_type.possible_types_include_in_subgraph(subgraph_id, object.id)
                {
                    // Here we know that the subgraph can provide this object but it doesn't
                    // implement the interface, so we need to add it separately.
                    self.write_type_condition_and_entity_fields(
                        buffer,
                        EntityDefinition::Object(object),
                        fields.iter().copied(),
                    )?;
                }
            }

            if add_interface_fragment {
                self.write_type_condition_and_entity_fields(buffer, entity, fields.iter().copied())?;
            }
        }

        Ok(())
    }

    fn write_type_condition_and_entity_fields<'a>(
        &mut self,
        buffer: &mut String,
        entity: EntityDefinition<'_>,
        fields: impl Iterator<Item = SubgraphField<'a>>,
    ) -> Result<(), Error> {
        write!(buffer, " ... on {} {{", entity.name())?;

        for field in fields {
            self.write_field(buffer, field)?;
        }

        buffer.push_str(" }");

        Ok(())
    }

    fn write_fields<'a>(
        &mut self,
        buffer: &mut String,
        fields: impl Iterator<Item = SubgraphField<'a>>,
    ) -> Result<(), Error> {
        for field in fields {
            self.write_field(buffer, field)?;
        }

        Ok(())
    }

    fn write_field(&mut self, buffer: &mut String, field: SubgraphField<'_>) -> Result<(), Error> {
        let response_key = field.subgraph_response_key_str();
        let name = field.definition().name();
        buffer.push(' ');
        if response_key == name {
            buffer.push_str(name);
        } else {
            write!(buffer, "{response_key}: {name}")?;
        }
        self.write_arguments(buffer, field.arguments())?;
        if let Some(ty) = field
            .definition()
            .subgraph_types()
            .find(|record| record.subgraph_id == self.subgraph_id)
            .and_then(|record| record.ty().definition().as_composite_type())
        {
            self.write_selection_set(ParentType::CompositeType(ty), buffer, field.selection_set())?;
        } else if let Some(ty) = field.definition().ty().definition().as_composite_type() {
            self.write_selection_set(ParentType::CompositeType(ty), buffer, field.selection_set())?;
        }
        Ok(())
    }

    fn write_arguments(&mut self, buffer: &mut String, arguments: PartitionFieldArguments<'_>) -> Result<(), Error> {
        if arguments.len() != 0 {
            write!(
                buffer,
                "({})",
                arguments.into_iter().format_with(", ", |arg, f| {
                    // If the argument is a constant value that would still be present after query
                    // normalization we keep it to avoid adding unnecessary variables.
                    if let Some(value) = arg.value_as_sanitized_query_const_value_str() {
                        f(&format_args!("{}: {}", arg.definition().name(), value))
                    } else {
                        let idx = self.variables.len();
                        let var = self.variables.entry(arg.value_id).or_insert_with(|| {
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
