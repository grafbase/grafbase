use std::fmt;

use cynic_parser::{
    executable::{
        ids::{FragmentDefinitionId, OperationDefinitionId, SelectionId, VariableDefinitionId},
        iter::Iter,
        Selection,
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
    operation: OperationDefinitionId,
    cache_group: CacheGroup,
    variables: Vec<VariableDefinitionId>,
}

#[derive(Default)]
pub(crate) struct CacheGroup {
    pub selections: IndexSet<SelectionId>,
    pub fragments: IndexSet<FragmentDefinitionId>,
}

impl QuerySubset {
    pub(crate) fn new(
        operation: OperationDefinitionId,
        cache_group: CacheGroup,
        _document: &ExecutableDocument,
    ) -> Self {
        QuerySubset {
            operation,
            cache_group,

            // TODO: do this
            variables: vec![],
        }
    }

    pub fn write(&self, document: &ExecutableDocument, target: &mut dyn fmt::Write) -> fmt::Result {
        // Note: be careful about putting query names in this output
        // This output is used to build a cache key, and we don't want query names to cause
        // queries that could otherwise share a cache entry to not do so
        write!(target, "query")?;

        if !self.variables.is_empty() {
            write!(target, "(")?;
            for (index, id) in self.variables.iter().enumerate() {
                let prefix = if index != 0 { ", " } else { "" };
                write!(target, "{prefix}{}", document.read(*id))?;
            }
            write!(target, ")")?;
        }
        let operation = document.read(self.operation);
        writeln!(
            target,
            "{} {}",
            operation.directives(),
            self.filter_selection_set(document, operation.selection_set())
        )?;

        for id in &self.cache_group.fragments {
            let fragment = document.read(*id);
            writeln!(
                target,
                "\nfragment {} on {}{} {}",
                fragment.name(),
                fragment.type_condition(),
                fragment.directives(),
                self.filter_selection_set(document, fragment.selection_set())
            )?;
        }

        Ok(())
    }

    fn filter_selection_set<'a>(
        &'a self,
        document: &'a ExecutableDocument,
        selections: Iter<'a, Selection<'a>>,
    ) -> FilteredSelectionSet<'a> {
        FilteredSelectionSet {
            document,
            visible_selections: &self.cache_group.selections,
            selections,
            indent_level: 0,
        }
    }
}

struct FilteredSelectionSet<'a> {
    document: &'a ExecutableDocument,
    visible_selections: &'a IndexSet<SelectionId>,
    selections: Iter<'a, Selection<'a>>,
    indent_level: usize,
}

impl<'a> FilteredSelectionSet<'a> {
    fn iter<'b>(&'b self) -> impl Iterator<Item = FilteredSelection<'a>> + 'b {
        self.selections
            .ids()
            .filter(|id| self.visible_selections.contains(id))
            .map(|id| FilteredSelection {
                document: self.document,
                visible_selections: self.visible_selections,
                selection: self.document.read(id),
                indent_level: self.indent_level + 1,
            })
    }
}

macro_rules! write_indent {
    ($f:expr, $level:expr) => {
        write!($f, "{:indent$}", "", indent = $level * 2)
    };
}

impl fmt::Display for FilteredSelectionSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.selections.len() == 0 {
            return Ok(());
        }
        writeln!(f, "{{")?;
        for selection in self.iter() {
            writeln!(f, "{selection}",)?;
        }
        write_indent!(f, self.indent_level)?;
        write!(f, "}}")
    }
}

struct FilteredSelection<'a> {
    document: &'a ExecutableDocument,
    visible_selections: &'a IndexSet<SelectionId>,
    selection: Selection<'a>,
    indent_level: usize,
}

impl<'a> FilteredSelection<'a> {
    fn wrap_set(&self, selections: Iter<'a, Selection<'a>>) -> FilteredSelectionSet<'a> {
        FilteredSelectionSet {
            document: self.document,
            visible_selections: self.visible_selections,
            selections,
            indent_level: self.indent_level,
        }
    }
}

impl fmt::Display for FilteredSelection<'_> {
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
