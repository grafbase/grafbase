use std::borrow::Cow;

use inflector::Inflector;
use postgres_types::database_definition::DatabaseDefinition;

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

    pub(crate) fn directive_name(&self) -> &str {
        self.directive_name
    }

    pub(crate) fn type_name<'b>(&self, name: &'b str) -> Cow<'b, str> {
        match self.namespace {
            Some(namespace) => Cow::Owned(format!("{namespace}_{name}").to_pascal_case()),
            None => Cow::Borrowed(name),
        }
    }

    // an output type which has limited number of fields, e.g. only scalar fields.
    pub(crate) fn reduced_type_name(&self, name: &str) -> String {
        match self.namespace {
            Some(namespace) => format!("{namespace}_{name}_reduced").to_pascal_case(),
            None => format!("{name}_reduced").to_pascal_case(),
        }
    }

    pub(crate) fn create_input_name(&self, name: &str) -> String {
        match self.namespace {
            Some(namespace) => format!("{namespace}_{name}_input").to_pascal_case(),
            None => format!("{name}_input").to_pascal_case(),
        }
    }

    pub(crate) fn filter_type_name(&self, scalar: &str) -> String {
        let base = format!("{scalar}_search_filter_input");

        match self.namespace {
            Some(namespace) => format!("{namespace}_{base}").to_pascal_case(),
            None => base.to_pascal_case(),
        }
    }

    pub(crate) fn connection_type_name(&self, name: &str) -> String {
        let base_name = format!("{name}Connection");

        match self.namespace {
            Some(namespace) => format!("{namespace}_{base_name}").to_pascal_case(),
            None => base_name,
        }
    }

    pub(crate) fn collection_query_name(&self, type_name: &str) -> String {
        format!("{type_name}_Collection").to_camel_case()
    }

    pub(crate) fn edge_type_name(&self, name: &str) -> String {
        let base_name = format!("{name}Edge");

        match self.namespace {
            Some(namespace) => format!("{namespace}_{base_name}").to_pascal_case(),
            None => base_name,
        }
    }

    pub(crate) fn orderby_input_type_name(&self, name: &str) -> String {
        let base_name = format!("{name}OrderByInput");

        match self.namespace {
            Some(namespace) => format!("{namespace}_{base_name}").to_pascal_case(),
            None => base_name,
        }
    }

    pub(crate) fn database_definition(&self) -> &DatabaseDefinition {
        &self.database_definition
    }

    pub(crate) fn finalize(self) -> DatabaseDefinition {
        self.database_definition
    }
}
