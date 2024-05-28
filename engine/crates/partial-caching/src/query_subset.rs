use std::fmt;

use cynic_parser::{
    executable::{
        ids::{FragmentDefinitionId, OperationDefinitionId, SelectionId, VariableDefinitionId},
        iter::Iter,
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

    pub fn is_empty(&self) -> bool {
        self.cache_group.selections.is_empty()
    }

    pub fn extend(&mut self, other: &QuerySubset) {
        self.cache_group
            .selections
            .extend(other.cache_group.selections.iter().copied());
        self.cache_group
            .fragments
            .extend(other.cache_group.fragments.iter().copied());
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
            subset.filter_selection_set(self.document, operation.selection_set())
        )?;

        for id in &subset.cache_group.fragments {
            let fragment = document.read(*id);
            writeln!(
                f,
                "\nfragment {} on {}{} {}",
                fragment.name(),
                fragment.type_condition(),
                fragment.directives(),
                subset.filter_selection_set(document, fragment.selection_set())
            )?;
        }

        Ok(())
    }
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
