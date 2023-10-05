mod builders;

use engine::registry::{EnumType, InputObjectType, MetaField, ObjectType};
use inflector::Inflector;
use parser_sdl::Registry;
use postgres_types::database_definition::{
    DatabaseDefinition, EnumId, RelationId, TableColumnId, TableId, UniqueConstraintId,
};

pub use self::builders::{EnumBuilder, InputTypeBuilder, ObjectTypeBuilder};

#[derive(Debug)]
pub struct OutputContext {
    query_type_name: String,
    mutation_type_name: String,
    registry: Registry,
    type_mapping: Vec<(String, TableId)>,
    unique_constraint_mapping: Vec<(String, UniqueConstraintId)>,
    field_mapping: Vec<(String, TableColumnId)>,
    enum_mapping: Vec<(String, EnumId)>,
    relation_mapping: Vec<(String, RelationId)>,
}

impl OutputContext {
    pub fn new(namespace: Option<&str>) -> Self {
        let query_type_name = namespace
            .map(|namespace| format!("{namespace}_Query").to_pascal_case())
            .unwrap_or_else(|| String::from("Query"));

        let mutation_type_name = namespace
            .map(|namespace| format!("{namespace}_Mutation").to_pascal_case())
            .unwrap_or_else(|| String::from("Mutation"));

        let mut registry = Registry::default();

        registry.create_type(
            |_| ObjectType::new(&query_type_name, []).into(),
            &query_type_name,
            &query_type_name,
        );

        registry.create_type(
            |_| ObjectType::new(&mutation_type_name, []).into(),
            &mutation_type_name,
            &mutation_type_name,
        );

        registry.mutation_type = Some(String::from("Mutation"));

        if let Some(namespace) = namespace {
            let mut query_type = ObjectType::new("Query", []);

            query_type.fields.insert(
                namespace.to_camel_case(),
                MetaField::new(namespace.to_camel_case(), query_type_name.clone()),
            );

            registry.create_type(|_| query_type.into(), "Query", "Query");

            let mut mutation_type = ObjectType::new("Mutation", []);

            mutation_type.fields.insert(
                namespace.to_camel_case(),
                MetaField::new(namespace.to_camel_case(), mutation_type_name.clone()),
            );

            registry.create_type(|_| mutation_type.into(), "Mutation", "Mutation");
        }

        Self {
            query_type_name,
            mutation_type_name,
            registry,
            type_mapping: Vec::new(),
            unique_constraint_mapping: Vec::new(),
            field_mapping: Vec::new(),
            enum_mapping: Vec::new(),
            relation_mapping: Vec::new(),
        }
    }

    pub fn with_input_type<F>(&mut self, name: &str, table_id: TableId, f: F)
    where
        F: FnOnce(&mut InputTypeBuilder),
    {
        let mut builder = InputTypeBuilder::new(name, table_id);

        f(&mut builder);

        self.type_mapping.extend(builder.type_mapping);
        self.unique_constraint_mapping.extend(builder.unique_constraint_mapping);
        self.field_mapping.extend(builder.field_mapping);
        self.relation_mapping.extend(builder.relation_mapping);

        self.registry
            .create_type(|_| builder.input_object_type.into(), name, name);

        for object in builder.nested {
            let name = object.name.clone();
            self.registry.create_type(|_| object.into(), &name, &name);
        }
    }

    pub fn with_object_type<F>(&mut self, name: &str, table_id: TableId, f: F)
    where
        F: FnOnce(&mut ObjectTypeBuilder),
    {
        let mut builder = ObjectTypeBuilder::new(name, table_id);

        f(&mut builder);

        self.type_mapping.extend(builder.type_mapping);
        self.field_mapping.extend(builder.field_mapping);
        self.relation_mapping.extend(builder.relation_mapping);

        self.create_object_type(builder.object_type);
    }

    pub fn create_object_type(&mut self, object: ObjectType) {
        let name = object.name.clone();
        self.registry.create_type(|_| object.into(), &name, &name);
    }

    pub fn create_enum_type(&mut self, r#enum: EnumType) {
        let name = r#enum.name.clone();
        self.registry.create_type(|_| r#enum.into(), &name, &name);
    }

    pub(crate) fn create_input_type(&mut self, input_object: InputObjectType) {
        let name = input_object.name.clone();
        self.registry.create_type(|_| input_object.into(), &name, &name);
    }

    pub fn with_enum<F>(&mut self, name: &str, enum_id: EnumId, f: F)
    where
        F: FnOnce(&mut EnumBuilder),
    {
        let mut builder = EnumBuilder::new(name);

        f(&mut builder);

        let name = builder.enum_type.name.clone();
        self.enum_mapping.push((name.clone(), enum_id));
        self.registry.create_type(|_| builder.enum_type.into(), &name, &name);
    }

    pub fn push_query(&mut self, query: MetaField) {
        let fields = self
            .registry
            .types
            .get_mut(&self.query_type_name)
            .and_then(|r#type| r#type.fields_mut())
            .expect("Query type not registered.");

        fields.insert(query.name.to_string(), query);
    }

    pub fn push_mutation(&mut self, mutation: MetaField) {
        let fields = self
            .registry
            .types
            .get_mut(&self.mutation_type_name)
            .and_then(|r#type| r#type.fields_mut())
            .expect("Mutation type not registered.");

        fields.insert(mutation.name.to_string(), mutation);
    }

    /// Merge the database definition to a registry.
    pub fn finalize(mut self, mut database_definition: DatabaseDefinition, name: &str) -> Registry {
        for (type_name, table_id) in self.type_mapping {
            database_definition.push_client_type_mapping(&type_name, table_id);
        }

        for (field_name, column_id) in self.field_mapping {
            let table_id = database_definition.walk(column_id).table().id();
            database_definition.push_client_field_mapping(&field_name, table_id, column_id);
        }

        for (enum_name, enum_id) in self.enum_mapping {
            database_definition.push_client_enum_mapping(&enum_name, enum_id);
        }

        for (field_name, relation_id) in self.relation_mapping {
            let table_id = database_definition.walk(relation_id).referencing_table().id();
            database_definition.push_client_relation_mapping(&field_name, table_id, relation_id);
        }

        for (field_name, constraint_id) in self.unique_constraint_mapping {
            let table_id = database_definition.walk(constraint_id).table().id();
            database_definition.push_client_field_unique_constraint_mapping(&field_name, table_id, constraint_id);
        }

        self.registry
            .postgres_databases
            .insert(name.to_string(), database_definition);

        self.registry
    }
}
