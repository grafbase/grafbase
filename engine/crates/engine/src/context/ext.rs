use std::any::{Any, TypeId};

use engine_value::{ConstValue as Value, Value as InputValue}; // WHY?
use graph_entities::QueryResponse;
use postgres_connector_types::database_definition::DatabaseDefinition;

use crate::{
    deferred::DeferredWorkloadSender, parser::types::FragmentDefinition, query_path::QueryPath, schema::SchemaEnv,
    Error, LegacyInputType, Name, Pos, Positioned, QueryEnv, Result, ServerError, ServerResult,
};

pub trait Context<'a> {
    fn path(&self) -> &QueryPath;
    fn query_env(&self) -> &'a QueryEnv;
    fn schema_env(&self) -> &'a SchemaEnv;

    fn registry(&self) -> &'a registry_v2::Registry {
        &self.schema_env().registry
    }
}

#[derive(Clone)]
pub struct TraceId(pub String);

/// Extension trait that defines shared behaviour for ContextSelectionSet & ContextField
pub trait ContextExt<'a>: Context<'a> {
    fn response<'b>(&'b self) -> async_lock::futures::Lock<'b, QueryResponse>
    where
        'a: 'b,
    {
        self.query_env().response.lock()
    }

    fn deferred_workloads(&self) -> Option<&'a DeferredWorkloadSender> {
        self.query_env().deferred_workloads.as_ref()
    }

    /// Find a fragment definition by name.
    fn get_fragment(&self, name: &str) -> Option<&'a FragmentDefinition> {
        self.query_env().fragments.get(name).map(|fragment| &fragment.node)
    }

    /// Find a type definition by name.
    fn get_type(&'a self, name: &str) -> Option<registry_v2::MetaType<'a>> {
        self.schema_env().registry.lookup_type(name)
    }

    /// Find a mongodb configuration with name.
    fn get_mongodb_config(&self, name: &str) -> Option<&'a registry_v2::mongodb::MongoDBConfiguration> {
        self.schema_env().registry.mongodb_configurations.get(name)
    }

    fn get_postgres_definition(&self, name: &str) -> Option<&'a DatabaseDefinition> {
        self.schema_env().registry.postgres_databases.get(name)
    }

    fn set_error_path(&self, error: ServerError) -> ServerError {
        if !error.path.is_empty() {
            // If the error already has a path we don't want to overwrite it.
            return error;
        }

        ServerError {
            path: self.path().iter().cloned().collect(),
            ..error
        }
    }

    /// Report a resolver error.
    ///
    /// When implementing `OutputType`, if an error occurs, call this function to report this error and return `Value::Null`.
    fn add_error(&self, error: ServerError) {
        self.query_env().errors.lock().unwrap().push(error);
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `Error` if the specified type data does not exist.
    fn data<D: Any + Send + Sync + 'a>(&self) -> Result<&'a D> {
        self.data_opt::<D>()
            .ok_or_else(|| Error::new(format!("Data `{}` does not exist.", std::any::type_name::<D>())))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    fn data_opt<D: Any + Send + Sync + 'a>(&self) -> Option<&'a D> {
        self.query_env()
            .ctx_data
            .0
            .get(&TypeId::of::<D>())
            .or_else(|| self.query_env().session_data.0.get(&TypeId::of::<D>()))
            .or_else(|| self.schema_env().data.0.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }

    fn var_value(&self, name: &str, pos: Pos) -> ServerResult<Value> {
        self.query_env()
            .operation
            .node
            .variable_definitions
            .iter()
            .find(|def| def.node.name.node == name)
            .and_then(|def| {
                self.query_env()
                    .variables
                    .get(&def.node.name.node)
                    .or_else(|| def.node.default_value())
            })
            .cloned()
            .ok_or_else(|| ServerError::new(format!("Variable {name} is not defined."), Some(pos)))
    }

    fn resolve_input_value(&self, value: Positioned<InputValue>) -> ServerResult<Value> {
        let pos = value.pos;
        value.node.into_const_with(|name| self.var_value(&name, pos))
    }

    #[doc(hidden)]
    fn get_param_value<Q: LegacyInputType>(
        &self,
        arguments: &[(Positioned<Name>, Positioned<InputValue>)],
        name: &str,
        default: Option<fn() -> Q>,
    ) -> ServerResult<(Pos, Q)> {
        let value = arguments
            .iter()
            .find(|(n, _)| n.node.as_str() == name)
            .map(|(_, value)| value)
            .cloned();

        if value.is_none() {
            if let Some(default) = default {
                return Ok((Pos::default(), default()));
            }
        }
        let (pos, value) = match value {
            Some(value) => (value.pos, Some(self.resolve_input_value(value)?)),
            None => (Pos::default(), None),
        };

        LegacyInputType::parse(value)
            .map(|value| (pos, value))
            .map_err(|e| e.into_server_error(pos))
    }
}

impl<'a, T> ContextExt<'a> for T where T: Context<'a> + ?Sized {}
