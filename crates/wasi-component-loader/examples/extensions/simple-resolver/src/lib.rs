use grafbase_sdk::{
    Error, Headers, ResolverExtension, Subscription,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
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
        _: Headers,
        _: &str,
        directive: FieldDefinitionDirective,
        _: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let args: FieldArgs = directive.arguments().unwrap();

        let mut output = FieldOutput::new();

        output.push_value(ResponseOutput {
            id: self.schema_args.id,
            name: args.name,
        });

        Ok(output)
    }

    fn resolve_subscription(
        &mut self,
        _: Headers,
        _: &str,
        _: FieldDefinitionDirective,
    ) -> Result<Box<dyn Subscription>, Error> {
        todo!()
    }
}
