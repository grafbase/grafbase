mod async_graphql;

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectionId(usize);

/// A GraphQL operation (query) for operation checks.
#[derive(Debug)]
pub struct Operation {
    /// fragment name -> fragment
    pub(crate) fragments: HashMap<String, Fragment>,

    pub(crate) operation_type: OperationType,
    pub(crate) root_selection: SelectionId,

    /// (parent selection, selection)
    pub(crate) selections: Vec<(SelectionId, Selection)>,

    pub(crate) enum_values_in_variable_defaults: Vec<String>,
}

#[derive(Debug)]
pub struct Fragment {
    pub type_condition: String,
    pub selection: SelectionId,
}

#[derive(Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug)]
pub(crate) struct Argument {
    pub(crate) name: String,
    /// Some(_) if the argument value is an enum literal.
    pub(crate) enum_literal_value: Option<String>,
}

#[derive(Debug)]
pub(crate) enum Selection {
    Field {
        field_name: String,
        arguments: Vec<Argument>,
        subselection: Option<SelectionId>,
    },
    FragmentSpread {
        fragment_name: String,
    },
    InlineFragment {
        on: Option<String>,
        selection: SelectionId,
    },
}
