use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext,
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
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

impl Extension for SimpleResolver {
    fn new(schema_directives: Vec<Directive>, _: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let schema_args = schema_directives
            .into_iter()
            .filter(|d| d.name() == "schemaArgs")
            .map(|d| d.arguments().unwrap())
            .next()
            .unwrap();

        Ok(Self { schema_args })
    }
}

impl Resolver for SimpleResolver {
    fn resolve_field(
        &mut self,
        _: SharedContext,
        directive: Directive,
        _: FieldDefinition,
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

    fn resolve_subscription(&mut self, _: SharedContext, _: Directive, _: FieldDefinition) -> Result<(), Error> {
        todo!()
    }

    fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, Error> {
        todo!()
    }
}
