use postgresql_types::database_definition::DatabaseDefinition;
use sdl_parser::Registry;

pub(super) fn generate(database_definition: DatabaseDefinition, name: &str, _namespaced: bool) -> Registry {
    let mut registry = Registry::default();

    registry
        .postgres_databases
        .insert(name.to_string(), database_definition);

    registry
}
