pub mod collection_args;

pub use collection_args::CollectionArgs;
use postgres_connector_types::database_definition::{RelationWalker, TableColumnWalker, TableWalker};

use super::PostgresContext;
use crate::{
    registry::type_kinds::{OutputType, SelectionSetTarget},
    Error, Lookahead, SelectionField,
};

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
    table: TableWalker<'a>,
    selection: Vec<SelectionField<'a>>,
    extra_columns: Vec<TableColumnWalker<'a>>,
    target: SelectionSetTarget<'a>,
    index: usize,
    extra_column_index: usize,
}

impl<'a> SelectionIterator<'a> {
    pub fn new(
        ctx: &'a PostgresContext<'a>,
        meta_type: OutputType<'a>,
        selection_field: &SelectionField<'_>,
        selection: Vec<SelectionField<'a>>,
    ) -> Self {
        let target: SelectionSetTarget<'a> = meta_type.try_into().unwrap();

        let table = ctx
            .database_definition
            .find_table_for_client_type(target.name())
            .expect("table for client type not found");

        let mut extra_columns = Vec::new();

        match selection_field
            .field
            .get_argument("orderBy")
            .and_then(|value| value.as_slice())
        {
            Some(order_by) => {
                for value in order_by {
                    let object = match value {
                        engine_value::Value::Object(obj) => obj,
                        _ => continue,
                    };

                    for field in object.keys() {
                        if selection
                            .iter()
                            .any(|select| select.field.name.as_str() == field.as_str())
                        {
                            continue;
                        }

                        let column = ctx
                            .database_definition
                            .find_column_for_client_field(&field, table.id())
                            .expect("ordering with non-existing column");

                        extra_columns.push(column);
                    }
                }
            }
            None => {
                for column in table.implicit_ordering_key().unwrap().columns() {
                    if selection
                        .iter()
                        .any(|select| select.field.name.as_str() == column.table_column().client_name())
                    {
                        continue;
                    }

                    extra_columns.push(column.table_column());
                }
            }
        }

        Self {
            ctx,
            table,
            selection,
            extra_columns,
            target,
            index: 0,
            extra_column_index: 0,
        }
    }
}

impl<'a> Iterator for SelectionIterator<'a> {
    type Item = Result<TableSelection<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(selection_field) = self.selection.get(self.index) else {
            let extra = self.extra_columns.get(self.extra_column_index);
            self.extra_column_index += 1;

            return extra.map(|column| Ok(TableSelection::Column(*column)));
        };

        self.index += 1;

        // Selecting a column.
        if let Some(column) = self
            .ctx
            .database_definition
            .find_column_for_client_field(selection_field.name(), self.table.id())
        {
            return Some(Ok(TableSelection::Column(column)));
        }

        // Joining a table with the current one, selecting from the joined table.
        let relation = match self
            .ctx
            .database_definition
            .find_relation_for_client_field(selection_field.name(), self.table.id())
        {
            Some(relation) => relation,
            None => return self.next(),
        };

        // The type of the relation field.
        let meta_type = self
            .target
            .field(selection_field.name())
            .and_then(|field| self.ctx.registry().lookup_expecting(&field.ty).ok())
            .expect("couldn't find a meta type for a selection");

        if relation.is_referenced_row_unique() {
            // The other side has a unique constraint, so our join must return at most one row.
            let selection_set = selection_field.selection_set().collect();
            let iterator = Self::new(self.ctx, meta_type, selection_field, selection_set);

            Some(Ok(TableSelection::JoinUnique(relation, iterator)))
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
                .field("edges")
                .and_then(|field| self.ctx.registry().lookup(&field.ty).ok())
                .as_ref()
                .and_then(|output| output.field("node"))
                .and_then(|field| self.ctx.registry().lookup_expecting(&field.ty).ok())
                .expect("couldn't find a meta type for a collection selection");

            let iterator = Self::new(self.ctx, meta_type, selection_field, selection_set);

            // By defining this, we mark the next select to return a collecton.
            let args = CollectionArgs::new(
                self.ctx.database_definition,
                relation.referenced_table(),
                selection_field,
            );

            match args {
                Ok(args) => Some(Ok(TableSelection::JoinMany(relation, iterator, args))),
                Err(error) => Some(Err(error)),
            }
        }
    }
}
