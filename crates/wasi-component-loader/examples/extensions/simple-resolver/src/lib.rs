use grafbase_sdk::{
    ResolverExtension, Subscription,
    types::{
        Configuration, Error, FieldDefinitionDirective, FieldInputs, FieldOutputs, SchemaDirective, SubgraphHeaders,
    },
};

#[derive(ResolverExtension)]
struct SimpleResolver {
    schema_args: SchemaArgs,
}

#[derive(serde::Deserialize)]
struct SchemaArgs {
    id: usize,
}

#[derive(serde::Deserialize)]
struct FieldArgs<'a> {
    name: &'a str,
}

#[derive(serde::Serialize)]
struct ResponseOutput<'a> {
    id: usize,
    name: &'a str,
}

impl ResolverExtension for SimpleResolver {
    fn new(schema_directives: Vec<SchemaDirective>, _: Configuration) -> Result<Self, Error> {
        let schema_args = schema_directives
            .into_iter()
            .filter(|d| d.name() == "schemaArgs")
            .map(|d| d.arguments().unwrap())
            .next()
            .unwrap();

        Ok(Self { schema_args })
    }

    fn resolve_field(
        &mut self,
        _: SubgraphHeaders,
        _: &str,
        directive: FieldDefinitionDirective,
        inputs: FieldInputs,
    ) -> Result<FieldOutputs, Error> {
        let args: FieldArgs = directive.arguments().unwrap();

        Ok(FieldOutputs::new(
            inputs,
            ResponseOutput {
                id: self.schema_args.id,
                name: args.name,
            },
        )?)
    }

    fn resolve_subscription(
        &mut self,
        _: SubgraphHeaders,
        _: &str,
        _: FieldDefinitionDirective,
    ) -> Result<Box<dyn Subscription>, Error> {
        todo!()
    }
}
