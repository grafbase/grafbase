use std::borrow::Cow;

use inflector::Inflector;

use super::{TableWalker, Walker};
use crate::database_definition::{
    names::StringId, table_column::IdentityGeneration, ColumnType, DatabaseType, TableColumn, TableColumnId,
};

/// Definition of a column located in a table.
pub type TableColumnWalker<'a> = Walker<'a, TableColumnId>;

impl<'a> TableColumnWalker<'a> {
    /// The table this column is located.
    pub fn table(self) -> TableWalker<'a> {
        self.walk(self.get().table_id())
    }

    /// The name of the column in the database.
    pub fn database_name(self) -> &'a str {
        self.get_name(self.get().database_name())
    }

    /// The name of the column in the GraphQL APIs.
    pub fn client_name(self) -> &'a str {
        self.get_name(self.get().client_name())
    }

    /// The type of the column in the database.
    pub fn database_type(self) -> DatabaseType<'a> {
        match self.get().database_type() {
            ColumnType::Scalar(scalar) => DatabaseType::Scalar(scalar),
            ColumnType::Enum(enum_id) => DatabaseType::Enum(self.walk(enum_id)),
        }
    }

    /// The base type without possible array notation.
    pub fn graphql_base_type(self, prefix: Option<&str>) -> Option<String> {
        self.graphql_type(prefix)
            .map(|graphql_type| graphql_type.trim_start_matches('[').trim_end_matches(']').to_string())
    }

    /// The type of this column in the GraphQL APIs.
    ///
    /// Returns `None`, if we don't support the database type yet.
    pub fn graphql_type(self, prefix: Option<&str>) -> Option<Cow<'a, str>> {
        match self.database_type() {
            DatabaseType::Scalar(scalar) => scalar.client_type().map(Cow::from),
            DatabaseType::Enum(r#enum) if self.is_array() => Some(Cow::from(match prefix {
                Some(prefix) => format!("[{prefix}_{}]", r#enum.client_name()).to_pascal_case(),
                None => format!("[{}]", r#enum.client_name()),
            })),
            DatabaseType::Enum(r#enum) => Some(match prefix {
                Some(prefix) => Cow::from(format!("{prefix}_{}", r#enum.client_name()).to_pascal_case()),
                None => Cow::from(r#enum.client_name()),
            }),
        }
    }

    pub fn has_supported_type(self) -> bool {
        self.graphql_type(None).is_some()
    }

    /// True, if the column allows null values.
    pub fn nullable(self) -> bool {
        self.get().nullable()
    }

    /// True, if the column has a default value defined.
    pub fn has_default(self) -> bool {
        self.get().has_default() || self.identity_generation().is_some()
    }

    /// True, if the column is an array.
    pub fn is_array(self) -> bool {
        self.get().is_array()
    }

    /// True, if user can define the column value manually.
    pub fn allows_user_input(self) -> bool {
        !matches!(self.identity_generation(), Some(IdentityGeneration::Always))
    }

    fn identity_generation(self) -> Option<IdentityGeneration> {
        self.get().identity_generation()
    }

    fn get(self) -> &'a TableColumn<StringId> {
        &self.database_definition.table_columns[self.id.0 as usize]
    }
}
