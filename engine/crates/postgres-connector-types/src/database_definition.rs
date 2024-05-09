mod r#enum;
mod enum_variant;
mod foreign_key;
mod foreign_key_column;
mod ids;
mod names;
mod postgres_type;
mod relations;
mod table;
mod table_column;
mod unique_constraint;
mod unique_constraint_column;
mod vectorize;
mod walkers;

use std::collections::HashMap;

pub use enum_variant::EnumVariant;
pub use foreign_key::ForeignKey;
pub use foreign_key_column::ForeignKeyColumn;
pub use ids::{
    BackRelationId, EnumId, EnumVariantId, ForeignKeyColumnId, ForeignKeyId, ForwardRelationId, RelationId, SchemaId,
    TableColumnId, TableId, UniqueConstraintColumnId, UniqueConstraintId,
};
use inflector::Inflector;
use names::{Names, StringId};
pub use postgres_type::{ColumnType, DatabaseType, ScalarType};
pub use r#enum::Enum;
use relations::Relations;
use serde::{Deserialize, Serialize};
pub use table::Table;
pub use table_column::{IdentityGeneration, TableColumn};
pub use unique_constraint::{ConstraintType, UniqueConstraint};
pub use unique_constraint_column::UniqueConstraintColumn;
pub use walkers::{
    EnumVariantWalker, EnumWalker, RelationWalker, TableColumnWalker, TableWalker, UniqueConstraintColumnWalker,
    UniqueConstraintWalker, Walker,
};

/// Definition of a PostgreSQL database. Contains all the
/// tables, enums, columns, constraints etc. for us to render
/// a GraphQL schema, and for us to allow querying the database
/// efficiently.
///
/// Due to Grafbase dependency tree, mutating this structure
/// outside of introspection is not recommended. Some of the
/// mutations are public, but from the perspective of the user,
/// the important call points are the table and enum iterators,
/// and the find methods with string slices.
///
/// Be aware that this structure is serialized in a cache for
/// fast worker startup. Any changes here must be backwards-compatible.
///
/// There will be a test failure if something changes to alert you.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseDefinition {
    /// The database connection string.
    connection_string: String,
    /// Ordered by name.
    schemas: Vec<String>,
    /// Ordered by schema id, then table name.
    tables: Vec<Table<StringId>>,
    /// Ordered by schema id, table id and then column position.
    table_columns: Vec<TableColumn<StringId>>,
    /// Ordered by schema id, then enum name.
    enums: Vec<Enum<StringId>>,
    /// Ordered by schema id, enum id and finally the variant position.
    enum_variants: Vec<EnumVariant<StringId>>,
    /// Ordered by schema id, table id and foreign key constraint name.
    foreign_keys: Vec<ForeignKey<StringId>>,
    /// Ordered by schema id, table id, foreign key id and the column position.
    foreign_key_columns: Vec<ForeignKeyColumn>,
    /// Ordered by schema id, table id and constraint name.
    unique_constraints: Vec<UniqueConstraint<StringId>>,
    /// Ordered by schema id, table id, constraint id and the column position.
    unique_constraint_columns: Vec<UniqueConstraintColumn>,
    names: Names,
    relations: Relations,
}

impl DatabaseDefinition {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            schemas: Vec::new(),
            tables: Vec::new(),
            table_columns: Vec::new(),
            enums: Vec::new(),
            enum_variants: Vec::new(),
            foreign_keys: Vec::new(),
            foreign_key_columns: Vec::new(),
            unique_constraints: Vec::new(),
            unique_constraint_columns: Vec::new(),
            names: Names::default(),
            relations: Relations::default(),
        }
    }

    /// The connection string this definition is introspected from.
    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }

    /// Iterates over all tables of the introspected database.
    pub fn tables(&self) -> impl ExactSizeIterator<Item = TableWalker<'_>> + '_ {
        (0..self.tables.len()).map(move |id| self.walk(TableId(id as u32)))
    }

    /// Iterates over all enums of the introspected database.
    pub fn enums(&self) -> impl ExactSizeIterator<Item = EnumWalker<'_>> + '_ {
        (0..self.enums.len()).map(move |id| self.walk(EnumId(id as u32)))
    }

    /// Find a table in a specified schema with the specified name.
    pub fn find_table(&self, schema_name: &str, table_name: &str) -> Option<TableWalker<'_>> {
        let schema_id = self.get_schema_id(schema_name)?;

        self.get_table_id(schema_id, table_name)
            .map(|table_id| self.walk(table_id))
    }

    /// Finds an enum in a specified schema with the specified name.
    pub fn find_enum(&self, schema_name: &str, enum_name: &str) -> Option<EnumWalker<'_>> {
        let schema_id = self.get_schema_id(schema_name)?;
        self.get_enum_id(schema_id, enum_name).map(|enum_id| self.walk(enum_id))
    }

    /// Find a table that represents the given client type.
    pub fn find_table_for_client_type(&self, client_type: &str) -> Option<TableWalker<'_>> {
        self.names
            .get_table_id_for_client_type(client_type)
            .map(|table_id| self.walk(table_id))
    }

    /// Find a column that represents the given client field.
    pub fn find_column_for_client_field(&self, client_field: &str, table_id: TableId) -> Option<TableColumnWalker<'_>> {
        self.names
            .get_column_id_for_client_field(client_field, table_id)
            .map(|table_id| self.walk(table_id))
    }

    /// Find a relation that represents the given client field.
    pub fn find_relation_for_client_field(&self, client_field: &str, table_id: TableId) -> Option<RelationWalker<'_>> {
        self.names
            .get_relation_id_for_client_field(client_field, table_id)
            .map(|relation_id| self.walk(relation_id))
    }

    /// Find a unique constraint that represents the given client field.
    pub fn find_unique_constraint_for_client_field(
        &self,
        client_field: &str,
        table_id: TableId,
    ) -> Option<UniqueConstraintWalker<'_>> {
        self.names
            .get_unique_constraint_id_for_client_field(client_field, table_id)
            .map(|constraint_id| self.walk(constraint_id))
    }

    /// Adds a schema to the definition.
    pub fn push_schema(&mut self, schema: String) -> SchemaId {
        let id = self.next_schema_id();
        self.schemas.push(schema);

        id
    }

    /// Adds a table to the definition.
    pub fn push_table(&mut self, table: Table<String>) -> TableId {
        let id = self.next_table_id();
        self.names.intern_table(&table, id);

        self.tables.push(Table {
            schema_id: table.schema_id(),
            database_name: self.names.intern_string(table.database_name()),
            client_name: self.names.intern_string(table.client_name()),
            client_field_name: self.names.intern_string(table.client_field_name()),
            client_field_name_plural: self.names.intern_string(table.client_field_name_plural()),
        });

        id
    }

    /// Adds a table column to the definition.
    pub fn push_table_column(&mut self, column: TableColumn<String>) -> TableColumnId {
        let id = self.next_table_column_id();

        self.names.intern_table_column(&column, id);

        self.table_columns.push(TableColumn {
            table_id: column.table_id(),
            database_name: self.names.intern_string(column.database_name()),
            database_type: column.database_type(),
            client_name: self.names.intern_string(column.client_name()),
            nullable: column.nullable(),
            has_default: column.has_default(),
            is_array: column.is_array(),
            identity_generation: column.identity_generation,
        });

        id
    }

    /// Adds an enum to the definition.
    pub fn push_enum(&mut self, r#enum: Enum<String>) -> EnumId {
        let id = self.next_enum_id();

        self.names.intern_enum(&r#enum, id);

        self.enums.push(Enum {
            schema_id: r#enum.schema_id(),
            database_name: self.names.intern_string(r#enum.database_name()),
            client_name: self.names.intern_string(r#enum.client_name()),
        });

        id
    }

    /// Adds an enum variant to the definition.
    pub fn push_enum_variant(&mut self, enum_variant: EnumVariant<String>) -> EnumVariantId {
        let id = self.next_enum_variant_id();

        self.names.intern_enum_variant(&enum_variant, id);

        self.enum_variants.push(EnumVariant {
            enum_id: enum_variant.enum_id(),
            database_name: self.names.intern_string(enum_variant.database_name()),
            client_name: self.names.intern_string(enum_variant.client_name()),
        });

        id
    }

    /// Adds a foreign key to the definition.
    pub fn push_foreign_key(&mut self, foreign_key: ForeignKey<String>) -> ForeignKeyId {
        let id = self.next_foreign_key_id();

        self.relations.push_relation(&foreign_key, id);
        self.names.intern_foreign_key(&foreign_key, id);

        self.foreign_keys.push(ForeignKey {
            constraint_name: self.names.intern_string(foreign_key.constraint_name()),
            schema_id: foreign_key.schema_id(),
            constrained_table_id: foreign_key.constrained_table_id(),
            referenced_table_id: foreign_key.referenced_table_id(),
        });

        id
    }

    /// Adds a foreign key column to the definition.
    pub fn push_foreign_key_column(&mut self, foreign_key_column: ForeignKeyColumn) -> ForeignKeyColumnId {
        let id = self.next_foreign_key_column_id();
        self.foreign_key_columns.push(foreign_key_column);

        id
    }

    /// Adds a unique constraint to the definition.
    pub fn push_unique_constraint(&mut self, unique_constraint: UniqueConstraint<String>) -> UniqueConstraintId {
        let id = self.next_unique_constraint_id();
        self.names.intern_unique_constraint(&unique_constraint, id);

        self.unique_constraints.push(UniqueConstraint {
            table_id: unique_constraint.table_id(),
            constraint_name: self.names.intern_string(unique_constraint.name()),
            r#type: unique_constraint.r#type,
        });

        id
    }

    /// Adds a unique constraint column to the definition.
    pub fn push_unique_constraint_column(
        &mut self,
        unique_constraint_column: UniqueConstraintColumn,
    ) -> UniqueConstraintColumnId {
        let id = self.next_unique_constraint_column_id();
        self.unique_constraint_columns.push(unique_constraint_column);

        id
    }

    /// Adds an index from client type name to table id.
    pub fn push_client_type_mapping(&mut self, type_name: &str, table_id: TableId) {
        self.names.intern_client_type(type_name, table_id);
    }

    /// Adds an index from client field name and table id to table column id.
    pub fn push_client_field_mapping(&mut self, field_name: &str, table_id: TableId, column_id: TableColumnId) {
        self.names.intern_client_field(field_name, table_id, column_id);
    }

    /// Adds an index from client field name and table id to unique constraint id.
    pub fn push_client_field_unique_constraint_mapping(
        &mut self,
        field_name: &str,
        table_id: TableId,
        constraint_id: UniqueConstraintId,
    ) {
        self.names
            .intern_client_unique_constraint(field_name, table_id, constraint_id);
    }

    /// Adds an index from client enum name to the corresponding enum id.
    pub fn push_client_enum_mapping(&mut self, enum_name: &str, enum_id: EnumId) {
        self.names.intern_client_enum(enum_name, enum_id);
    }

    /// Adds an index from client field name to a forward relation.
    pub fn push_client_relation_mapping(&mut self, field_name: &str, table_id: TableId, relation_id: RelationId) {
        self.names.intern_client_relation(field_name, table_id, relation_id);
    }

    /// Finds the id of a schema with the given name, if existing.
    pub fn get_schema_id(&self, schema: &str) -> Option<SchemaId> {
        self.schemas
            .binary_search_by(|schema_name| schema_name.as_str().cmp(schema))
            .ok()
            .map(|position| SchemaId(position as u32))
    }

    /// Finds the id of a table with the given name, if existing.
    pub fn get_table_id(&self, schema_id: SchemaId, table_name: &str) -> Option<TableId> {
        self.names.get_table_id(schema_id, table_name)
    }

    /// Finds the id of a column in a table with the given name, if existing.
    pub fn get_table_column_id(&self, table_id: TableId, column_name: &str) -> Option<TableColumnId> {
        self.names.get_table_column_id(table_id, column_name)
    }

    /// Finds the id of an enum with the given name, if existing.
    pub fn get_enum_id(&self, schema_id: SchemaId, enum_name: &str) -> Option<EnumId> {
        self.names.get_enum_id(schema_id, enum_name)
    }

    /// Finds the id of an enum with the given name, if existing.
    pub fn get_foreign_key_id(&self, schema_id: SchemaId, constraint_name: &str) -> Option<ForeignKeyId> {
        self.names.get_foreign_key_id(schema_id, constraint_name)
    }

    /// Finds the id of a unique constraint with the given name, if existing.
    pub fn get_unique_constraint_id(&self, table_id: TableId, constraint_name: &str) -> Option<UniqueConstraintId> {
        self.names.get_unique_constraint_id(table_id, constraint_name)
    }

    /// Finalizes the definition. Handles name deduplication, and sorts the internal data structures
    /// accordingly.
    pub fn finalize(&mut self) {
        self.deduplicate_names();

        self.relations.from.sort_by_key(|(table_id, _)| *table_id);
        self.relations.to.sort_by_key(|(table_id, _)| *table_id);
    }

    /// Walk an item in the definition by its ID.
    pub fn walk<Id>(&self, id: Id) -> Walker<'_, Id> {
        Walker {
            id,
            database_definition: self,
        }
    }

    /// Tables and enums are namespaced per schema in PostgreSQL, but in GraphQL all schemas are in the same namespace.
    ///
    /// If a table or enum has a duplicate name in different schemas, we'll prefix the name with the name of the schema.
    fn deduplicate_names(&mut self) {
        let mut names = HashMap::new();

        for table in &self.tables {
            let counter = names.entry(table.client_name()).or_default();
            *counter += 1;
        }

        for table in &mut self.tables {
            if names.get(&table.client_name()).copied().unwrap_or(0) < 2 {
                continue;
            }

            let schema_name = &self.schemas[table.schema_id().0 as usize];
            let client_name = self.names.get_name(table.client_name());

            let new_client_name = format!("{schema_name}_{client_name}").to_pascal_case();
            let client_name = self.names.intern_string(&new_client_name);

            let new_client_field_name = self.names.intern_string(&new_client_name.to_camel_case());
            let new_client_field_name_plural = self.names.intern_string(&new_client_name.to_camel_case().to_plural());

            table.set_client_name(client_name);
            table.set_client_field_name(new_client_field_name);
            table.set_client_field_name_plural(new_client_field_name_plural);
        }

        names.clear();

        for r#enum in &self.enums {
            let counter = names.entry(r#enum.client_name()).or_default();
            *counter += 1;
        }

        for r#enum in &mut self.enums {
            if names.get(&r#enum.client_name()).copied().unwrap_or(0) < 2 {
                continue;
            }

            let schema_name = &self.schemas[r#enum.schema_id().0 as usize];
            let client_name = self.names.get_name(r#enum.client_name());

            let client_name = self
                .names
                .intern_string(&format!("{schema_name}_{client_name}").to_pascal_case());

            r#enum.set_client_name(client_name);
        }
    }

    fn next_schema_id(&self) -> SchemaId {
        SchemaId(self.schemas.len() as u32)
    }

    fn next_table_id(&self) -> TableId {
        TableId(self.tables.len() as u32)
    }

    fn next_table_column_id(&self) -> TableColumnId {
        TableColumnId(self.table_columns.len() as u32)
    }

    fn next_enum_id(&self) -> EnumId {
        EnumId(self.enums.len() as u32)
    }

    fn next_enum_variant_id(&self) -> EnumVariantId {
        EnumVariantId(self.enum_variants.len() as u32)
    }

    fn next_foreign_key_id(&self) -> ForeignKeyId {
        ForeignKeyId(self.foreign_keys.len() as u32)
    }

    fn next_foreign_key_column_id(&self) -> ForeignKeyColumnId {
        ForeignKeyColumnId(self.foreign_key_columns.len() as u32)
    }

    fn next_unique_constraint_id(&self) -> UniqueConstraintId {
        UniqueConstraintId(self.unique_constraints.len() as u32)
    }

    fn next_unique_constraint_column_id(&self) -> UniqueConstraintColumnId {
        UniqueConstraintColumnId(self.unique_constraint_columns.len() as u32)
    }
}
