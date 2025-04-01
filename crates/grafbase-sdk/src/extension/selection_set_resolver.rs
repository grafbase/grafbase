use crate::{
    component::AnyExtension,
    types::{ArgumentValues, Configuration, Data, Error, Field, Schema, SubgraphHeaders},
};

pub trait SelectionSetResolverExtension: Sized + 'static {
    fn new(schema: Schema<'_>, config: Configuration) -> Result<Self, Error>;
    fn prepare(&mut self, field: Field<'_>) -> Result<Vec<u8>, Error>;
    fn resolve(
        &mut self,
        headers: SubgraphHeaders,
        prepared: &[u8],
        arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error>;
}

#[doc(hidden)]
pub fn register<T: SelectionSetResolverExtension>() {
    pub(super) struct Proxy<T: SelectionSetResolverExtension>(T);

    impl<T: SelectionSetResolverExtension> AnyExtension for Proxy<T> {
        fn selection_set_resolver_prepare(&mut self, field: Field<'_>) -> Result<Vec<u8>, Error> {
            self.0.prepare(field)
        }
        fn selection_set_resolver_resolve(
            &mut self,
            headers: SubgraphHeaders,
            prepared: Vec<u8>,
            arguments: ArgumentValues<'_>,
        ) -> Result<Data, Error> {
            self.0.resolve(headers, &prepared, arguments)
        }
    }

    crate::component::register_extension(Box::new(|schema, _, config| {
        <T as SelectionSetResolverExtension>::new(schema, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
