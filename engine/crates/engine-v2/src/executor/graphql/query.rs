use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use engine_parser::types::OperationType;

use crate::{
    plan::{
        PlanId, PlannedFieldWalker, PlannedFragmentSpreadWalker, PlannedInlineFragmentWalker,
        PlannedSelectionSetWalker, PlannedSelectionWalker,
    },
    request::{Operation, OperationFieldArgumentWalker},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

#[derive(Default)]
pub struct QueryBuilder {
    fragments: HashMap<String, Buffer>,
    variable_references: HashSet<String>,
}

impl QueryBuilder {
    pub fn build(
        operation: &Operation,
        plan_id: PlanId,
        selection_set: PlannedSelectionSetWalker<'_>,
    ) -> Result<String, Error> {
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
        out.push_str(&query);
        for fragment in builder.fragments.into_values() {
            out.push('\n');
            out.push_str(&fragment);
        }

        Ok(out)
    }

    fn write_selection_set(
        &mut self,
        buffer: &mut Buffer,
        selection_set: PlannedSelectionSetWalker<'_>,
    ) -> Result<(), Error> {
        buffer.write(" {\n")?;
        buffer.indent += 1;
        for selection in selection_set {
            match selection {
                PlannedSelectionWalker::Field(field) => self.write_field(buffer, field)?,
                PlannedSelectionWalker::FragmentSpread(spread) => self.write_fragment_spread(buffer, spread)?,
                PlannedSelectionWalker::InlineFragment(fragment) => self.write_inline_fragment(buffer, fragment)?,
            };
        }
        buffer.indent -= 1;
        buffer.indent_write("}\n")?;
        Ok(())
    }

    fn write_fragment_spread(
        &mut self,
        buffer: &mut Buffer,
        spread: PlannedFragmentSpreadWalker<'_>,
    ) -> Result<(), Error> {
        let fragment = spread.fragment();
        buffer.write(format!("...{}", fragment.name()))?;
        // Nothing to deal with fragment cycles here, they should have been detected way earlier
        // during query validation.
        if !self.fragments.contains_key(fragment.name()) {
            let mut buffer = Buffer::default();
            buffer.write(format!(
                "fragment {} on {}",
                fragment.name(),
                fragment.type_condition_name()
            ))?;
            self.write_selection_set(&mut buffer, fragment.selection_set())?;
            self.fragments.insert(fragment.name().to_string(), buffer);
        }
        Ok(())
    }

    fn write_inline_fragment(
        &mut self,
        buffer: &mut Buffer,
        fragment: PlannedInlineFragmentWalker<'_>,
    ) -> Result<(), Error> {
        buffer.indent_write("...")?;
        if let Some(name) = fragment.type_condition_name() {
            buffer.write(format!(" on {name}"))?;
        }
        self.write_selection_set(buffer, fragment.selection_set())?;
        Ok(())
    }

    fn write_field(&mut self, buffer: &mut Buffer, field: PlannedFieldWalker<'_>) -> Result<(), Error> {
        buffer.indent_write(field.name())?;
        self.write_arguments(buffer, field.bound_arguments())?;
        if let Some(selection_set) = field.selection_set() {
            self.write_selection_set(buffer, selection_set)?;
        }
        Ok(())
    }

    fn write_arguments<'a>(
        &mut self,
        buffer: &mut Buffer,
        arguments: impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'a>>,
    ) -> Result<(), Error> {
        if arguments.len() != 0 {
            buffer.write("(")?;

            let mut arguments = arguments.peekable();
            while let Some(argument) = arguments.next() {
                let value = argument.query_value();
                self.add_variable_references(value.variables_used().map(|name| name.to_string()));
                buffer.write(argument.name())?;
                buffer.write(": ")?;
                buffer.write(value.to_string())?;
                if arguments.peek().is_some() {
                    buffer.write(", ")?;
                }
            }
            buffer.write(")")?;
        }
        Ok(())
    }

    fn add_variable_references(&mut self, names: impl IntoIterator<Item = String>) {
        self.variable_references.extend(names);
    }
}

#[derive(Default)]
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

impl Buffer {
    fn indent_write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.write(&"\t".repeat(self.indent))?;
        self.write(s)
    }

    fn write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.inner.write_str(s.as_ref())?;
        Ok(())
    }
}
