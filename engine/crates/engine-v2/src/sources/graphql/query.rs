use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

use engine_parser::types::OperationType;
use itertools::Itertools;

use crate::{
    operation::{OpInputValueId, SelectionSetTypeWalker},
    plan::{
        PlanField, PlanFieldArgument, PlanFragmentSpread, PlanInlineFragment, PlanSelection, PlanSelectionSet,
        PlanWalker,
    },
};

const VARIABLE_PREFIX: &str = "var";

macro_rules! indent_write {
    ($dst:ident, $($arg:tt)*) => {{
        $dst.write_indent();
        write!($dst, $($arg)*)
    }};
}

pub(super) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: QueryVariables,
}

impl PreparedGraphqlOperation {
    pub(super) fn build(
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut ctx = QueryBuilderContext::default();
        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::default();
            ctx.write_selection_set(&mut buffer, plan.selection_set())?;
            buffer.into_string()
        };

        let mut query = String::new();
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

pub(super) struct PreparedFederationEntityOperation {
    pub query: String,
    pub entities_variable_name: String,
    pub variables: QueryVariables,
}

impl PreparedFederationEntityOperation {
    pub(super) fn build(plan: PlanWalker<'_>) -> Result<Self, Error> {
        let mut ctx = QueryBuilderContext::default();
        let mut query = String::from("query");

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::default();
            buffer.indent += 2;
            ctx.write_selection_set(&mut buffer, plan.selection_set())?;
            buffer.into_string()
        };

        query.push('(');
        let entities_variable_name = format!("{VARIABLE_PREFIX}{}", ctx.variables.len());
        write!(query, "${entities_variable_name}: [_Any!]!")?;

        if !ctx.variables.is_empty() {
            query.push(',');
            ctx.write_operation_arguments_without_parenthesis(&mut query)?;
        }
        query.push(')');
        let type_name = plan.selection_set().ty().name();
        query.push_str(" {");
        write!(query, "\n  _entities(representations: ${entities_variable_name}) {{")?;
        query.push_str("\n    __typename");
        write!(query, "\n    ... on {type_name} {selection_set}  }}",)?;
        query.push_str("\n}\n");

        Ok(PreparedFederationEntityOperation {
            query,
            entities_variable_name,
            variables: ctx.into_query_variables(),
        })
    }
}

/// All variables associated with a subgraph query. Each one is associated with the variable name
/// "{$VARIABLE_PREFIX}{idx}" with `idx` being the position of the input value in the inner vec.
pub struct QueryVariables(Vec<OpInputValueId>);

impl QueryVariables {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, OpInputValueId)> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, &id)| (format!("{VARIABLE_PREFIX}{}", idx), id))
    }
}

pub struct QueryVariable {
    idx: usize,
    ty: String,
}

#[derive(Default)]
pub struct QueryBuilderContext {
    variables: HashMap<OpInputValueId, QueryVariable>,
}

impl QueryBuilderContext {
    pub fn into_query_variables(self) -> QueryVariables {
        let mut vars = vec![0.into(); self.variables.len()];
        for (input_value_id, var) in self.variables {
            vars[var.idx] = input_value_id;
        }
        QueryVariables(vars)
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

    fn write_selection_set(&mut self, buffer: &mut Buffer, selection_set: PlanSelectionSet<'_>) -> Result<(), Error> {
        buffer.write_str(" {\n")?;
        buffer.indent += 1;
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            indent_write!(buffer, "__typename\n")?;
        }
        self.write_selection_set_fields(buffer, selection_set)?;
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
        buffer: &mut Buffer,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        let ty = selection_set.ty();
        for selection in selection_set {
            match selection {
                PlanSelection::Field(field) => self.write_field(buffer, field)?,
                PlanSelection::FragmentSpread(spread) => self.write_fragment_spread(buffer, ty, spread)?,
                PlanSelection::InlineFragment(fragment) => self.write_inline_fragment(buffer, ty, fragment)?,
            };
        }
        Ok(())
    }

    fn write_fragment_spread(
        &mut self,
        buffer: &mut Buffer,
        parent_ty: SelectionSetTypeWalker<'_>,
        spread: PlanFragmentSpread<'_>,
    ) -> Result<(), Error> {
        // We're writing a named fragment directly into the query as an inline one. The query might
        // end up being a bit larger, but trying to keep fragments is not that efficient as their
        // fields might have been planned differently at different locations. The other concern are
        // named fragment only directives, but that's something we'll see if we ever need to.
        let selection_set = spread.selection_set();
        let ty = selection_set.ty();
        // We don't create a nested selection set if the type condition is equivalent to the parent
        // type
        if parent_ty == ty {
            self.write_selection_set_fields(buffer, selection_set)?;
        } else {
            indent_write!(buffer, "... on {}", ty.name())?;
            self.write_selection_set(buffer, selection_set)?;
        }
        Ok(())
    }

    fn write_inline_fragment(
        &mut self,
        buffer: &mut Buffer,
        parent_ty: SelectionSetTypeWalker<'_>,
        fragment: PlanInlineFragment<'_>,
    ) -> Result<(), Error> {
        let selection_set = fragment.selection_set();
        let ty = selection_set.ty();
        // We don't create a nested selection set if the type condition is equivalent to the parent
        // type
        if parent_ty == ty {
            self.write_selection_set_fields(buffer, selection_set)?;
        } else {
            indent_write!(buffer, "... on {}", ty.name())?;
            self.write_selection_set(buffer, selection_set)?;
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
            self.write_selection_set(buffer, selection_set)?;
        } else {
            buffer.push('\n');
        }
        Ok(())
    }

    fn write_arguments<'a>(
        &mut self,
        buffer: &mut Buffer,
        arguments: impl ExactSizeIterator<Item = PlanFieldArgument<'a>>,
    ) -> Result<(), Error> {
        if arguments.len() != 0 {
            write!(
                buffer,
                "({})",
                arguments.format_with(", ", |arg, f| {
                    let idx = self.variables.len();
                    let var = self.variables.entry(arg.value().id()).or_insert_with(|| QueryVariable {
                        idx,
                        ty: arg.ty().to_string(),
                    });
                    f(&format_args!("{}: ${VARIABLE_PREFIX}{}", arg.name(), var.idx))
                })
            )?;
        }
        Ok(())
    }
}

#[derive(Default, Hash, PartialEq, Eq)]
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
