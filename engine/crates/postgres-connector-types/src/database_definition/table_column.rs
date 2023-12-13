use inflector::Inflector;
use serde::{Deserialize, Serialize};

use super::{names::StringId, ColumnType, TableId};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum IdentityGeneration {
    /// Cannot insert a custom value to the column, always generated.
    Always,
    /// Can optionally insert a custom value to the column, by default generated.
    ByDefault,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableColumn<T> {
    pub(super) table_id: TableId,
    pub(super) database_name: T,
    pub(super) database_type: ColumnType,
    pub(super) client_name: T,
    pub(super) nullable: bool,
    pub(super) has_default: bool,
    pub(super) is_array: bool,
    #[serde(default)]
    pub(super) identity_generation: Option<IdentityGeneration>,
}

impl<T> TableColumn<T> {
    pub(crate) fn database_type(&self) -> ColumnType {
        self.database_type
    }

    pub(crate) fn has_default(&self) -> bool {
        self.has_default
    }

    pub(crate) fn nullable(&self) -> bool {
        self.nullable
    }

    pub fn set_nullable(&mut self, value: bool) {
        self.nullable = value;
    }

    pub fn set_has_default(&mut self, value: bool) {
        self.has_default = value;
    }

    pub fn set_is_array(&mut self, value: bool) {
        self.is_array = value;
    }

    pub fn set_identity_generation(&mut self, value: impl Into<IdentityGeneration>) {
        self.identity_generation = Some(value.into());
    }

    pub fn identity_generation(&self) -> Option<IdentityGeneration> {
        self.identity_generation
    }

    pub(crate) fn table_id(&self) -> TableId {
        self.table_id
    }

    pub(crate) fn is_array(&self) -> bool {
        self.is_array
    }
}

impl TableColumn<String> {
    pub fn new(table_id: TableId, name: String, database_type: ColumnType) -> Self {
        let client_name = name.to_camel_case();

        Self {
            table_id,
            database_name: name,
            database_type,
            client_name,
            nullable: false,
            has_default: false,
            is_array: false,
            identity_generation: None,
        }
    }

    pub(crate) fn database_name(&self) -> &str {
        &self.database_name
    }

    pub(crate) fn client_name(&self) -> &str {
        &self.client_name
    }
}

impl TableColumn<StringId> {
    pub(crate) fn database_name(&self) -> StringId {
        self.database_name
    }

    pub(crate) fn client_name(&self) -> StringId {
        self.client_name
    }
}
