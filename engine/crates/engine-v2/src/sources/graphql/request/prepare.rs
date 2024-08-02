use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::EntityId;

use crate::{
    execution::{PlanField, PlanSelectionSet, PlanWalker},
    operation::{FieldArgumentsWalker, QueryInputValueId},
};

const VARIABLE_PREFIX: &str = "var";

macro_rules! indent_write {
    ($dst:ident, $($arg:tt)*) => {{
        $dst.write_indent();
        write!($dst, $($arg)*)
    }};
}

pub(crate) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: QueryVariables,
}

impl PreparedGraphqlOperation {
    pub(crate) fn build(
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut ctx = QueryBuilderContext::default();
        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::with_capacity(256);
            let entity_id = EntityId::Object(match operation_type {
                OperationType::Query => plan.schema().as_ref().graph.root_operation_types.query,
                OperationType::Mutation => plan.schema().as_ref().graph.root_operation_types.mutation.unwrap(),
                OperationType::Subscription => plan.schema().as_ref().graph.root_operation_types.subscription.unwrap(),
            });
            ctx.write_selection_set(Some(entity_id), &mut buffer, plan.selection_set())?;
            buffer.into_string()
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
    pub(crate) fn build(plan: PlanWalker<'_>) -> Result<Self, Error> {
        let mut ctx = QueryBuilderContext::default();

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::with_capacity(256);
            buffer.indent += 1;
            ctx.write_selection_set(None, &mut buffer, plan.selection_set())?;
            buffer.into_string()
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
            " {{\n  _entities(representations: ${entities_variable_name}){selection_set}}}"
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

#[derive(Default)]
struct QueryBuilderContext {
    variables: HashMap<QueryInputValueId, QueryVariable>,
    estimated_variable_definitions_string_len: usize,
}

impl QueryBuilderContext {
    fn into_query_variables(self) -> QueryVariables {
        let mut vars = vec![None; self.variables.len()];
        for (input_value_id, var) in self.variables {
            vars[var.idx] = Some(input_value_id);
        }

        QueryVariables(vars.into_iter().map(Option::unwrap).collect())
    }
}

impl QueryBuilderContext {
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
        maybe_entity_id: Option<EntityId>,
        buffer: &mut Buffer,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        buffer.write_str(" {\n")?;
        buffer.indent += 1;
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            indent_write!(buffer, "__typename\n")?;
        }
        self.write_selection_set_fields(maybe_entity_id, buffer, selection_set)?;
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            indent_write!(buffer, "__typename\n")?;
        }
        buffer.indent -= 1;
        indent_write!(buffer, "}}\n")
    }

    fn write_selection_set_fields(
        &mut self,
        maybe_entity_id: Option<EntityId>,
        buffer: &mut Buffer,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        let entity_to_fields = selection_set
            .fields_ordered_by_parent_entity_id_then_position()
            .into_iter()
            .chunk_by(|field| field.parent_entity().id());
        for (entity_id, fields) in entity_to_fields.into_iter() {
            if maybe_entity_id != Some(entity_id) {
                indent_write!(
                    buffer,
                    "... on {} {{\n",
                    selection_set.walker().schema().walk(entity_id).name()
                )?;
                buffer.indent += 1;
            }
            for field in fields {
                self.write_field(buffer, field)?;
            }
            if maybe_entity_id != Some(entity_id) {
                buffer.indent -= 1;
                indent_write!(buffer, "}}\n")?;
            }
        }
        Ok(())
    }

    fn write_field(&mut self, buffer: &mut Buffer, field: PlanField<'_>) -> Result<(), Error> {
        let response_key = field.response_key_str();
        let name = field.name();
        if response_key == name {
            indent_write!(buffer, "{name}")?;
        } else {
            indent_write!(buffer, "{response_key}: {name}")?;
        }
        self.write_arguments(buffer, field.arguments())?;
        if let Some(selection_set) = field.selection_set() {
            self.write_selection_set(EntityId::maybe_from(field.ty().inner().id()), buffer, selection_set)?;
        } else {
            buffer.push('\n');
        }
        Ok(())
    }

    fn write_arguments(&mut self, buffer: &mut Buffer, arguments: FieldArgumentsWalker<'_>) -> Result<(), Error> {
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
                        f(&format_args!("{}: {}", arg.name(), value))
                    } else {
                        let idx = self.variables.len();
                        let var = self.variables.entry(arg.as_ref().input_value_id).or_insert_with(|| {
                            let ty = arg.ty().to_string();
                            // prefix + ': ' + index (2) + ',' + ty.len()
                            self.estimated_variable_definitions_string_len += VARIABLE_PREFIX.len() + 5 + ty.len();
                            QueryVariable { idx, ty }
                        });
                        f(&format_args!("{}: ${VARIABLE_PREFIX}{}", arg.name(), var.idx))
                    }
                })
            )?;
        }
        Ok(())
    }
}

#[derive(Hash, PartialEq, Eq)]
struct Buffer {
    inner: String,
    indent: usize,
}

impl std::ops::Deref for Buffer {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Buffer {
    fn with_capacity(capacity: usize) -> Self {
        Buffer {
            inner: String::with_capacity(capacity),
            indent: 0,
        }
    }

    fn into_string(self) -> String {
        self.inner
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.inner.push(' ');
            self.inner.push(' ');
        }
    }
}
