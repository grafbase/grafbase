use crate::{
    component::AnyExtension,
    types::{Configuration, Contract, ContractDirective, Error, GraphqlSubgraph},
};

pub trait ContractsExtension: Sized + 'static {
    fn new(config: Configuration) -> Result<Self, Error>;
    fn construct(
        &mut self,
        key: String,
        directives: Vec<ContractDirective<'_>>,
        subgraphs: Vec<GraphqlSubgraph>,
    ) -> Result<Contract, String>;
}

#[doc(hidden)]
pub fn register<T: ContractsExtension>() {
    pub(super) struct Proxy<T: ContractsExtension>(T);

    impl<T: ContractsExtension> AnyExtension for Proxy<T> {
        fn construct(
            &mut self,
            key: String,
            directives: Vec<ContractDirective<'_>>,
            subgraphs: Vec<GraphqlSubgraph>,
        ) -> Result<Contract, String> {
            ContractsExtension::construct(&mut self.0, key, directives, subgraphs)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as ContractsExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
