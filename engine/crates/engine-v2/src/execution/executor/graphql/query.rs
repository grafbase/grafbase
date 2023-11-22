use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use engine_parser::types::OperationType;
use engine_value::ConstValue;
use itertools::Itertools;

use crate::{
    execution::walkers::{
        FieldArgumentWalker, FieldWalker, FragmentSpreadWalker, InlineFragmentWalker, SelectionSetWalker,
        SelectionWalker, VariablesWalker,
    },
    plan::PlanId,
    request::Operation,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

pub struct Query<'a> {
    pub query: String,
    pub variables: HashMap<String, &'a ConstValue>,
}

#[derive(Default)]
pub struct QueryBuilder {
    fragment_contents: HashMap<Buffer, String>,
    fragment_last_id: HashMap<String, usize>,
    variable_references: HashSet<String>,
}

impl QueryBuilder {
    pub fn build<'a>(
        operation: &'a Operation,
        plan_id: PlanId,
        variables: VariablesWalker<'a>,
        selection_set: SelectionSetWalker<'a>,
    ) -> Result<Query<'a>, Error> {
        let mut builder = QueryBuilder::default();
        let mut query = Buffer::default();
        builder.write_selection_set(&mut query, selection_set)?;

        let mut out = String::new();
        match operation.ty {
            OperationType::Query => write!(out, "query ")?,
            OperationType::Mutation => write!(out, "mutation ")?,
            OperationType::Subscription => write!(out, "subscription ")?,
        };

        out.push_str(
            &operation
                .name
                .as_ref()
                .map(|name| format!("{name}_Plan{plan_id}"))
                .unwrap_or_else(|| format!("Plan{plan_id}")),
        );
        if !builder.variable_references.is_empty() {
            out.push_str(&format!(
                "({})",
                builder.variable_references.iter().format_with(", ", |name, f| {
                    let variable = variables.unchecked_get(name);
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
        }
        out.push_str(&query);
        for (fragment, name) in builder.fragment_contents {
            out.push_str(&format!("\nfragment {name} {}", fragment.inner));
        }

        Ok(Query {
            query: out,
            variables: builder
                .variable_references
                .into_iter()
                .map(|name| {
                    let value = variables.unchecked_get(&name).value();
                    (name, value)
                })
                .collect(),
        })
    }

    fn write_selection_set(&mut self, buffer: &mut Buffer, selection_set: SelectionSetWalker<'_>) -> Result<(), Error> {
        buffer.write_str(" {\n")?;
        buffer.indent += 1;
        for selection in selection_set {
            match selection {
                SelectionWalker::Field(field) => self.write_field(buffer, field)?,
                SelectionWalker::FragmentSpread(spread) => self.write_fragment_spread(buffer, spread)?,
                SelectionWalker::InlineFragment(fragment) => self.write_inline_fragment(buffer, fragment)?,
            };
        }
        buffer.indent -= 1;
        buffer.indent_write("}\n")?;
        Ok(())
    }

    fn write_fragment_spread(&mut self, buffer: &mut Buffer, spread: FragmentSpreadWalker<'_>) -> Result<(), Error> {
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
        let name = self.fragment_contents.entry(fragment_buffer).or_insert_with(|| {
            let id = self
                .fragment_last_id
                .entry(fragment.name.to_string())
                .and_modify(|id| *id += 1)
                .or_default();
            format!("{}_{}", fragment.name, id)
        });
        buffer.indent_write(&format!("...{name}\n"))?;
        Ok(())
    }

    fn write_inline_fragment(&mut self, buffer: &mut Buffer, fragment: InlineFragmentWalker<'_>) -> Result<(), Error> {
        buffer.indent_write("...")?;
        if let Some(name) = fragment.type_condition_name() {
            buffer.write_str(&format!(" on {name}"))?;
        }
        self.write_selection_set(buffer, fragment.selection_set())?;
        Ok(())
    }

    fn write_field(&mut self, buffer: &mut Buffer, field: FieldWalker<'_>) -> Result<(), Error> {
        buffer.indent_write(field.name())?;
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
        arguments: impl ExactSizeIterator<Item = FieldArgumentWalker<'a>>,
    ) -> Result<(), Error> {
        if arguments.len() != 0 {
            buffer.write_str("(")?;

            let mut arguments = arguments.peekable();
            while let Some(argument) = arguments.next() {
                let value = argument.query_value();
                self.add_variable_references(value.variables_used().map(|name| name.to_string()));
                buffer.write_str(argument.name())?;
                buffer.write_str(": ")?;
                buffer.write_str(&value.to_string())?;
                if arguments.peek().is_some() {
                    buffer.write_str(", ")?;
                }
            }
            buffer.write_str(")")?;
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
