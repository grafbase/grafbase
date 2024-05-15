use inflector::Inflector;
use serde::{Deserialize, Serialize};

use super::{names::StringId, SchemaId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table<T> {
    pub(super) schema_id: SchemaId,
    pub(super) database_name: T,
    pub(super) client_name: T,
    pub(super) client_field_name: T,
    pub(super) client_field_name_plural: T,
}

impl<T> Copy for Table<T> where T: Copy {}

impl<T> Table<T> {
    pub(crate) fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub(super) fn set_client_name(&mut self, client_name: T) {
        self.client_name = client_name;
    }

    pub(super) fn set_client_field_name(&mut self, client_field_name: T) {
        self.client_field_name = client_field_name;
    }

    pub(super) fn set_client_field_name_plural(&mut self, client_field_name_plural: T) {
        self.client_field_name_plural = client_field_name_plural;
    }
}

impl Table<String> {
    pub fn new(schema_id: SchemaId, name: String) -> Self {
        let client_name = name.to_pascal_case();
        let client_field_name = client_name.to_camel_case();
        let client_field_name_plural = client_field_name.to_plural();

        Self {
            schema_id,
            database_name: name,
            client_name,
            client_field_name,
            client_field_name_plural,
        }
    }

    pub(crate) fn database_name(&self) -> &str {
        &self.database_name
    }

    pub(crate) fn client_name(&self) -> &str {
        &self.client_name
    }

    pub(crate) fn client_field_name(&self) -> &str {
        &self.client_field_name
    }

    pub(crate) fn client_field_name_plural(&self) -> &str {
        &self.client_field_name_plural
    }
}

impl Table<StringId> {
    pub fn new(
        schema_id: SchemaId,
        database_name: StringId,
        client_name: StringId,
        client_field_name: StringId,
        client_field_name_plural: StringId,
    ) -> Self {
        Self {
            schema_id,
            database_name,
            client_name,
            client_field_name,
            client_field_name_plural,
        }
    }

    pub(crate) fn database_name(&self) -> StringId {
        self.database_name
    }

    pub(crate) fn client_name(&self) -> StringId {
        self.client_name
    }

    pub(crate) fn client_field_name(&self) -> StringId {
        self.client_field_name
    }

    pub(crate) fn client_field_name_plural(&self) -> StringId {
        self.client_field_name_plural
    }
}
