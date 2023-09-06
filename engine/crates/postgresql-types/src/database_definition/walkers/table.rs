use super::{
    forward_relation::ForwardRelationWalker, table_column::TableColumnWalker,
    unique_constraint::UniqueConstraintWalker, BackRelationWalker, Walker,
};
use crate::database_definition::{
    ids::{BackRelationId, ForwardRelationId},
    names::StringId,
    Table, TableColumnId, TableId, UniqueConstraintId,
};

/// Definition of a table.
pub type TableWalker<'a> = Walker<'a, TableId>;

impl<'a> TableWalker<'a> {
    /// The name of the schema this table is located.
    pub fn schema(self) -> &'a str {
        &self.database_definition.schemas[self.get().schema_id().0 as usize]
    }

    /// The name of the table in the database.
    pub fn database_name(self) -> &'a str {
        self.get_name(self.get().database_name())
    }

    /// The name of the table in the GraphQL APIs.
    pub fn client_name(self) -> &'a str {
        self.get_name(self.get().client_name())
    }

    /// An iterator over all the columns in the table.
    pub fn columns(self) -> impl Iterator<Item = TableColumnWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.table_columns, self.id, |column| {
            column.table_id()
        });

        range
            .map(move |id| self.walk(TableColumnId(id as u32)))
            .filter(|column| column.has_supported_type())
    }

    /// A table can be used in the client, if it has at least one supported column
    /// and at least one unique constraint that contains columns of supported type.
    pub fn allowed_in_client(self) -> bool {
        self.columns().next().is_some() && self.unique_constraints().next().is_some()
    }

    /// An iterator over all the unqiue constraints, including the primary key.
    pub fn unique_constraints(self) -> impl Iterator<Item = UniqueConstraintWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.unique_constraints, self.id, |constraint| {
            constraint.table_id()
        });

        range
            .map(move |id| self.walk(UniqueConstraintId(id as u32)))
            .filter(|constraint| constraint.all_columns_use_supported_types())
    }

    /// Find a column by name.
    pub fn find_column(self, name: &str) -> Option<TableColumnWalker<'a>> {
        self.database_definition
            .names
            .get_table_column_id(self.id, name)
            .map(|id| self.walk(id))
    }

    /// Find a unique constraint by name.
    pub fn find_unique_constraint(self, constraint_name: &str) -> Option<UniqueConstraintWalker<'a>> {
        self.database_definition
            .names
            .get_unique_constraint_id(self.id, constraint_name)
            .map(|id| self.walk(id))
    }

    /// An iterator over relations having the foreign key defined on this table.
    pub fn forward_relations(self) -> impl Iterator<Item = ForwardRelationWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.relations.from, self.id, |(table_id, _)| {
            *table_id
        });

        range
            .map(move |id| self.walk(ForwardRelationId(id as u32)))
            .filter(|relation| relation.all_columns_use_supported_types())
    }

    /// An iterator over relations having the foreign key defined on the opposite table.
    pub fn backward_relations(self) -> impl Iterator<Item = BackRelationWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.relations.to, self.id, |(table_id, _)| {
            *table_id
        });

        range
            .map(move |id| self.walk(BackRelationId(id as u32)))
            .filter(|relation| relation.all_columns_use_supported_types())
    }

    fn get(self) -> &'a Table<StringId> {
        &self.database_definition.tables[self.id.0 as usize]
    }
}
