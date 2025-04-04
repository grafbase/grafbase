use crate::types::{ArgumentValues, Configuration, Data, Error, Field, SubgraphHeaders, SubgraphSchema};

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
