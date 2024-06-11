use std::fmt;

use cynic_parser::{
    executable::{ids::SelectionId, iter::Iter, Selection},
    ExecutableDocument,
};
use indexmap::IndexSet;

use super::{FilteredSelectionSet, QuerySubset};

pub struct QuerySubsetDisplay<'a> {
    pub(super) subset: &'a QuerySubset,
    pub(super) document: &'a ExecutableDocument,
    pub(super) include_query_name: bool,
}

impl QuerySubsetDisplay<'_> {
    pub fn include_query_name(self) -> Self {
        QuerySubsetDisplay {
            include_query_name: true,
            ..self
        }
    }
}

pub(super) struct SelectionSetDisplay<'a> {
    pub(super) selections: FilteredSelectionSet<'a, 'a>,
    pub(super) visible_selections: &'a IndexSet<SelectionId>,
    pub(super) indent_level: usize,
}

struct SelectionDisplay<'a> {
    visible_selections: &'a IndexSet<SelectionId>,
    selection: Selection<'a>,
    indent_level: usize,
}

impl<'a> SelectionDisplay<'a> {
    fn wrap_set(&self, selections: Iter<'a, Selection<'a>>) -> SelectionSetDisplay<'a> {
        SelectionSetDisplay {
            visible_selections: self.visible_selections,
            selections: FilteredSelectionSet {
                visible_ids: self.visible_selections,
                selections: selections.with_ids(),
            },
            indent_level: self.indent_level,
        }
    }
}

macro_rules! write_indent {
    ($f:expr, $level:expr) => {
        write!($f, "{:indent$}", "", indent = $level * 2)
    };
}

impl std::fmt::Display for QuerySubsetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let QuerySubsetDisplay {
            subset,
            document,
            include_query_name,
        } = self;

        write!(f, "query")?;

        let operation = document.read(subset.operation);

        if *include_query_name {
            if let Some(name) = operation.name() {
                write!(f, " {name}")?;
            }
        }

        if !subset.variables.is_empty() {
            write!(f, "(")?;
            for (index, id) in subset.variables.iter().enumerate() {
                let prefix = if index != 0 { ", " } else { "" };
                write!(f, "{prefix}{}", document.read(*id))?;
            }
            write!(f, ")")?;
        }
        writeln!(
            f,
            "{} {}",
            operation.directives(),
            subset.selection_set_display(operation.selection_set())
        )?;

        for id in &subset.partition.fragments {
            let fragment = document.read(*id);
            writeln!(
                f,
                "\nfragment {} on {}{} {}",
                fragment.name(),
                fragment.type_condition(),
                fragment.directives(),
                subset.selection_set_display(fragment.selection_set())
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for SelectionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut selections = self.selections.peekable();
        if selections.peek().is_none() {
            return Ok(());
        }
        writeln!(f, "{{")?;
        for selection in selections {
            writeln!(
                f,
                "{}",
                SelectionDisplay {
                    visible_selections: self.visible_selections,
                    selection,
                    indent_level: self.indent_level + 1
                }
            )?;
        }
        write_indent!(f, self.indent_level)?;
        write!(f, "}}")
    }
}

impl fmt::Display for SelectionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_indent!(f, self.indent_level)?;
        match self.selection {
            Selection::Field(field) => {
                if let Some(alias) = field.alias() {
                    write!(f, "{}: ", alias)?;
                }

                let space = if field.selection_set().len() != 0 { " " } else { "" };

                write!(
                    f,
                    "{}{}{}{space}{}",
                    field.name(),
                    field.arguments(),
                    field.directives(),
                    self.wrap_set(field.selection_set())
                )
            }
            Selection::InlineFragment(fragment) => {
                write!(f, "...")?;

                if let Some(on_type) = fragment.type_condition() {
                    write!(f, " on {}", on_type)?;
                }

                write!(
                    f,
                    "{} {}",
                    fragment.directives(),
                    self.wrap_set(fragment.selection_set())
                )
            }
            Selection::FragmentSpread(spread) => {
                write!(f, "{spread}")
            }
        }
    }
}
