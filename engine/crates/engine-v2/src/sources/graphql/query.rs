use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use engine_parser::types::OperationType;
use engine_value::ConstValue;
use itertools::Itertools;
use serde::ser::SerializeMap;

use crate::{
    execution::ExecutionContext,
    plan::{PlanId, PlanOutput},
    request::{
        PlanField, PlanFieldArgument, PlanFragmentSpread, PlanInlineFragment, PlanOperationWalker, PlanSelection,
        PlanSelectionSet,
    },
    response::ResponseBoundaryObjectsView,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

#[derive(serde::Serialize)]
pub(super) struct Query<'a> {
    pub query: String,
    pub variables: HashMap<String, &'a ConstValue>,
}

impl<'ctx> Query<'ctx> {
    pub(super) fn build(
        ctx: ExecutionContext<'ctx>,
        plan_id: PlanId,
        plan_output: &PlanOutput,
    ) -> Result<Query<'ctx>, Error> {
        let operation = ctx.walk(plan_output);
        let mut builder = QueryBuilder::default();
        let selection_set = {
            let mut buffer = Buffer::default();
            builder.write_operation_selection_set(&mut buffer, operation)?;
            buffer.inner
        };

        let mut query = String::new();
        match operation.ty() {
            OperationType::Query => write!(query, "query ")?,
            OperationType::Mutation => write!(query, "mutation ")?,
            OperationType::Subscription => write!(query, "subscription ")?,
        };
        query.push_str(&QueryBuilder::operation_name(operation, plan_id));

        let variables = if !builder.variable_references.is_empty() {
            query.push('(');
            let variables = builder.write_operation_arguments_without_parenthesis(ctx, &mut query)?;
            query.push(')');
            variables
        } else {
            HashMap::new()
        };

        query.push_str(&selection_set);
        builder.write_fragments(&mut query);

        Ok(Query { query, variables })
    }
}

#[derive(serde::Serialize)]
pub(super) struct FederationEntityQuery<'a> {
    pub query: String,
    pub variables: FederationEntityVariables<'a>,
}

#[derive(Default)]
pub(super) struct FederationEntityVariables<'a> {
    pub query_variables: HashMap<String, &'a ConstValue>,
    pub representations: HashMap<String, ResponseBoundaryObjectsView<'a>>,
}

impl<'a> serde::Serialize for FederationEntityVariables<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.query_variables.len() + self.representations.len()))?;
        for (key, value) in &self.query_variables {
            map.serialize_entry(key, value)?;
        }
        for (key, response_objects) in &self.representations {
            map.serialize_entry(key, response_objects)?;
        }
        map.end()
    }
}

impl<'a> FederationEntityQuery<'a> {
    pub(super) fn build(
        ctx: ExecutionContext<'a>,
        plan_id: PlanId,
        plan_output: &PlanOutput,
        entities: ResponseBoundaryObjectsView<'a>,
    ) -> Result<FederationEntityQuery<'a>, Error> {
        let operation = ctx.walk(plan_output);
        let mut builder = QueryBuilder::default();
        let mut query = String::from("query ");
        query.push_str(&QueryBuilder::operation_name(operation, plan_id));

        let selection_set = {
            let mut buffer = Buffer::default();
            buffer.indent += 2;
            builder.write_operation_selection_set(&mut buffer, operation)?;
            buffer.inner
        };

        let mut variables = FederationEntityVariables::default();
        query.push('(');
        let var_name = format!("representationsPlan{plan_id}");
        variables.representations.insert(var_name.clone(), entities);
        query.push_str(&format!("${var_name}: [_Any!]!"));
        if !builder.variable_references.is_empty() {
            query.push(',');
            variables
                .query_variables
                .extend(builder.write_operation_arguments_without_parenthesis(ctx, &mut query)?);
        }
        query.push(')');
        let type_name = operation.selection_set().ty().name();
        query.push_str(" {");
        query.push_str(&format!("\n\t_entities(representations: ${var_name}) {{"));
        query.push_str("\n\t\t__typename");
        query.push_str(&format!("\n\t\t... on {type_name} {selection_set}\t}}"));
        query.push_str("\n}\n");

        builder.write_fragments(&mut query);

        Ok(FederationEntityQuery { query, variables })
    }
}

#[derive(Default)]
pub struct QueryBuilder {
    fragment_content_to_name: HashMap<Buffer, String>,
    fragment_name_to_last_id: HashMap<String, usize>,
    variable_references: HashSet<String>,
}

impl QueryBuilder {
    fn write_operation_selection_set(
        &mut self,
        buffer: &mut Buffer,
        operation: PlanOperationWalker<'_>,
    ) -> Result<(), Error> {
        self.write_selection_set(buffer, operation.selection_set())
    }

    fn operation_name(operation: PlanOperationWalker<'_>, plan_id: PlanId) -> String {
        operation
            .name()
            .map(|name| format!("{name}_Plan{plan_id}"))
            .unwrap_or_else(|| format!("Plan{plan_id}"))
    }

    fn write_operation_arguments_without_parenthesis<'ctx>(
        &self,
        ctx: ExecutionContext<'ctx>,
        out: &mut String,
    ) -> Result<HashMap<String, &'ctx ConstValue>, Error> {
        let mut variables = HashMap::new();
        out.push_str(&format!(
            "{}",
            self.variable_references.iter().format_with(", ", |name, f| {
                let variable = ctx
                    .variables()
                    .get(name)
                    .expect("we just found it in the query so validation wouldn't have passed otherwise.");

                let Some(value) = variable.value() else { return Ok(()) };

                variables.insert(name.to_string(), value);
                if let Some(default_value) = variable.default_value() {
                    f(&format_args!(
                        "${name}: {ty} = {default_value}",
                        ty = variable.type_name()
                    ))
                } else {
                    f(&format_args!("${name}: {ty}", ty = variable.type_name()))
                }
            })
        ));
        Ok(variables)
    }

    fn write_fragments(&self, out: &mut String) {
        out.push_str(&format!(
            "\n{}",
            self.fragment_content_to_name
                .iter()
                .format_with("\n", |(fragment, name), f| {
                    f(&format_args!("fragment {name} {}", fragment.inner))
                }),
        ));
    }

    fn write_selection_set(&mut self, buffer: &mut Buffer, selection_set: PlanSelectionSet<'_>) -> Result<(), Error> {
        buffer.write_str(" {\n")?;
        buffer.indent += 1;
        let n = buffer.len();
        if !selection_set.ty().is_object() {
            // We always need to know the concrete object.
            buffer.indent_write("__typename\n")?;
        }
        for selection in selection_set {
            match selection {
                PlanSelection::Field(field) => self.write_field(buffer, field)?,
                PlanSelection::FragmentSpread(spread) => self.write_fragment_spread(buffer, spread)?,
                PlanSelection::InlineFragment(fragment) => self.write_inline_fragment(buffer, fragment)?,
            };
        }
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            buffer.indent_write("__typename\n")?;
        }
        buffer.indent -= 1;
        buffer.indent_write("}\n")?;
        Ok(())
    }

    fn write_fragment_spread(&mut self, buffer: &mut Buffer, spread: PlanFragmentSpread<'_>) -> Result<(), Error> {
        let fragment = spread.fragment();
        // Nothing to deal with fragment cycles here, they should have been detected way earlier
        // during query validation.
        let mut fragment_buffer = Buffer::default();
        // the actual name is computed afterwards as attribution of the fragment fields will depend
        // on its spread location, so it isn't necessarily the same. Once we have tests for
        // directives we could simplify that as there is not need to keep named fragment except for
        // their directives that the upstream server may understand.
        fragment_buffer.write_str(&format!("on {}", fragment.type_condition_name()))?;
        self.write_selection_set(&mut fragment_buffer, spread.selection_set())?;
        let name = self.fragment_content_to_name.entry(fragment_buffer).or_insert_with(|| {
            let id = self
                .fragment_name_to_last_id
                .entry(fragment.name().to_string())
                .and_modify(|id| *id += 1)
                .or_default();
            format!("{}_{}", fragment.name(), id)
        });
        buffer.indent_write(&format!("...{name}\n"))?;
        Ok(())
    }

    fn write_inline_fragment(&mut self, buffer: &mut Buffer, fragment: PlanInlineFragment<'_>) -> Result<(), Error> {
        buffer.indent_write("...")?;
        if let Some(name) = fragment.type_condition_name() {
            buffer.write_str(&format!(" on {name}"))?;
        }
        self.write_selection_set(buffer, fragment.selection_set())?;
        Ok(())
    }

    fn write_field(&mut self, buffer: &mut Buffer, field: PlanField<'_>) -> Result<(), Error> {
        let response_key = field.response_key_str();
        let name = field.name();
        if response_key == name {
            buffer.indent_write(name)?;
        } else {
            buffer.indent_write(&format!("{response_key}: {name}"))?;
        }
        self.write_arguments(buffer, field.bound_arguments())?;
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
            buffer.write_str(&format!(
                "({})",
                arguments.format_with(", ", |argument, f| {
                    let value = argument.query_value();
                    self.add_variable_references(value.variables_used().map(|name| name.to_string()));
                    f(&format_args!("{name}: {value}", name = argument.name()))
                })
            ))?;
        }
        Ok(())
    }

    fn add_variable_references(&mut self, names: impl IntoIterator<Item = String>) {
        self.variable_references.extend(names);
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
    fn indent_write(&mut self, s: impl AsRef<str>) -> Result<(), std::fmt::Error> {
        let indent = "\t".repeat(self.indent);
        self.write_str(&indent)?;
        self.write_str(s.as_ref())
    }
}
