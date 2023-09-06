mod collection_args;

use super::PostgresContext;
use crate::{
    registry::{type_kinds::SelectionSetTarget, MetaType},
    Lookahead, SelectionField,
};
pub use collection_args::CollectionArgs;
use postgresql_types::database_definition::{RelationWalker, TableColumnWalker};

pub enum TableSelection<'a> {
    /// Selects a single column.
    Column(TableColumnWalker<'a>),
    /// Joins a unique row with a nested selection.
    JoinUnique(RelationWalker<'a>, SelectionIterator<'a>),
    /// Joins a collection of rows with a nested selection.
    JoinMany(RelationWalker<'a>, SelectionIterator<'a>, CollectionArgs),
}

/// An iterator over a GraphQL selection. Returns either a column or a
/// join, which should be handled accordingly when generating an SQL query.
#[derive(Clone)]
pub struct SelectionIterator<'a> {
    ctx: &'a PostgresContext<'a>,
    meta_type: &'a MetaType,
    selection: Vec<SelectionField<'a>>,
    target: SelectionSetTarget<'a>,
    index: usize,
}

impl<'a> SelectionIterator<'a> {
    pub fn new(ctx: &'a PostgresContext<'a>, meta_type: &'a MetaType, selection: Vec<SelectionField<'a>>) -> Self {
        let target = meta_type.try_into().unwrap();

        Self {
            ctx,
            meta_type,
            selection,
            target,
            index: 0,
        }
    }
}

impl<'a> Iterator for SelectionIterator<'a> {
    type Item = TableSelection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(selection_field) = self.selection.get(self.index) else { return None };

        self.index += 1;

        let table = self
            .ctx
            .database_definition
            .find_table_for_client_type(self.meta_type.name())
            .expect("table for client type not found");

        // Selecting a column.
        if let Some(column) = self
            .ctx
            .database_definition
            .find_column_for_client_field(selection_field.name(), table.id())
        {
            return Some(TableSelection::Column(column));
        }

        // Joining a table with the current one, selecting from the joined table.
        let relation = self
            .ctx
            .database_definition
            .find_relation_for_client_field(selection_field.name(), table.id())
            .expect("column or relation not found for the given field");

        // The type of the relation field.
        let meta_type = self
            .target
            .field(selection_field.name())
            .and_then(|field| self.ctx.registry().lookup_by_str(field.ty.base_type_name()).ok())
            .expect("couldn't find a meta type for a selection");

        if relation.is_referenced_row_unique() {
            // The other side has a unique constraint, so our join must return at most one row.
            let selection_set = selection_field.selection_set().collect();
            let iterator = Self::new(self.ctx, meta_type, selection_set);

            Some(TableSelection::JoinUnique(relation, iterator))
        } else {
            // The other side has not a unique constraint that matches with the foreign key,
            // meaning the resulting set is a collection.

            // `userCollection { edges { node { field } } }`, the selection part.
            let selection_set = Lookahead::from(selection_field)
                .field("edges")
                .field("node")
                .iter_selection_fields()
                .flat_map(|selection| selection.selection_set())
                .collect();

            // `userCollection { edges { node { field } } }`, the type part.
            let meta_type = meta_type
                .field_by_name("edges")
                .and_then(|field| self.ctx.registry().lookup(&field.ty).ok())
                .as_ref()
                .and_then(|output| output.field("node"))
                .and_then(|field| self.ctx.registry().lookup_by_str(field.ty.base_type_name()).ok())
                .expect("couldn't find a meta type for a collection selection");

            let iterator = Self::new(self.ctx, meta_type, selection_set);

            // By defining this, we mark the next select to return a collecton.
            let args = CollectionArgs::new(self.ctx, relation.referenced_table(), selection_field);

            Some(TableSelection::JoinMany(relation, iterator, args))
        }
    }
}
