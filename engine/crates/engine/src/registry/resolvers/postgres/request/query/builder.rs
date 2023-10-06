use crate::registry::resolvers::postgres::context::{CollectionArgs, FilterIterator, SelectionIterator};
use postgres_types::database_definition::{RelationWalker, TableWalker};
use std::borrow::Cow;

/// A builder for building a Postgres `SELECT` statement.
#[derive(Clone)]
pub struct SelectBuilder<'a> {
    table: TableWalker<'a>,
    selection: SelectionIterator<'a>,
    filter: Option<FilterIterator<'a>>,
    collection_args: Option<CollectionArgs>,
    field_name: Cow<'static, str>,
    relation: Option<RelationWalker<'a>>,
}

impl<'a> SelectBuilder<'a> {
    /// Starting from the given table, select the fields in the iterator
    /// and name the selection with `field_name`.
    pub fn new(
        table: TableWalker<'a>,
        selection: SelectionIterator<'a>,
        field_name: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            table,
            selection,
            filter: None,
            collection_args: None,
            field_name: field_name.into(),
            relation: None,
        }
    }

    /// Adds a `WHERE` clause to the statement.
    pub fn set_filter(&mut self, filter: FilterIterator<'a>) {
        self.filter = Some(filter);
    }

    /// If defining collection arguments to the query, it sets the
    /// result to be an array of rows, and allows defining the relay
    /// arguments with first/last/before/after and orderBy.
    pub fn set_collection_args(&mut self, args: CollectionArgs) {
        self.collection_args = Some(args);
    }

    /// Marks the query as a selection for a relation.
    pub fn set_relation(&mut self, relation: RelationWalker<'a>) {
        self.relation = Some(relation);
    }

    /// The name of the table we're selecting from.
    pub fn table(&self) -> TableWalker<'a> {
        self.table
    }

    /// The selected fields from the user.
    pub fn selection(&self) -> SelectionIterator<'a> {
        self.selection.clone()
    }

    /// How we name the result of this query. Set to `root` if generating the main query,
    /// and to the name of the relation field if creating a select for a join.
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// The arguments to define how multiple rows should be fetched.
    pub fn collection_args(&self) -> Option<&CollectionArgs> {
        self.collection_args.as_ref()
    }

    /// The `WHERE` statement for this select.
    pub fn filter(&self) -> Option<FilterIterator<'a>> {
        self.filter.clone()
    }

    /// If selecting for a join, this should have the definition of the relation we're
    /// currently on.
    pub fn relation(&self) -> Option<RelationWalker<'a>> {
        self.relation
    }
}
