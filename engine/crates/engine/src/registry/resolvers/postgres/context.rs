mod create_input;
mod filter;
pub mod selection;
mod update_input;

pub(super) use create_input::{CreateInputItem, CreateInputIterator};
pub(super) use filter::FilterIterator;
use postgres_types::{
    database_definition::{DatabaseDefinition, TableWalker},
    transport::Transport,
};
pub use selection::CollectionArgs;
pub(super) use selection::{SelectionIterator, TableSelection};
use serde_json::{Map, Value};
pub(super) use update_input::{UpdateInputItem, UpdateInputIterator};

use self::filter::{ByFilterIterator, ComplexFilterIterator};
use crate::{
    registry::{resolvers::ResolverContext, type_kinds::SelectionSetTarget, Registry},
    Context, ContextExt, ContextField, Error, SelectionField, ServerResult,
};

/// The API to access the request parameters, such as filters and selection, and map that together with
/// the database types.
pub struct PostgresContext<'a> {
    context: &'a ContextField<'a>,
    resolver_context: &'a ResolverContext<'a>,
    database_definition: &'a DatabaseDefinition,
    transport: Box<dyn Transport>,
}

impl<'a> PostgresContext<'a> {
    pub async fn new(
        context: &'a ContextField<'a>,
        resolver_context: &'a ResolverContext<'a>,
        database_definition: &'a DatabaseDefinition,
        transport: Box<dyn Transport>,
    ) -> Result<PostgresContext<'a>, Error> {
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

    pub fn mutation_is_returning(&self) -> bool {
        self.context.look_ahead().field("returning").exists()
    }

    pub fn returning_selection(&'a self) -> Option<SelectionIterator<'a>> {
        if !self.mutation_is_returning() {
            return None;
        }

        let selection = self
            .context
            .look_ahead()
            .field("returning")
            .iter_selection_fields()
            .flat_map(|selection| selection.selection_set())
            .collect();

        let target: SelectionSetTarget = self.resolver_context.ty.try_into().unwrap();

        let output_type = target
            .field("returning")
            .and_then(|field| self.registry().lookup(&field.ty).ok())
            .expect("couldn't find a meta type for a returning selection");

        Some(SelectionIterator::new(self, output_type, &self.root_field(), selection))
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

    /// A complex `user(filter: { id: { eq: 1 } })` filter.
    pub fn filter(&'a self) -> ServerResult<FilterIterator<'a>> {
        let filter_map: Map<String, Value> = self.context.input_by_name("filter")?;
        let input_type = self.context.find_argument_type("filter")?;
        let iterator = ComplexFilterIterator::new(self, input_type, filter_map);

        Ok(FilterIterator::Complex(iterator))
    }

    /// An iterator for create input value definition.
    pub fn create_input(&'a self) -> ServerResult<CreateInputIterator<'a>> {
        let input_map: Map<String, Value> = self.context.input_by_name("input")?;
        let input_type = self.context.find_argument_type("input")?;
        let iterator = CreateInputIterator::new(self.database_definition(), input_type, input_map);

        Ok(iterator)
    }

    /// A collection of iterators for multiple create input value definitions.
    pub fn create_many_input(&'a self) -> ServerResult<Vec<CreateInputIterator<'a>>> {
        let input_map: Vec<Map<String, Value>> = self.context.input_by_name("input")?;
        let input_type = self.context.find_argument_type("input")?;

        let iterators = input_map
            .into_iter()
            .map(|input_map| CreateInputIterator::new(self.database_definition(), input_type, input_map))
            .collect();

        Ok(iterators)
    }

    /// An iterator for update value definition.
    pub fn update_input(&'a self) -> ServerResult<UpdateInputIterator<'a>> {
        let input_map: Map<String, Value> = self.context.input_by_name("input")?;
        let input_type = self.context.find_argument_type("input")?;
        let iterator = UpdateInputIterator::new(self.database_definition(), input_type, input_map);

        Ok(iterator)
    }

    /// The database connection.
    pub fn transport(&self) -> &dyn Transport {
        self.transport.as_ref()
    }

    pub fn runtime_ctx(&self) -> Result<&runtime::Context, crate::Error> {
        self.context.data::<runtime::Context>()
    }

    pub fn ray_id(&self) -> Result<&str, crate::Error> {
        Ok(self.runtime_ctx()?.ray_id())
    }

    pub fn fetch_log_endpoint_url(&self) -> Result<Option<&str>, crate::Error> {
        Ok(self.runtime_ctx()?.log.fetch_log_endpoint_url.as_deref())
    }
}
