use std::{any::Any, ops::Deref, sync::Arc};

use dynamodb::CurrentDateTime;
use futures_util::stream::{self, Stream, StreamExt};
use graph_entities::CompactValue;
use indexmap::map::IndexMap;

use crate::{
    context::{Data, QueryEnvInner},
    deferred,
    extensions::{ExtensionFactory, Extensions},
    model::__DirectiveLocation,
    parser::{
        parse_query,
        types::{Directive, DocumentOperations, OperationType, Selection, SelectionSet},
        Positioned,
    },
    registry::{MetaDirective, MetaInputValue, Registry},
    resolver_utils::{self, resolve_root_container, resolve_root_container_serial},
    response::{IncrementalPayload, StreamingPayload},
    subscription::collect_subscription_streams,
    types::QueryRoot,
    validation::{check_rules, ValidationMode},
    BatchRequest, BatchResponse, CacheControl, ContextExt, ContextSelectionSet, LegacyInputType, LegacyOutputType,
    ObjectType, QueryEnv, QueryEnvBuilder, QueryPath, Request, Response, ServerError, SubscriptionType, Variables, ID,
};

/// Schema builder
pub struct SchemaBuilder {
    validation_mode: ValidationMode,
    registry: Registry,
    data: Data,
    complexity: Option<usize>,
    depth: Option<usize>,
    extensions: Vec<Box<dyn ExtensionFactory>>,
}

impl SchemaBuilder {
    /// Manually register a input type in the schema.
    ///
    /// You can use this function to register schema types that are not directly referenced.
    #[must_use]
    pub fn register_input_type<T: LegacyInputType>(mut self) -> Self {
        T::create_type_info(&mut self.registry);
        self
    }

    /// Manually register a output type in the schema.
    ///
    /// You can use this function to register schema types that are not directly referenced.
    #[must_use]
    pub fn register_output_type<T: LegacyOutputType>(mut self) -> Self {
        T::create_type_info(&mut self.registry);
        self
    }

    /// Disable introspection queries.
    #[must_use]
    pub fn disable_introspection(mut self) -> Self {
        self.registry.disable_introspection = true;
        self
    }

    /// Set the maximum complexity a query can have. By default, there is no limit.
    #[must_use]
    pub fn limit_complexity(mut self, complexity: usize) -> Self {
        self.complexity = Some(complexity);
        self
    }

    /// Set the maximum depth a query can have. By default, there is no limit.
    #[must_use]
    pub fn limit_depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Add an extension to the schema.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use engine::*;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn value(&self) -> i32 {
    ///         100
    ///     }
    /// }
    ///
    /// let schema = Schema::build(Query, EmptyMutation,EmptySubscription)
    ///     .extension(extensions::Logger)
    ///     .finish();
    /// ```
    #[must_use]
    pub fn extension(mut self, extension: impl ExtensionFactory) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Add a global data that can be accessed in the `Schema`. You access it with `Context::data`.
    #[must_use]
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Set the validation mode, default is `ValidationMode::Strict`.
    #[must_use]
    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Enable federation, which is automatically enabled if the Query has least one entity definition.
    #[must_use]
    pub fn enable_federation(mut self) -> Self {
        self.registry.enable_federation = true;
        self
    }

    /// Make the Federation SDL include subscriptions.
    ///
    /// Note: Not included by default, in order to be compatible with Apollo Server.
    #[must_use]
    pub fn enable_subscription_in_federation(mut self) -> Self {
        self.registry.federation_subscription = true;
        self
    }

    /// Override the name of the specified input type.
    #[must_use]
    pub fn override_input_type_description<T: LegacyInputType>(mut self, desc: &'static str) -> Self {
        self.registry.set_description(&T::type_name(), desc);
        self
    }

    /// Override the name of the specified output type.
    #[must_use]
    pub fn override_output_type_description<T: LegacyOutputType>(mut self, desc: &'static str) -> Self {
        self.registry.set_description(&T::type_name(), desc);
        self
    }

    /// Build schema.
    pub fn finish(mut self) -> Schema {
        // federation
        if self.registry.enable_federation && self.registry.has_entities() {
            self.registry.create_federation_types();
        }

        Schema(Arc::new(SchemaInner {
            validation_mode: self.validation_mode,
            complexity: self.complexity,
            depth: self.depth,
            extensions: self.extensions,
            env: SchemaEnv(Arc::new(SchemaEnvInner {
                registry: self.registry,
                data: self.data,
            })),
        }))
    }
}

#[doc(hidden)]
pub struct SchemaEnvInner {
    pub registry: Registry,
    pub data: Data,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct SchemaEnv(Arc<SchemaEnvInner>);

impl Deref for SchemaEnv {
    type Target = SchemaEnvInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
pub struct SchemaInner {
    pub(crate) validation_mode: ValidationMode,
    pub(crate) complexity: Option<usize>,
    pub(crate) depth: Option<usize>,
    pub(crate) extensions: Vec<Box<dyn ExtensionFactory>>,
    pub(crate) env: SchemaEnv,
}

/// GraphQL schema.
///
/// Cloning a schema is cheap, so it can be easily shared.
pub struct Schema(Arc<SchemaInner>);

impl Clone for Schema {
    fn clone(&self) -> Self {
        Schema(self.0.clone())
    }
}

impl Default for Schema {
    fn default() -> Self {
        Schema::new(Self::create_registry())
    }
}

impl Deref for Schema {
    type Target = SchemaInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Schema {
    /// Create a schema builder
    ///
    /// The root object for the query and Mutation needs to be specified.
    /// If there is no mutation, you can use `EmptyMutation`.
    /// If there is no subscription, you can use `EmptySubscription`.
    pub fn build(registry: Registry) -> SchemaBuilder {
        SchemaBuilder {
            validation_mode: ValidationMode::Strict,
            registry,
            data: Default::default(),
            complexity: None,
            depth: None,
            extensions: Default::default(),
        }
    }

    pub fn create_registry_static<Query, Mutation, Subscription>() -> Registry
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        let mut registry = Registry {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: Query::type_name().to_string(),
            mutation_type: if Mutation::is_empty() {
                None
            } else {
                Some(Mutation::type_name().to_string())
            },
            subscription_type: if Subscription::is_empty() {
                None
            } else {
                Some(Subscription::type_name().to_string())
            },
            disable_introspection: false,
            enable_federation: false,
            federation_subscription: false,
            ..Default::default()
        };

        Schema::add_builtins_to_registry(&mut registry);

        QueryRoot::<Query>::create_type_info(&mut registry);
        if !Mutation::is_empty() {
            Mutation::create_type_info(&mut registry);
        }
        if !Subscription::is_empty() {
            Subscription::create_type_info(&mut registry);
        }

        registry.remove_unused_types();
        registry
    }

    pub fn create_registry() -> Registry {
        let mut registry = Default::default();

        Schema::add_builtins_to_registry(&mut registry);

        registry.remove_unused_types();
        registry
    }

    fn add_builtins_to_registry(registry: &mut Registry) {
        registry.add_directive(MetaDirective {
            name: "include".to_string(),
            description: Some(
                "Directs the executor to include this field or fragment only when the `if` argument is true."
                    .to_string(),
            ),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "if".to_string(),
                    MetaInputValue::new("if".to_string(), "Boolean!").with_description("Included when true."),
                );
                args
            },
            is_repeatable: false,
            visible: None,
        });

        registry.add_directive(MetaDirective {
            name: "skip".to_string(),
            description: Some(
                "Directs the executor to skip this field or fragment when the `if` argument is true.".to_string(),
            ),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "if".to_string(),
                    MetaInputValue::new("if", "Boolean!").with_description("Skipped when true."),
                );
                args
            },
            is_repeatable: false,
            visible: None,
        });

        registry.add_directive(MetaDirective {
            name: "oneOf".to_string(),
            description: Some("Indicates that an input object is a oneOf input object".to_string()),
            locations: vec![__DirectiveLocation::INPUT_OBJECT],
            args: IndexMap::new(),
            is_repeatable: false,
            visible: Some(|_| true),
        });

        registry.add_directive(MetaDirective {
            name: "defer".to_string(),
            description: Some("De-prioritizes a fragment, causing the fragment to be omitted in the initial response and delivered as a subsequent response afterward.".to_string()),
            locations: vec![
                __DirectiveLocation::INLINE_FRAGMENT,
                __DirectiveLocation::FRAGMENT_SPREAD,
            ],
            args: [
                MetaInputValue::new("if", "Boolean!")
                    .with_description("When true fragment may be deferred")
                    .with_default(engine_value::ConstValue::Boolean(true)),
                MetaInputValue::new("label", "String")
                    .with_description("This label should be used by GraphQL clients to identify the data from patch responses and associate it with the correct fragment.")
            ]
                .into_iter()
                .map(|directive| (directive.name.clone(), directive))
                .collect(),
            is_repeatable: false,
            visible: None,
        });

        // register scalars
        <bool as LegacyInputType>::create_type_info(registry);
        <i32 as LegacyInputType>::create_type_info(registry);
        <f32 as LegacyInputType>::create_type_info(registry);
        <String as LegacyInputType>::create_type_info(registry);
        <ID as LegacyInputType>::create_type_info(registry);
    }

    /// Create a schema
    pub fn new(registry: Registry) -> Schema {
        Self::build(registry).finish()
    }

    #[inline]
    #[allow(unused)]
    pub fn registry(&self) -> &Registry {
        &self.env.registry
    }

    /// Returns SDL(Schema Definition Language) of this schema.
    pub fn sdl(&self) -> String {
        self.0.env.registry.export_sdl(false)
    }

    /// Returns Federation SDL(Schema Definition Language) of this schema.
    pub fn federation_sdl(&self) -> String {
        self.0.env.registry.export_sdl(true)
    }

    /// Get all names in this schema
    ///
    /// Maybe you want to serialize a custom binary protocol. In order to minimize message size, a dictionary
    /// is usually used to compress type names, field names, directive names, and parameter names. This function gets all the names,
    /// so you can create this dictionary.
    pub fn names(&self) -> Vec<String> {
        self.0.env.registry.names()
    }

    fn create_extensions(&self, session_data: Arc<Data>) -> Extensions {
        Extensions::new(
            self.extensions.iter().map(|f| f.create()),
            self.env.clone(),
            session_data,
        )
    }

    async fn prepare_request(
        &self,
        mut extensions: Extensions,
        request: Request,
        session_data: Arc<Data>,
    ) -> Result<(QueryEnvBuilder, CacheControl), Vec<ServerError>> {
        let mut request = request;
        let query_data = Arc::new(std::mem::take(&mut request.data));
        extensions.attach_query_data(query_data.clone());

        let request = extensions.prepare_request(request).await?;
        let mut document = {
            let query = &request.query;
            let fut_parse = async { parse_query(query).map_err(Into::<ServerError>::into) };
            futures_util::pin_mut!(fut_parse);
            extensions
                .parse_query(query, &request.variables, &mut fut_parse)
                .await?
        };

        // check rules
        let validation_result = {
            let validation_fut = async {
                check_rules(
                    &self.env.registry,
                    &document,
                    Some(&request.variables),
                    self.validation_mode,
                )
            };
            futures_util::pin_mut!(validation_fut);
            extensions.validation(&mut validation_fut).await?
        };

        // check limit
        if let Some(limit_complexity) = self.complexity {
            if validation_result.complexity > limit_complexity {
                return Err(vec![ServerError::new("Query is too complex.", None)]);
            }
        }

        if let Some(limit_depth) = self.depth {
            if validation_result.depth > limit_depth {
                return Err(vec![ServerError::new("Query is nested too deep.", None)]);
            }
        }

        let operation = if let Some(operation_name) = &request.operation_name {
            match document.operations {
                DocumentOperations::Single(_) => None,
                DocumentOperations::Multiple(mut operations) => operations
                    .remove(operation_name.as_str())
                    .map(|operation| (Some(operation_name.clone()), operation)),
            }
            .ok_or_else(|| ServerError::new(format!(r#"Unknown operation named "{operation_name}""#), None))
        } else {
            match document.operations {
                DocumentOperations::Single(operation) => Ok((None, operation)),
                DocumentOperations::Multiple(map) if map.len() == 1 => {
                    let (operation_name, operation) = map.into_iter().next().unwrap();
                    Ok((Some(operation_name.to_string()), operation))
                }
                DocumentOperations::Multiple(_) => Err(ServerError::new("Operation name required in request.", None)),
            }
        };

        let (operation_name, mut operation) = operation.map_err(|err| vec![err])?;

        // remove skipped fields
        for fragment in document.fragments.values_mut() {
            remove_skipped_selection(&mut fragment.node.selection_set.node, &request.variables);
        }
        remove_skipped_selection(&mut operation.node.selection_set.node, &request.variables);

        // We could have the whole flow here to create the LogicalQuery
        // As the rules passed, we could in theory have a working LogicalQuery
        //
        // Then we can pass it along with other variables to an execution layer
        // Or just print it for now.
        // LogicalQuery::build(document, registry);

        let env = QueryEnvInner {
            extensions,
            variables: request.variables,
            operation_name,
            operation,
            fragments: document.fragments,
            uploads: request.uploads,
            session_data,
            ctx_data: query_data,
            response_http_headers: Default::default(),
            disable_introspection: request.disable_introspection,
            errors: Default::default(),
            current_datetime: CurrentDateTime::new(),
            cache_invalidations: validation_result.cache_invalidation_policies,
            response: Default::default(),
            deferred_workloads: None,
        };
        Ok((QueryEnvBuilder::new(env), validation_result.cache_control))
    }

    async fn execute_once(&self, env: QueryEnv) -> Response {
        // execute
        let ctx = ContextSelectionSet {
            ty: self.registry().root_type(env.operation.node.ty),
            path: QueryPath::empty(),
            item: &env.operation.node.selection_set,
            schema_env: &self.env,
            query_env: &env,
        };

        let res = match &env.operation.node.ty {
            OperationType::Query => resolve_root_container(&ctx).await,
            OperationType::Mutation => resolve_root_container_serial(&ctx).await,
            OperationType::Subscription => Err(ServerError::new(
                "Subscriptions are not supported on this transport.",
                None,
            )),
        };

        let operation_name =
            env.operation_name
                .as_deref()
                .or_else(|| match env.operation.selection_set.node.items.as_slice() {
                    [Positioned {
                        node: Selection::Field(field),
                        ..
                    }] => Some(field.node.name.node.as_str()),
                    _ => None,
                });

        let mut resp = match res {
            Ok(value) => {
                let response = &mut *ctx.response().await;
                response.set_root_unchecked(value);
                Response::new(std::mem::take(response), operation_name, &env.operation)
            }
            Err(err) => Response::from_errors(vec![err], operation_name, &env.operation),
        }
        .http_headers(std::mem::take(&mut *env.response_http_headers.lock().unwrap()));

        resp.errors.extend(std::mem::take(&mut *env.errors.lock().unwrap()));
        resp
    }

    /// Execute a GraphQL query.
    pub async fn execute(&self, request: impl Into<Request>) -> Response {
        let request = request.into();
        let extensions = self.create_extensions(Default::default());
        let request_fut = {
            let extensions = extensions.clone();
            async move {
                match self.prepare_request(extensions, request, Default::default()).await {
                    Ok((env_builder, cache_control)) => {
                        let env = env_builder.build();
                        let fut = async { self.execute_once(env.clone()).await.cache_control(cache_control) };
                        futures_util::pin_mut!(fut);
                        env.extensions
                            .execute(env.operation_name.as_deref(), &env.operation, &mut fut)
                            .await
                    }
                    Err(errors) => Response::bad_request(errors),
                }
            }
        };
        futures_util::pin_mut!(request_fut);
        extensions.request(&mut request_fut).await
    }

    /// Execute a GraphQL batch query.
    pub async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse {
        match batch_request {
            BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
            BatchRequest::Batch(requests) => BatchResponse::Batch(
                futures_util::stream::iter(requests.into_iter())
                    .then(|request| self.execute(request))
                    .collect()
                    .await,
            ),
        }
    }

    /// Execute a GraphQL streaming request with session data
    ///
    /// This should be called when we receive some kind of streaming request.
    /// It can either serve a subscription or a query/mutation that makes
    /// use of `@stream` & `@defer`
    #[doc(hidden)]
    pub fn execute_stream_with_session_data(
        &self,
        request: impl Into<Request> + Send,
        session_data: Arc<Data>,
    ) -> impl Stream<Item = StreamingPayload> + Send + Unpin {
        let schema = self.clone();
        let request = request.into();
        let extensions = self.create_extensions(session_data.clone());

        futures_util::stream::StreamExt::boxed({
            let extensions = extensions.clone();
            async_stream::stream! {
                let (env_builder, cache_control) = match schema.prepare_request(extensions, request, session_data).await {
                    Ok(res) => res,
                    Err(errors) => {
                        yield Response::from_errors_with_type(errors, OperationType::Subscription).into_streaming_payload(false);
                        return;
                    }
                };

                if env_builder.operation_type() != OperationType::Subscription {
                    let (sender, mut receiver) = deferred::workload_channel();
                    let env = env_builder.with_deferred_sender(sender).build();

                    let initial_response = schema
                        .execute_once(env.clone())
                        .await
                        .cache_control(cache_control);

                    let mut next_workload = receiver.receive();

                    yield initial_response.into_streaming_payload(next_workload.is_some());

                    // For now we're taking the simple approach and running all the deferred
                    // workloads serially. We can look into doing something smarter later.
                    while let Some(workload) = next_workload {
                        let mut next_response = process_deferred_workload(workload, &schema, &env).await;
                        next_workload = receiver.receive();
                        next_response.has_next = next_workload.is_some();
                        yield next_response.into()
                    }
                    return;
                }

                let env = env_builder.build();

                let ctx = env.create_context(
                    &schema.env,
                    &env.operation.node.selection_set,
                    schema.registry().root_type(env.operation.node.ty),
                );

                let mut streams = Vec::new();
                if let Err(err) = collect_subscription_streams(&ctx, &crate::EmptySubscription, &mut streams) {
                    // This hasNext: false is probably not correct, but we dont' support subscriptios atm so whatever
                    yield Response::from_errors_with_type(vec![err], OperationType::Subscription).into_streaming_payload(false);
                }

                let mut stream = stream::select_all(streams);
                while let Some(resp) = stream.next().await {
                    yield resp.into_streaming_payload(false);
                }
            }
        })
    }

    /// Execute a GraphQL streaming request.
    ///
    /// This should be called when we receive some kind of streaming request.
    /// It can either serve a subscription or a query/mutation that makes
    /// use of `@stream` & `@defer`
    pub fn execute_stream(
        &self,
        request: impl Into<Request> + Send,
    ) -> impl Stream<Item = StreamingPayload> + Send + Unpin {
        self.execute_stream_with_session_data(request, Default::default())
    }
}

async fn process_deferred_workload(
    workload: deferred::DeferredWorkload,
    schema: &Schema,
    env: &QueryEnv,
) -> IncrementalPayload {
    let context = workload.to_context(&schema.env, env);
    let result = resolver_utils::resolve_deferred_container(&context, workload.parent_resolver_value.clone()).await;

    let mut data = std::mem::take(&mut *context.response().await);
    let mut errors = std::mem::take(&mut *context.query_env.errors.lock().expect("to be able to lock this mutex"));

    let root_node = match result {
        Ok(root_node) => root_node,
        Err(error) => {
            errors.push(error);
            data.insert_node(CompactValue::Null)
        }
    };

    data.set_root_unchecked(root_node);

    IncrementalPayload {
        label: workload.label,
        data,
        path: workload.path,
        has_next: false, // We hardcode this to false here, the function calling us should override
        errors,
    }
}

fn remove_skipped_selection(selection_set: &mut SelectionSet, variables: &Variables) {
    fn is_skipped(directives: &[Positioned<Directive>], variables: &Variables) -> bool {
        for directive in directives {
            let include = match &*directive.node.name.node {
                "skip" => false,
                "include" => true,
                _ => continue,
            };

            if let Some(condition_input) = directive.node.get_argument("if") {
                let value = condition_input
                    .node
                    .clone()
                    .into_const_with(|name| variables.get(&name).cloned().ok_or(()))
                    .unwrap_or_default();
                let value: bool = LegacyInputType::parse(Some(value)).unwrap_or_default();
                if include != value {
                    return true;
                }
            }
        }

        false
    }

    selection_set
        .items
        .retain(|selection| !is_skipped(selection.node.directives(), variables));

    for selection in &mut selection_set.items {
        selection
            .node
            .directives_mut()
            .retain(|directive| directive.node.name.node != "skip" && directive.node.name.node != "include");
    }

    for selection in &mut selection_set.items {
        match &mut selection.node {
            Selection::Field(field) => {
                remove_skipped_selection(&mut field.node.selection_set.node, variables);
            }
            Selection::FragmentSpread(_) => {}
            Selection::InlineFragment(inline_fragment) => {
                remove_skipped_selection(&mut inline_fragment.node.selection_set.node, variables);
            }
        }
    }
}
