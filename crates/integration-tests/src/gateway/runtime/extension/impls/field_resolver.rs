use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use futures::{FutureExt as _, stream::BoxStream};
use runtime::{
    extension::Anything,
    extension::{Data, FieldResolverExtension},
};

use crate::gateway::{
    AnyExtension, DispatchRule, DynHookContext, ExtContext, ExtensionsBuilder, ExtensionsDispatcher, TestExtensions,
    TestManifest,
};

impl FieldResolverExtension<ExtContext> for ExtensionsDispatcher {
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
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
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Data, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .resolve_subscription_field(directive, field_definition, subgraph_headers, directive_arguments)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .resolve_subscription_field(directive, field_definition, subgraph_headers, directive_arguments)
                    .await
            }
        }
    }
}

impl FieldResolverExtension<DynHookContext> for TestExtensions {
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
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
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Data, GraphqlError>>, GraphqlError>
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
    async fn resolve_field(
        &self,
        directive: ExtensionDirective<'_>,
        field_definition: FieldDefinition<'_>,
        subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError>;
}
