use crate::{
    component::AnyExtension,
    types::{ArgumentValues, Configuration, Data, Error, Field, IndexedSchema, SubgraphHeaders, SubgraphSchema},
};

pub trait SelectionSetResolverExtension: Sized + 'static {
    fn new(subgraph_schemas: Vec<SubgraphSchema<'_>>, config: Configuration) -> Result<Self, Error>;
    fn prepare(&mut self, subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error>;
    fn resolve(
        &mut self,
        headers: SubgraphHeaders,
        subgraph_name: &str,
        prepared: &[u8],
        arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error>;
}

#[doc(hidden)]
pub fn register<T: SelectionSetResolverExtension>() {
    pub(super) struct Proxy<T: SelectionSetResolverExtension>(T);

    impl<T: SelectionSetResolverExtension> AnyExtension for Proxy<T> {
        fn selection_set_resolver_prepare(&mut self, subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error> {
            self.0.prepare(subgraph_name, field)
        }
        fn selection_set_resolver_resolve(
            &mut self,
            headers: SubgraphHeaders,
            subgraph_name: &str,
            prepared: Vec<u8>,
            arguments: ArgumentValues<'_>,
        ) -> Result<Data, Error> {
            self.0.resolve(headers, subgraph_name, &prepared, arguments)
        }
    }

    crate::component::register_extension(Box::new(|subgraph_schemas, config| {
        let schemas = subgraph_schemas
            .into_iter()
            .map(IndexedSchema::from)
            .collect::<Vec<_>>();
        <T as SelectionSetResolverExtension>::new(schemas.iter().map(SubgraphSchema).collect(), config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
