use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use futures::{FutureExt as _, stream::BoxStream};
use runtime::{
    extension::{Data, FieldResolverExtension},
    hooks::Anything,
};

use crate::federation::{
    AnyExtension, DispatchRule, DynHookContext, ExtContext, ExtensionsBuilder, ExtensionsDispatcher, TestExtensions,
    TestManifest,
};

impl FieldResolverExtension<ExtContext> for ExtensionsDispatcher {
    async fn prepare<'ctx>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .prepare(directive, field_definition, directive_arguments)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .prepare(directive, field_definition, directive_arguments)
                    .await
            }
        }
    }

    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => self
                .wasm
                .resolve_field(
                    directive,
                    field_definition,
                    prepared_data,
                    subgraph_headers,
                    directive_arguments,
                    inputs,
                )
                .boxed(),
            DispatchRule::Test => self
                .test
                .resolve_field(
                    directive,
                    field_definition,
                    prepared_data,
                    subgraph_headers,
                    directive_arguments,
                    inputs,
                )
                .boxed(),
        }
    }

    async fn resolve_subscription_field<'ctx, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .resolve_subscription_field(
                        directive,
                        field_definition,
                        prepared_data,
                        subgraph_headers,
                        directive_arguments,
                    )
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .resolve_subscription_field(
                        directive,
                        field_definition,
                        prepared_data,
                        subgraph_headers,
                        directive_arguments,
                    )
                    .await
            }
        }
    }
}

impl FieldResolverExtension<DynHookContext> for TestExtensions {
    async fn prepare<'ctx>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        let instance = self
            .state
            .lock()
            .await
            .get_field_resolver_ext(directive.extension_id, directive.subgraph());
        instance
            .prepare(
                directive,
                field_definition,
                serde_json::to_value(directive_arguments).unwrap(),
            )
            .await
    }

    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let inputs = inputs
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let directive_arguments = serde_json::to_value(directive_arguments).unwrap();
        async move {
            let instance = self
                .state
                .lock()
                .await
                .get_field_resolver_ext(directive.extension_id, directive.subgraph());
            instance
                .resolve_field(
                    directive,
                    field_definition,
                    prepared_data,
                    subgraph_headers,
                    directive_arguments,
                    inputs,
                )
                .await
        }
    }

    async fn resolve_subscription_field<'ctx, 'f>(
        &'ctx self,
        _directive: ExtensionDirective<'ctx>,
        _field_definition: FieldDefinition<'ctx>,
        _prepared_data: &'ctx [u8],
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        unimplemented!()
    }
}

pub struct FieldResolverExt {
    instance: Arc<dyn FieldResolverTestExtension>,
    name: &'static str,
    sdl: Option<&'static str>,
}

impl FieldResolverExt {
    pub fn new<T: FieldResolverTestExtension>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            name: "field-resolver",
            sdl: None,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_sdl(mut self, sdl: &'static str) -> Self {
        self.sdl = Some(sdl);
        self
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }
}

impl AnyExtension for FieldResolverExt {
    fn register(self, state: &mut ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: self.name.to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::FieldResolver(extension_catalog::FieldResolverType {
                resolver_directives: None,
            }),
            sdl: self.sdl,
        });
        let instance = self.instance;
        state
            .test
            .field_resolver_builders
            .insert(id, Arc::new(move || Arc::clone(&instance)));
    }
}

pub trait FieldResolverTestExtensionBuilder: Send + Sync + 'static {
    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn FieldResolverTestExtension>;
}

impl<F: Fn() -> Arc<dyn FieldResolverTestExtension> + Send + Sync + 'static> FieldResolverTestExtensionBuilder for F {
    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn FieldResolverTestExtension> {
        self()
    }
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait FieldResolverTestExtension: Send + Sync + 'static {
    async fn prepare<'ctx>(
        &self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: serde_json::Value,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn resolve_field(
        &self,
        directive: ExtensionDirective<'_>,
        field_definition: FieldDefinition<'_>,
        prepared_data: &[u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError>;
}
