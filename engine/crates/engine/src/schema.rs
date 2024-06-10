use std::any::TypeId;
use std::{any::Any, ops::Deref, sync::Arc};

use engine_validation::check_strict_rules;
use futures_util::stream::{self, Stream, StreamExt};
use futures_util::FutureExt;
use grafbase_tracing::gql_response_status::GraphqlResponseStatus;
use grafbase_tracing::metrics::GraphqlOperationMetrics;
use grafbase_tracing::span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GqlRequestAttributes};
use graph_entities::{CompactValue, QueryResponse};

use registry_v2::OperationLimits;
use tracing::{Instrument, Span};

use crate::registry::type_kinds::SelectionSetTarget;
use crate::{
    context::{Data, QueryEnvInner},
    current_datetime::CurrentDateTime,
    deferred,
    extensions::{ExtensionFactory, Extensions},
    parser::{
        parse_query,
        types::{Directive, DocumentOperations, OperationType, Selection, SelectionSet},
        Positioned,
    },
    registry::Registry,
    registry::RegistrySdlExt,
    resolver_utils::{self, resolve_root_container, resolve_root_container_serial},
    response::{IncrementalPayload, StreamingPayload},
    subscription::collect_subscription_streams,
    types::QueryRoot,
    BatchRequest, BatchResponse, CacheControl, ContextExt, ContextSelectionSet, LegacyInputType, LegacyOutputType,
    ObjectType, QueryEnv, QueryEnvBuilder, QueryPath, Request, Response, ServerError, SubscriptionType, Variables,
};
use crate::{new_futures_spawner, registry_operation_type_from_parser, QuerySpawnedFuturesWaiter};

/// Schema builder
pub struct SchemaBuilder {
    registry: Arc<registry_v2::Registry>,
    data: Data,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    operation_metrics: GraphqlOperationMetrics,
}

impl SchemaBuilder {
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

    /// Build schema.
    pub fn finish(self) -> Schema {
        Schema(Arc::new(SchemaInner {
            operation_limits: self.registry.operation_limits.clone(),
            extensions: self.extensions,
            env: SchemaEnv(Arc::new(SchemaEnvInner {
                registry: self.registry,
                data: self.data,
                operation_metrics: self.operation_metrics,
            })),
        }))
    }
}

#[doc(hidden)]
pub struct SchemaEnvInner {
    pub registry: Arc<registry_v2::Registry>,
    pub operation_metrics: GraphqlOperationMetrics,
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
    pub(crate) operation_limits: OperationLimits,
    pub(crate) extensions: Vec<Box<dyn ExtensionFactory>>,
    pub env: SchemaEnv,
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
    pub fn build(
        registry: Arc<registry_v2::Registry>,
        meter: grafbase_tracing::otel::opentelemetry::metrics::Meter,
    ) -> SchemaBuilder {
        SchemaBuilder {
            registry,
            data: Default::default(),
            extensions: Default::default(),
            operation_metrics: GraphqlOperationMetrics::build(&meter),
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

        registry.add_builtins_to_registry();

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
        let mut registry = Registry::default();

        registry.add_builtins_to_registry();

        registry.remove_unused_types();
        registry
    }

    /// Create a schema
    pub fn new(registry: Arc<registry_v2::Registry>) -> Schema {
        Self::build(registry, grafbase_tracing::metrics::meter_from_global_provider()).finish()
    }

    #[inline]
    #[allow(unused)]
    pub fn registry(&self) -> &registry_v2::Registry {
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
    ) -> Result<(QueryEnvBuilder, QuerySpawnedFuturesWaiter, CacheControl), Vec<ServerError>> {
        let mut request = request;
        let query_data = Arc::new(std::mem::take(&mut request.data));
        extensions.attach_query_data(query_data.clone());

        let request = extensions.prepare_request(request).await?;
        let mut document = {
            let query = request.query();
            let fut_parse = async { parse_query(query).map_err(Into::<ServerError>::into) };
            futures_util::pin_mut!(fut_parse);
            extensions
                .parse_query(query, &request.variables, &mut fut_parse)
                .await?
        };

        // check rules
        let validation_result = {
            let validation_fut = async {
                check_strict_rules(&self.env.registry, &document, Some(&request.variables))
                    .map_err(|errors| errors.into_iter().map(ServerError::from).collect())
            };
            futures_util::pin_mut!(validation_fut);
            extensions.validation(&mut validation_fut).await?
        };

        if !request.operation_limits_disabled() {
            // Check limits.
            if let Some(limit_complexity) = self.operation_limits.complexity {
                if validation_result.complexity > limit_complexity as usize {
                    return Err(vec![ServerError::new("Query is too complex.", None)]);
                }
            }

            if let Some(limit_depth) = self.operation_limits.depth {
                if validation_result.depth > limit_depth as usize {
                    return Err(vec![ServerError::new("Query is nested too deep.", None)]);
                }
            }

            if let Some(height) = self.operation_limits.height {
                if validation_result.height > height as usize {
                    return Err(vec![ServerError::new("Query is too high.", None)]);
                }
            }

            if let Some(root_field_count) = self.operation_limits.root_fields {
                if validation_result.root_field_count > root_field_count as usize {
                    return Err(vec![ServerError::new("Query has too many root fields.", None)]);
                }
            }

            if let Some(alias_count) = self.operation_limits.aliases {
                if validation_result.alias_count > alias_count as usize {
                    return Err(vec![ServerError::new("Query has too many aliases.", None)]);
                }
            }
        }

        let operation = if let Some(operation_name) = request.operation_name() {
            match document.operations {
                DocumentOperations::Single(_) => None,
                DocumentOperations::Multiple(mut operations) => operations
                    .remove(operation_name)
                    .map(|operation| (Some(operation_name.to_string()), operation)),
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

        let introspection_state = request.introspection_state();
        let (futures_spawner, futures_waiter) = new_futures_spawner();
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
            introspection_state,
            errors: Default::default(),
            current_datetime: CurrentDateTime::new(),
            cache_invalidations: validation_result.cache_invalidation_policies,
            response: Default::default(),
            deferred_workloads: None,
            futures_spawner,
        };
        Ok((
            QueryEnvBuilder::new(env),
            futures_waiter,
            validation_result.cache_control,
        ))
    }

    async fn execute_once(&self, env: QueryEnv, futures_waiter: QuerySpawnedFuturesWaiter) -> Response {
        // execute
        let ctx = ContextSelectionSet {
            ty: self
                .registry()
                .root_type(registry_operation_type_from_parser(env.operation.node.ty))
                .and_then(|ty| SelectionSetTarget::try_from(ty).ok())
                .expect("registry is malformed"),
            path: QueryPath::empty(),
            item: &env.operation.node.selection_set,
            schema_env: &self.env,
            query_env: &env,
        };

        let execution = async {
            match &env.operation.node.ty {
                OperationType::Query => resolve_root_container(&ctx).await,
                OperationType::Mutation => resolve_root_container_serial(&ctx).await,
                OperationType::Subscription => Err(ServerError::new(
                    "Subscriptions are not supported on this transport.",
                    None,
                )),
            }
        };
        let res = futures_util::select! {
            res = execution.fuse() => res,
            _ = futures_waiter.wait_until_no_spawners_left().fuse() => unreachable!(),
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
                let data = &mut *ctx.response().await;
                data.set_root_unchecked(value);
                Response {
                    data: std::mem::take(data),
                    ..Default::default()
                }
            }
            Err(err) => Response {
                // At this point it can't be a request error anymore, so data must be present.
                // Having an error propagating here just means it propagated up to the root and
                // data is null.
                data: QueryResponse::new_root(CompactValue::Null),
                errors: vec![err],
                ..Default::default()
            },
        }
        .http_headers(std::mem::take(&mut *env.response_http_headers.lock().unwrap()))
        .with_graphql_operation_from(operation_name, &env.operation.node);

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
                    Ok((env_builder, futures_waiter, cache_control)) => {
                        let env = env_builder.build();
                        Span::current().record_gql_request(GqlRequestAttributes {
                            operation_type: env.operation.ty.as_str(),
                            operation_name: env.operation_name.clone(),
                        });

                        let fut = async {
                            self.execute_once(env.clone(), futures_waiter)
                                .await
                                .cache_control(cache_control)
                        };
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
        let start = web_time::Instant::now();
        let schema = self.clone();
        let request: Request = request.into();
        let extensions = self.create_extensions(session_data.clone());
        let gql_span = GqlRequestSpan::new().into_span();
        let client = schema
            .env
            .data
            .get(&TypeId::of::<runtime::Context>())
            .and_then(|data| data.downcast_ref::<runtime::Context>())
            .and_then(|ctx| grafbase_tracing::grafbase_client::Client::extract_from(ctx.headers()));

        let normalized_query = operation_normalizer::normalize(request.query(), request.operation_name()).ok();

        let request = futures_util::stream::StreamExt::boxed({
            let extensions = extensions.clone();
            async_stream::stream! {
                let (env_builder, futures_waiter, cache_control) = match schema.prepare_request(extensions, request, session_data).await {
                    Ok(res) => res,
                    Err(errors) => {
                        Span::current().record_gql_status(GraphqlResponseStatus::RequestError { count: errors.len() as u64 });
                        yield Response::from_errors_with_type(errors, OperationType::Subscription).into_streaming_payload(false);
                        return;
                    }
                };

                let mut status = GraphqlResponseStatus::Success;
                let env = if env_builder.operation_type() != OperationType::Subscription {
                    let (sender, mut receiver) = deferred::workload_channel();
                    let env = env_builder.with_deferred_sender(sender).build();
                    Span::current().record_gql_request(GqlRequestAttributes {
                        operation_type: env.operation.ty.as_str(),
                        operation_name: env.operation_name.clone()
                    });

                    let initial_response = schema
                        .execute_once(env.clone(), futures_waiter)
                        .await
                        .cache_control(cache_control);
                    status = initial_response.status();

                    let mut next_workload = receiver.receive();

                    yield initial_response.into_streaming_payload(next_workload.is_some());

                    // For now we're taking the simple approach and running all the deferred
                    // workloads serially. We can look into doing something smarter later.
                    while let Some(workload) = next_workload {
                        let mut next_response = process_deferred_workload(workload, &schema, &env).await;
                        next_workload = receiver.receive();
                        next_response.has_next = next_workload.is_some();
                        let response: StreamingPayload = next_response.into();
                        status = status.union(response.status());

                        yield response
                    }
                    env
                } else {
                    let env = env_builder.build();

                    Span::current().record_gql_request(GqlRequestAttributes {
                        operation_type: env.operation.ty.as_str(),
                        operation_name: env.operation_name.clone()
                    });

                    let ctx = env.create_context(
                        &schema.env,
                        &env.operation.node.selection_set,
                        schema.registry().root_type(registry_operation_type_from_parser(env.operation.node.ty)).and_then(|ty| SelectionSetTarget::try_from(ty).ok()).expect("registry is malformed"),
                    );

                    let mut streams = Vec::new();
                    if let Err(err) = collect_subscription_streams(&ctx, &crate::EmptySubscription, &mut streams) {
                        status = GraphqlResponseStatus::RequestError {count: 1};
                        // This hasNext: false is probably not correct, but we dont' support subscriptios atm so whatever
                        yield Response::from_errors_with_type(vec![err], OperationType::Subscription).into_streaming_payload(false);
                    }

                    let mut stream = stream::select_all(streams);
                    while let Some(resp) = stream.next().await {
                        let response = resp.into_streaming_payload(false);
                        status = status.union(response.status());
                        yield response
                    }
                    env.clone()
                };

                Span::current().record_gql_status(status);
                if let Some(normalized_query) = normalized_query {
                    schema.env.operation_metrics.record(
                        grafbase_tracing::metrics::GraphqlOperationMetricsAttributes {
                            ty: env.operation.ty.as_str(),
                            name: env.operation_name.clone(),
                            normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                            normalized_query,
                            status,
                            cache_status: None,
                            client
                        },
                        start.elapsed(),
                    );
                }
            }
        });

        request.instrument(gql_span).into_inner()
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

impl From<engine_validation::RuleError> for ServerError {
    fn from(e: engine_validation::RuleError) -> Self {
        Self {
            message: e.message,
            source: None,
            locations: e.locations,
            path: Vec::new(),
            extensions: None,
        }
    }
}
