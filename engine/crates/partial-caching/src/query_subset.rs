use std::fmt;

use cynic_parser::{
    executable::{
        ids::{FragmentDefinitionId, OperationDefinitionId, SelectionId, VariableDefinitionId},
        iter::{IdIter, Iter},
        Selection, VariableDefinition,
    },
    ExecutableDocument,
};
use indexmap::IndexSet;

/// Part of a query that was submitted to the API.
///
/// This is a group of fields with the same cache settings, and all the
/// ancestors, variables & fragments required for those fields to make a
/// valid query
pub struct QuerySubset {
    pub(crate) operation: OperationDefinitionId,
    partition: Partition,
    variables: IndexSet<VariableDefinitionId>,
}

#[derive(Default, Debug)]
pub(crate) struct Partition {
    pub selections: IndexSet<SelectionId>,
    pub fragments: IndexSet<FragmentDefinitionId>,
}

impl QuerySubset {
    pub(crate) fn new(
        operation: OperationDefinitionId,
        cache_group: Partition,
        variables: IndexSet<VariableDefinitionId>,
    ) -> Self {
        QuerySubset {
            operation,
            partition: cache_group,
            variables,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.partition.selections.is_empty()
    }

    pub fn extend(&mut self, other: &QuerySubset) {
        self.partition
            .selections
            .extend(other.partition.selections.iter().copied());
        self.partition
            .fragments
            .extend(other.partition.fragments.iter().copied());
        self.variables.extend(other.variables.iter().cloned());
    }

    pub fn as_display<'a>(&'a self, document: &'a ExecutableDocument) -> QuerySubsetDisplay<'a> {
        QuerySubsetDisplay {
            subset: self,
            document,
            include_query_name: false,
        }
    }

    pub fn variables<'a>(
        &'a self,
        document: &'a ExecutableDocument,
    ) -> impl Iterator<Item = VariableDefinition<'a>> + 'a {
        self.variables.iter().map(|id| document.read(*id))
    }

    fn selection_set_display<'a>(&'a self, selections: Iter<'a, Selection<'a>>) -> SelectionSetDisplay<'a> {
        SelectionSetDisplay {
            visible_selections: &self.partition.selections,
            selections: self.selection_iter(selections),
            indent_level: 0,
        }
    }

    pub(crate) fn selection_iter<'a>(&'a self, selection_set: Iter<'a, Selection<'a>>) -> FilteredSelections<'a> {
        FilteredSelections {
            visible_ids: &self.partition.selections,
            selections: selection_set.with_ids(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FilteredSelections<'a> {
    visible_ids: &'a IndexSet<SelectionId>,
    selections: IdIter<'a, Selection<'a>>,
}

impl<'a> Iterator for FilteredSelections<'a> {
    type Item = Selection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for (id, selection) in self.selections.by_ref() {
            if self.visible_ids.contains(&id) {
                return Some(selection);
            }
        }
        None
    }
}

pub struct QuerySubsetDisplay<'a> {
    subset: &'a QuerySubset,
    document: &'a ExecutableDocument,
    include_query_name: bool,
}

impl QuerySubsetDisplay<'_> {
    pub fn include_query_name(self) -> Self {
        QuerySubsetDisplay {
            include_query_name: true,
            ..self
        }
    }
}

struct SelectionSetDisplay<'a> {
    selections: FilteredSelections<'a>,
    visible_selections: &'a IndexSet<SelectionId>,
    indent_level: usize,
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
            selections: FilteredSelections {
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
