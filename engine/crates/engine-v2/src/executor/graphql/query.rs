use std::{collections::HashSet, fmt::Write};

use engine_parser::types::OperationType;

use crate::request::{OperationFieldArgumentWalker, OperationFieldWalker, OperationSelectionSetWalker};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

pub struct QueryBuilder {
    buffer: String,
    indent: usize,
    variable_references: HashSet<String>,
}

impl QueryBuilder {
    pub fn build(
        operation_type: OperationType,
        selection_set: OperationSelectionSetWalker<'_>,
    ) -> Result<String, Error> {
        let mut builder = QueryBuilder {
            buffer: String::new(),
            indent: 0,
            variable_references: HashSet::new(),
        };
        builder.write_selection_set(selection_set)?;

        let mut out = String::new();
        match operation_type {
            OperationType::Query => write!(out, "query ")?,
            OperationType::Mutation => write!(out, "mutation ")?,
            OperationType::Subscription => write!(out, "subscription ")?,
        };

        let operation_name = "Subgraph"; // TODO: should re-use the real operation name
        out.push_str(operation_name);
        out.push_str(&builder.buffer);

        Ok(out)
    }

    fn write_selection_set(&mut self, selection_set: OperationSelectionSetWalker<'_>) -> Result<(), Error> {
        if !selection_set.is_empty() {
            self.write(" {\n")?;
            self.indent += 1;
            for field in selection_set.all_fields() {
                self.write_selection(field)?;
            }
            self.indent -= 1;
            self.indent_write("}\n")?;
        }
        Ok(())
    }

    fn write_selection(&mut self, field: OperationFieldWalker<'_>) -> Result<(), Error> {
        self.indent_write(field.name())?;
        self.write_arguments(field.arguments())?;
        self.write_selection_set(field.subselection())?;
        Ok(())
    }

    fn write_arguments<'a>(
        &mut self,
        arguments: impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'a>>,
    ) -> Result<(), Error> {
        if arguments.len() != 0 {
            self.write("(")?;

            let mut arguments = arguments.peekable();
            while let Some(argument) = arguments.next() {
                let value = argument.query_value();
                self.add_variable_references(value.variables_used().map(|name| name.to_string()));
                self.write(argument.name())?;
                self.write(": ")?;
                self.write(value.to_string())?;
                if arguments.peek().is_some() {
                    self.write(", ")?;
                }
            }
            self.write(")")?;
        }
        Ok(())
    }

    fn add_variable_references(&mut self, names: impl IntoIterator<Item = String>) {
        self.variable_references.extend(names);
    }

    fn indent_write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.write(&"\t".repeat(self.indent))?;
        self.write(s)
    }

    fn write(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.buffer.write_str(s.as_ref())?;
        Ok(())
    }
}
