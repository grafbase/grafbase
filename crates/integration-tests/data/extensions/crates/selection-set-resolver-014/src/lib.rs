use grafbase_sdk::{
    SelectionSetResolverExtension,
    types::{ArgumentValues, Configuration, Data, Error, Field, SubgraphHeaders, SubgraphSchema},
};

#[derive(SelectionSetResolverExtension)]
struct Resolver {
    value: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct Config {
    value: serde_json::Value,
}

impl SelectionSetResolverExtension for Resolver {
    fn new(_subgraph_schemas: Vec<SubgraphSchema<'_>>, config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize()?;
        Ok(Self { value: config.value })
    }

    fn prepare(&mut self, _subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error> {
        Ok(field.into_bytes())
    }

    fn resolve(
        &mut self,
        _headers: SubgraphHeaders,
        _subgraph_name: &str,
        prepared: &[u8],
        _arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error> {
        Field::with_bytes(prepared, |_field| Ok(Data::new(&self.value)?))?
    }
}
