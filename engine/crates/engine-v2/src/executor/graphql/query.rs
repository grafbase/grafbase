use std::{collections::HashSet, fmt::Write};

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::{Names, Schema};

use crate::request::{OperationArgument, OperationFields, OperationSelection, OperationSelectionSet, TypeCondition};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

pub struct QueryBuilder<'a> {
    schema: &'a Schema,
    names: &'a dyn Names,
    fields: &'a OperationFields,
    indent: usize,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(schema: &'a Schema, names: &'a dyn Names, fields: &'a OperationFields) -> Self {
        Self {
            schema,
            names,
            fields,
            indent: 0,
        }
    }

    pub fn build(&self, operation_type: OperationType, selection_set: &OperationSelectionSet) -> Result<String, Error> {
        let mut buffer = Buffer::new();
        self.write_selection_set(&mut buffer, selection_set)?;

        let mut out = String::new();
        match operation_type {
            OperationType::Query => write!(out, "query ")?,
            OperationType::Mutation => write!(out, "mutation ")?,
            OperationType::Subscription => write!(out, "subscription ")?,
        };

        let operation_name = "Subgraph"; // TODO: should re-use the real operation name
        out.push_str(operation_name);
        out.push_str(&buffer.inner);

        Ok(out)
    }

    fn write_selection_set(&self, buffer: &mut Buffer, selection_set: &OperationSelectionSet) -> Result<(), Error> {
        if !selection_set.is_empty() {
            buffer.write(" {\n")?;
            buffer.indent += 1;
            let gouped_by_type_condition = selection_set
                .iter()
                .map(|selection| (&self.fields[selection.operation_field_id].type_condition, selection))
                .group_by(|(type_condition, _)| *type_condition);
            for (type_condition, selections) in &gouped_by_type_condition {
                let selections = selections.into_iter().map(|(_, selection)| selection);
                if let Some(type_condition) = type_condition {
                    self.write_inline_fragment(buffer, *type_condition, selections)?;
                } else {
                    for selection in selections {
                        self.write_selection(buffer, selection)?;
                    }
                }
            }
            buffer.indent -= 1;
            buffer.indent_write("}\n")?;
        }
        Ok(())
    }

    fn write_inline_fragment<'b>(
        &self,
        buffer: &mut Buffer,
        type_condition: TypeCondition,
        selections: impl IntoIterator<Item = &'b OperationSelection>,
    ) -> Result<(), Error> {
        buffer.indent_write("... on ")?;
        buffer.write(match type_condition {
            TypeCondition::Interface(interface_id) => self.names.interface(interface_id),
            TypeCondition::Object(object_id) => self.names.object(object_id),
            TypeCondition::Union(union_id) => self.names.union(union_id),
        })?;
        buffer.write(" {\n")?;
        buffer.indent += 1;
        for selection in selections {
            self.write_selection(buffer, selection)?;
        }
        buffer.indent -= 1;
        buffer.indent_write("}\n")?;
        Ok(())
    }

    fn write_selection(&self, buffer: &mut Buffer, selection: &OperationSelection) -> Result<(), Error> {
        let field = &self.fields[selection.operation_field_id];
        let name = self.names.field(field.field_id);
        buffer.indent_write(name)?;
        self.write_selection_set(buffer, &selection.subselection)?;
        Ok(())
    }

    fn write_arguments(&self, buffer: &mut Buffer, arguments: &Vec<OperationArgument>) -> Result<(), Error> {
        if !arguments.is_empty() {
            buffer.write("(")?;

            let mut arguments = arguments.iter().peekable();
            while let Some(argument) = arguments.next() {
                buffer.add_variable_references(argument.value.variables_used().map(|name| name.to_string()));
                buffer.write(&self.schema[argument.name])?;
                buffer.write(": ")?;
                buffer.write(argument.value.to_string())?;
                if arguments.peek().is_some() {
                    buffer.write(", ")?;
                }
            }
            buffer.write(")")?;
        }
        Ok(())
    }
}

struct Buffer {
    inner: String,
    indent: usize,
    variable_references: HashSet<String>,
}

impl Buffer {
    fn new() -> Self {
        Self {
            inner: String::new(),
            indent: 0,
            variable_references: HashSet::new(),
        }
    }

    fn add_variable_references(&mut self, names: impl IntoIterator<Item = String>) {
        self.variable_references.extend(names);
    }

    fn indent_write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.write(&"\t".repeat(self.indent))?;
        self.write(s)
    }

    fn write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.inner.write_str(s.as_ref())?;
        Ok(())
    }
}
