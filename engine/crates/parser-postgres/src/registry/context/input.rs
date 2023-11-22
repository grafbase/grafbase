use inflector::Inflector;
use parser_sdl::MetaNames;
use postgres_connector_types::database_definition::DatabaseDefinition;

pub struct InputContext<'a> {
    namespace: Option<&'a str>,
    database_definition: DatabaseDefinition,
    directive_name: &'a str,
}

impl<'a> InputContext<'a> {
    pub(crate) fn new(database_definition: DatabaseDefinition, name: &'a str, namespaced: bool) -> Self {
        Self {
            namespace: namespaced.then_some(name),
            database_definition,
            directive_name: name,
        }
    }

    pub(crate) fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    pub(crate) fn directive_name(&self) -> &str {
        self.directive_name
    }

    pub(crate) fn type_name(&self, name: &str) -> String {
        let base_name = name.to_string();
        self.namespaced(base_name)
    }

    pub(crate) fn delete_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::delete_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn delete_many_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::delete_many_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn update_input_name(&self, name: &str) -> String {
        let base_name = MetaNames::update_input_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn update_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::update_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn update_many_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::update_many_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    // an output type which has limited number of fields, e.g. only scalar fields.
    pub(crate) fn mutation_return_type_name(&self, name: &str) -> String {
        let base_name = format!("{name}_mutation").to_pascal_case();
        self.namespaced(base_name)
    }

    pub(crate) fn create_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::create_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn create_many_payload_name(&self, name: &str) -> String {
        let base_name = MetaNames::create_many_payload_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn returning_type_name(&self, name: &str) -> String {
        let base_name = format!("{name}_returning").to_pascal_case();
        self.namespaced(base_name)
    }

    pub(crate) fn create_input_name(&self, name: &str) -> String {
        let base_name = MetaNames::create_input_by_str(name, None);
        self.namespaced(base_name)
    }

    pub(crate) fn filter_type_name(&self, scalar: &str) -> String {
        let base_name = format!("{scalar}_search_filter_input").to_pascal_case();
        self.namespaced(base_name)
    }

    pub(crate) fn connection_type_name(&self, name: &str) -> String {
        let base_name = MetaNames::pagination_connection_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn edge_type_name(&self, name: &str) -> String {
        let base_name = MetaNames::pagination_edge_type_by_str(name);
        self.namespaced(base_name)
    }

    pub(crate) fn orderby_input_type_name(&self, name: &str) -> String {
        let base_name = MetaNames::pagination_orderby_input_by_str(name).to_string();
        self.namespaced(base_name)
    }

    pub(crate) fn collection_query_name(&self, type_name: &str) -> String {
        MetaNames::collection_by_str(type_name).to_camel_case()
    }

    pub(crate) fn database_definition(&self) -> &DatabaseDefinition {
        &self.database_definition
    }

    pub(crate) fn finalize(self) -> DatabaseDefinition {
        self.database_definition
    }

    fn namespaced(&self, name: String) -> String {
        match self.namespace {
            Some(namespace) => format!("{namespace}_{name}").to_pascal_case(),
            None => name,
        }
    }
}
