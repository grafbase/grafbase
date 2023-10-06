mod filter;
pub mod selection;

pub use selection::CollectionArgs;

pub(super) use filter::FilterIterator;
pub(super) use selection::{SelectionIterator, TableSelection};

use crate::{
    registry::{resolvers::ResolverContext, type_kinds::SelectionSetTarget, Registry},
    Context, ContextExt, ContextField, Error, SelectionField, ServerResult,
};
use postgres_types::{
    database_definition::{DatabaseDefinition, TableWalker},
    transport::NeonTransport,
};
use serde_json::{Map, Value};

use self::filter::{ByFilterIterator, ComplexFilterIterator};

/// The API to access the request parameters, such as filters and selection, and map that together with
/// the database types.
pub struct PostgresContext<'a> {
    context: &'a ContextField<'a>,
    resolver_context: &'a ResolverContext<'a>,
    database_definition: &'a DatabaseDefinition,
    transport: NeonTransport,
}

impl<'a> PostgresContext<'a> {
    pub fn new(
        context: &'a ContextField<'a>,
        resolver_context: &'a ResolverContext<'a>,
        directive_name: &str,
    ) -> Result<Self, Error> {
        let database_definition = context
            .get_postgres_definition(directive_name)
            .expect("directive must exist");

        let ray_id = context.data::<runtime::Context>()?.ray_id();
        let transport = NeonTransport::new(ray_id, database_definition.connection_string())
            .map_err(|error| Error::new(error.to_string()))?;

        Ok(Self {
            context,
            resolver_context,
            database_definition,
            transport,
        })
    }

    pub fn database_definition(&self) -> &DatabaseDefinition {
        self.database_definition
    }

    /// The main table accessed by this request.
    pub fn table(&self) -> TableWalker<'a> {
        self.database_definition
            .find_table_for_client_type(self.resolver_context.ty.name())
            .expect("could not find table for client type")
    }

    /// The first field of the query, e.g. the query.
    pub fn root_field(&self) -> SelectionField<'a> {
        self.context
            .look_ahead()
            .iter_selection_fields()
            .next()
            .expect("we always have at least one field in the query")
    }

    /// The selection of fields in the request.
    pub fn selection(&'a self) -> SelectionIterator<'a> {
        let selection = self
            .context
            .look_ahead()
            .iter_selection_fields()
            .flat_map(|selection| selection.selection_set())
            .collect();

        SelectionIterator::new(self, self.resolver_context.ty, &self.root_field(), selection)
    }

    pub fn collection_selection(&'a self) -> SelectionIterator<'a> {
        let selection = self
            .context
            .look_ahead()
            .field("edges")
            .field("node")
            .iter_selection_fields()
            .flat_map(|selection| selection.selection_set())
            .collect();

        let target: SelectionSetTarget = self.resolver_context.ty.try_into().unwrap();

        let output_type = target
            .field("edges")
            .and_then(|field| self.registry().lookup(&field.ty).ok())
            .as_ref()
            .and_then(|output| output.field("node"))
            .and_then(|field| self.registry().lookup(&field.ty).ok())
            .expect("couldn't find a meta type for a collection selection");

        SelectionIterator::new(self, output_type, &self.root_field(), selection)
    }

    /// Access to the schema registry.
    pub fn registry(&self) -> &Registry {
        self.context.registry()
    }

    /// A simple `user(by: { id: 1 })` filter, that has exactly one equals operation.
    pub fn by_filter(&self) -> ServerResult<FilterIterator<'a>> {
        let filter_map: Map<String, Value> = self.context.input_by_name("by")?;
        let input_type = self.context.find_argument_type("by")?;
        let iterator = ByFilterIterator::new(self.database_definition, input_type, filter_map);

        Ok(FilterIterator::By(iterator))
    }

    pub fn filter(&'a self) -> ServerResult<FilterIterator<'a>> {
        let filter_map: Map<String, Value> = self.context.input_by_name("filter")?;
        let input_type = self.context.find_argument_type("filter")?;
        let iterator = ComplexFilterIterator::new(self, input_type, filter_map);

        Ok(FilterIterator::Complex(iterator))
    }

    /// The database connection.
    pub fn transport(&self) -> &NeonTransport {
        &self.transport
    }
}
