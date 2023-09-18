mod filter;
mod selection;

pub(super) use filter::SimpleFilterIterator;
pub(super) use selection::{CollectionArgs, SelectionIterator, TableSelection};

use crate::{
    registry::{resolvers::ResolverContext, Registry},
    Context, Error, ServerResult,
};
use postgresql_types::{
    database_definition::{DatabaseDefinition, TableWalker},
    transport::NeonTransport,
};
use serde_json::{Map, Value};

/// The API to access the request parameters, such as filters and selection, and map that together with
/// the database types.
pub struct PostgresContext<'a> {
    context: &'a Context<'a>,
    resolver_context: &'a ResolverContext<'a>,
    database_definition: &'a DatabaseDefinition,
    transport: NeonTransport,
}

impl<'a> PostgresContext<'a> {
    pub fn new(
        context: &'a Context<'a>,
        resolver_context: &'a ResolverContext<'a>,
        directive_name: &str,
    ) -> Result<Self, Error> {
        let database_definition = context
            .get_postgresql_definition(directive_name)
            .expect("directive must exist");

        let transport = NeonTransport::new(database_definition.connection_string())
            .map_err(|error| Error::new(error.to_string()))?;

        Ok(Self {
            context,
            resolver_context,
            database_definition,
            transport,
        })
    }

    /// The main table accessed by this request.
    pub fn table(&self) -> TableWalker<'a> {
        self.resolver_context
            .ty
            .and_then(|meta_type| self.database_definition.find_table_for_client_type(meta_type.name()))
            .expect("could not find table for client type")
    }

    /// The selection of fields in the request.
    pub fn selection(&'a self) -> SelectionIterator<'a> {
        let selection = self
            .context
            .look_ahead()
            .selection_fields()
            .into_iter()
            .flat_map(|selection| selection.selection_set())
            .collect();

        let meta_type = self.resolver_context.ty.unwrap();
        SelectionIterator::new(self, meta_type, selection)
    }

    /// Access to the schema registry.
    pub fn registry(&self) -> &Registry {
        self.context.registry()
    }

    /// A simple `user(by: { id: 1 })` filter, that has exactly one equals operation.
    pub fn by_filter(&self) -> ServerResult<SimpleFilterIterator<'a>> {
        let filter_map: Map<String, Value> = self.context.input_by_name("by")?;
        let input_type = self.context.find_argument_type("by")?;
        let iterator = SimpleFilterIterator::new(self.database_definition, input_type, filter_map);

        Ok(iterator)
    }

    /// The database connection.
    pub fn transport(&self) -> &NeonTransport {
        &self.transport
    }
}
