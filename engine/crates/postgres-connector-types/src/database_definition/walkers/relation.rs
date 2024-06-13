use std::borrow::Cow;

use inflector::Inflector;
use itertools::Itertools;

use super::{ForeignKeyWalker, TableColumnWalker, TableWalker, Walker};
use crate::database_definition::RelationId;

pub type RelationWalker<'a> = Walker<'a, RelationId>;

impl<'a> RelationWalker<'a> {
    /// The table this relation starts from.
    pub fn referencing_table(self) -> TableWalker<'a> {
        match self.id() {
            RelationId::Forward(id) => self.walk(id).referencing_table(),
            RelationId::Back(id) => self.walk(id).referencing_table(),
        }
    }

    /// The opposite table.
    pub fn referenced_table(self) -> TableWalker<'a> {
        match self.id() {
            RelationId::Forward(id) => self.walk(id).referenced_table(),
            RelationId::Back(id) => self.walk(id).referenced_table(),
        }
    }

    /// The columns on this table that are forming the constraint.
    pub fn referencing_columns(self) -> Box<dyn ExactSizeIterator<Item = TableColumnWalker<'a>> + 'a> {
        match self.id() {
            RelationId::Forward(id) => Box::new(self.walk(id).referencing_columns()),
            RelationId::Back(id) => Box::new(self.walk(id).referencing_columns()),
        }
    }

    /// The columns on the other table that are forming the constraint.
    pub fn referenced_columns(self) -> Box<dyn ExactSizeIterator<Item = TableColumnWalker<'a>> + 'a> {
        match self.id() {
            RelationId::Forward(id) => Box::new(self.walk(id).referenced_columns()),
            RelationId::Back(id) => Box::new(self.walk(id).referenced_columns()),
        }
    }

    /// True, if the referenced column(s) is (are) unique, this means there can only be at most one row on the other side of the relation.
    pub fn is_other_side_one(self) -> bool {
        self.referenced_table()
            .unique_constraints()
            .any(|constraint| constraint.has_all_the_columns(self.referenced_columns()))
    }

    /// True, if all the columns of the relation are of supported type.
    pub fn all_columns_use_supported_types(self) -> bool {
        self.foreign_key().all_columns_use_supported_types()
    }

    /// True, if we use the referenced table in the client. E.g. it has at least one
    /// column of supported type and one unique constraint.
    pub fn referenced_table_is_allowed_in_client(self) -> bool {
        self.referenced_table().allowed_in_client()
    }

    /// The name of the relation field.
    pub fn client_field_name(self) -> String {
        let base_name = if self.is_other_side_one() {
            self.referenced_table().client_field_name()
        } else {
            self.referenced_table().client_field_name_plural()
        };

        let mut is_name_collision = self
            .referencing_table()
            .columns()
            .any(|column| column.client_name() == base_name);

        let fk = self.foreign_key();

        is_name_collision |= {
            let constrained_table = fk.constrained_table();
            let referenced_table = fk.referenced_table();
            constrained_table
                .forward_relations()
                .any(|relation| relation.foreign_key() != fk && relation.referenced_table() == referenced_table)
        };

        if is_name_collision {
            let referencing_columns = fk
                .columns()
                .map(|column| column.constrained_column().client_name())
                .join("_");

            format!("{base_name}_by_{referencing_columns}").to_camel_case()
        } else {
            base_name.to_string()
        }
    }

    /// Is the relation field nullable.
    pub fn nullable(self) -> bool {
        self.referencing_columns().all(|column| column.nullable())
    }

    /// The client type of the relation field.
    pub fn client_type(self) -> Cow<'a, str> {
        let base_name = self.referenced_table().client_name();

        if self.is_other_side_one() {
            Cow::Borrowed(base_name)
        } else {
            Cow::Owned(format!("{base_name}Collection"))
        }
    }

    /// The foreign key backing the relation.
    fn foreign_key(self) -> ForeignKeyWalker<'a> {
        match self.id() {
            RelationId::Forward(id) => self.walk(id).foreign_key(),
            RelationId::Back(id) => self.walk(id).foreign_key(),
        }
    }
}
