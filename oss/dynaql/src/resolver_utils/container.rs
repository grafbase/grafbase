use std::{future::Future, pin::Pin, sync::Arc};

use dynaql_parser::{types::SelectionSet, Pos, Positioned};
use futures_util::FutureExt;
use graph_entities::{CompactValue, NodeID, ResponseContainer, ResponseNodeId, ResponseNodeRelation};

use crate::{
    extensions::ResolveInfo,
    parser::types::Selection,
    registry::{resolvers::ResolvedValue, MetaType, Registry},
    relations_edges, Context, ContextBase, ContextSelectionSet, Error, LegacyOutputType, Name, ServerError,
    ServerResult, Value,
};

use super::field::resolve_field;

/// Represents a GraphQL container object.
///
/// This helper trait allows the type to call `resolve_container` on itself in its
/// `OutputType::resolve` implementation.
#[async_trait::async_trait]
pub trait ContainerType: LegacyOutputType {
    /// This function returns true of type `EmptyMutation` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value `dynaql::Value`.
    ///
    /// If the field was not found returns None.
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<ResponseNodeId>>;

    /// Collect all the fields of the container that are queried in the selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must call it on their
    /// internal type.
    fn collect_all_fields_native<'a>(
        &'a self,
        ctx: &ContextSelectionSet<'a>,
        fields: &mut Fields<'a>,
    ) -> ServerResult<()>
    where
        Self: Send + Sync,
    {
        fields.add_set_native(ctx, self)
    }

    /// Find the GraphQL entity with the given name from the parameter.
    ///
    /// Objects should override this in case they are the query root.
    async fn find_entity(&self, _: &Context<'_>, _params: &Value) -> ServerResult<Option<Value>> {
        Ok(None)
    }
}

/// Collect all the fields of the container that are queried in the selection set.
///
/// Objects do not have to override this, but interfaces and unions must call it on their
/// internal type.
fn collect_all_fields_graph_meta<'a>(
    ty: &'a MetaType,
    ctx: &ContextSelectionSet<'a>,
    fields: &mut FieldExecutionSet<'a>,
    node_id: Option<NodeID<'a>>,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<()> {
    fields.add_selection_set(ctx, ty, node_id, parent_resolver_value)
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(*self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(*self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Arc<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Box<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType, E: Into<Error> + Send + Sync + Clone> ContainerType for Result<T, E> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<ResponseNodeId>> {
        match self {
            Ok(value) => T::resolve_field(value, ctx).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        match self {
            Ok(value) => T::find_entity(value, ctx, params).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_root_container<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, root, None, None).await
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_root_container_serial<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, false, root, None, None).await
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_container_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner_native(ctx, root, true).await
}

pub(super) async fn resolve_container<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &MetaType,
    node_id: Option<NodeID<'a>>,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, root, node_id, parent_resolver_value).await
}

async fn resolve_container_inner<'a>(
    ctx: &ContextSelectionSet<'a>,
    parallel: bool,
    root: &MetaType,
    node_id: Option<NodeID<'a>>,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<ResponseNodeId> {
    #[cfg(feature = "tracing_worker")]
    {
        logworker::trace!(ctx.trace_id(), "Where: {}", root.name());
        logworker::trace!(ctx.trace_id(), "Id: {:?}", node_id);
    }

    let mut fields = FieldExecutionSet(Vec::new());
    fields.add_selection_set(ctx, root, node_id.clone(), parent_resolver_value)?;

    let results = if parallel {
        futures_util::future::try_join_all(fields.0).await?
    } else {
        let mut results = Vec::with_capacity(fields.0.len());
        for field in fields.0 {
            results.push(field.await?);
        }
        results
    };

    let results = results.flatten();

    let relations = relations_edges(ctx, root);

    if let Some(node_id) = node_id {
        let mut container = ResponseContainer::new_node(node_id);
        for ((alias, name), value) in results {
            let name = name.to_string();
            let alias = alias.map(|x| x.to_string().into());
            // Temp: little hack while we rework the execution step, we should not do that here to
            // follow OneToMany relations.
            if let Some(relation) = relations.get(&name) {
                container.insert(
                    ResponseNodeRelation::relation(
                        name,
                        relation.name.clone(),
                        relation.relation.0.as_ref().map(ToString::to_string),
                        relation.relation.1.to_string(),
                    ),
                    value,
                );
            } else {
                container.insert(
                    ResponseNodeRelation::NotARelation {
                        field: name.into(),
                        response_key: alias,
                    },
                    value,
                );
            }
        }
        Ok(ctx.response_graph.write().await.insert_node(container))
    } else {
        let mut container = ResponseContainer::new_container();
        for ((alias, name), value) in results {
            let name = name.to_string();
            let alias = alias.map(|x| x.to_string().into());

            if let Some(relation) = relations.get(&name) {
                container.insert(
                    ResponseNodeRelation::relation(
                        name,
                        relation.name.clone(),
                        relation.relation.0.as_ref().map(ToString::to_string),
                        relation.relation.1.to_string(),
                    ),
                    value,
                );
            } else {
                container.insert(
                    ResponseNodeRelation::NotARelation {
                        field: name.into(),
                        response_key: alias,
                    },
                    value,
                );
            }
        }
        Ok(ctx.response_graph.write().await.insert_node(container))
    }
}

async fn resolve_container_inner_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    parallel: bool,
) -> ServerResult<ResponseNodeId> {
    let mut fields = Fields(Vec::new());
    fields.add_set_native(ctx, root)?;

    let res = if parallel {
        futures_util::future::try_join_all(fields.0).await?
    } else {
        let mut results = Vec::with_capacity(fields.0.len());
        for field in fields.0 {
            results.push(field.await?);
        }
        results
    };

    let container = ResponseContainer::with_children(res.into_iter().map(|(name, value)| {
        (
            ResponseNodeRelation::not_a_relation(name.to_string().into(), None),
            value,
        )
    }));

    Ok(ctx.response_graph.write().await.insert_node(container))
}

/// We take individual selections from our selection set and convert those into futures.
///
/// Each of those futures will put out one of these.
#[derive(Debug)]
enum FieldExecutionOutput {
    /// Field selections are going to put out one of these variants.
    Field((Option<Name>, Name), ResponseNodeId),
    /// Spreads or fragments are going to put out one of these because each spread can
    /// resolve to multiple inner fields.
    MultipleFields(Vec<((Option<Name>, Name), ResponseNodeId)>),
}

type FieldExecutionFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<FieldExecutionOutput>> + 'a + Send>>;

/// A set of futures associated with the fields of a selection set.
///
/// Running these futures should populate the response_graph with the results of the selection set
pub struct FieldExecutionSet<'a>(Vec<FieldExecutionFuture<'a>>);

async fn response_id_unwrap_or_null(ctx: &Context<'_>, opt_id: Option<ResponseNodeId>) -> ResponseNodeId {
    if let Some(id) = opt_id {
        id
    } else {
        ctx.response_graph.write().await.insert_node(CompactValue::Null)
    }
}

impl<'a> FieldExecutionSet<'a> {
    /// Creates futures for all the fields in the given selection set and adds them
    /// to the field execution set
    pub fn add_selection_set(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a MetaType,
        current_node_id: Option<NodeID<'a>>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            let parent_resolver_value = parent_resolver_value.clone();
            let current_node_id = current_node_id.clone();
            match &selection.node {
                Selection::Field(field) => {
                    self.add_field(ctx, root, field, parent_resolver_value);
                }
                Selection::FragmentSpread(_) | Selection::InlineFragment(_) => {
                    let fragment_details = FragmentDetails::from_fragment_selection(ctx, &selection.node)?;
                    self.add_spread(ctx, root, fragment_details, current_node_id, parent_resolver_value)?;
                }
            }
        }
        Ok(())
    }

    // Adds a field to the FieldsGraph
    fn add_field(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a MetaType,
        field: &'a Positioned<dynaql_parser::types::Field>,
        parent_resolver_value: Option<ResolvedValue>,
    ) {
        if field.node.name.node == "__typename" {
            let ctx = ctx.clone();
            let field_name = field.node.name.node.clone();
            let alias = field.node.alias.clone().map(|x| x.node);

            self.0.push(Box::pin({
                async move {
                    let node = CompactValue::String(resolve_typename(root, parent_resolver_value.as_ref()).await);
                    Ok(FieldExecutionOutput::Field(
                        (alias, field_name),
                        ctx.response_graph.write().await.insert_node(node),
                    ))
                }
            }));
            return;
        }
        self.0.push(Box::pin({
            let ctx = ctx.clone();
            async move {
                let ctx_field = ctx.with_field(field, Some(root), Some(&ctx.item.node));
                let field_name = ctx_field.item.node.name.node.clone();
                let alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                let extensions = &ctx.query_env.extensions;

                let resolve_fut = resolve_field(&ctx_field, root, parent_resolver_value);

                if extensions.is_empty() && field.node.directives.is_empty() {
                    // If we've no extensions or directives, just return the data
                    return Ok(FieldExecutionOutput::Field(
                        (alias, field_name),
                        response_id_unwrap_or_null(&ctx_field, resolve_fut.await?).await,
                    ));
                }

                let type_name = root.name();
                #[cfg(feature = "tracing_worker")]
                {
                    logworker::trace!(
                        ctx.trace_id(),
                        "Resolving {field} on {type_name}",
                        field = field.node.name.node.as_str()
                    );
                }
                let args_values: Vec<(Positioned<Name>, Option<Value>)> = ctx_field
                    .item
                    .node
                    .arguments
                    .clone()
                    .into_iter()
                    .map(|(key, val)| (key, ctx_field.resolve_input_value(val).ok()))
                    .collect();

                let meta_field = ctx_field
                    .schema_env
                    .registry
                    .types
                    .get(type_name)
                    .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()));

                let resolve_info = ResolveInfo {
                    path_node: ctx_field.path_node.as_ref().unwrap(),
                    parent_type: &type_name,
                    return_type: match meta_field.map(|field| &field.ty) {
                        Some(ty) => &ty,
                        None => {
                            return Err(ServerError::new(
                                format!(r#"Cannot query field "{field_name}" on type "{type_name}"self."#),
                                Some(ctx_field.item.pos),
                            ));
                        }
                    },
                    name: field.node.name.node.as_str(),
                    alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                    required_operation: meta_field.and_then(|f| f.required_operation),
                    auth: meta_field.and_then(|f| f.auth.as_ref()),
                    input_values: args_values,
                };

                if field.node.directives.is_empty() {
                    futures_util::pin_mut!(resolve_fut);
                    return Ok(FieldExecutionOutput::Field(
                        (alias, field_name),
                        response_id_unwrap_or_null(
                            &ctx_field,
                            extensions.resolve(resolve_info, &mut resolve_fut).await?,
                        )
                        .await,
                    ));
                }

                let mut resolve_fut = resolve_fut.boxed();

                for directive in &field.node.directives {
                    if let Some(directive_factory) =
                        ctx.schema_env.custom_directives.get(directive.node.name.node.as_str())
                    {
                        let ctx_directive = ContextBase {
                            path_node: ctx_field.path_node,
                            resolver_node: ctx_field.resolver_node.clone(),
                            item: directive,
                            schema_env: ctx_field.schema_env,
                            query_env: ctx_field.query_env,
                            resolvers_data: ctx_field.resolvers_data.clone(),
                            response_graph: ctx_field.response_graph.clone(),
                        };
                        let directive_instance = directive_factory.create(&ctx_directive, &directive.node)?;
                        resolve_fut = Box::pin({
                            let ctx_field = ctx_field.clone();
                            async move { directive_instance.resolve_field(&ctx_field, &mut resolve_fut).await }
                        });
                    }
                }

                Ok(FieldExecutionOutput::Field(
                    (alias, field_name),
                    response_id_unwrap_or_null(&ctx_field, extensions.resolve(resolve_info, &mut resolve_fut).await?)
                        .await,
                ))
            }
        }));
    }

    /// Adds futures for an inline/named fragment spread to the set
    fn add_spread(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a MetaType,
        fragment_details: FragmentDetails<'a>,
        current_node_id: Option<NodeID<'a>>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> Result<(), ServerError> {
        let type_condition = fragment_details.type_condition;
        match root {
            MetaType::Union { .. } => {
                let type_condition = type_condition.ok_or_else(|| {
                    ServerError::new(
                        "Spreads on union types require a type condition",
                        Some(fragment_details.position),
                    )
                })?;

                self.0.push(Box::pin({
                    let ctx = ctx.clone();
                    async move {
                        resolve_spread_with_type_condition(
                            ctx,
                            type_condition,
                            root,
                            fragment_details.selection_set,
                            current_node_id,
                            parent_resolver_value,
                        )
                        .await
                    }
                }));
            }
            MetaType::Interface { .. } if type_condition.is_some() => {
                let type_condition = type_condition.unwrap();

                self.0.push(Box::pin({
                    let ctx = ctx.clone();
                    async move {
                        resolve_spread_with_type_condition(
                            ctx,
                            type_condition,
                            root,
                            fragment_details.selection_set,
                            current_node_id,
                            parent_resolver_value,
                        )
                        .await
                    }
                }));
            }
            _ => {
                self.add_spread_fields(
                    root,
                    type_condition,
                    &ctx.with_selection_set(fragment_details.selection_set),
                    current_node_id,
                    parent_resolver_value,
                )?;
            }
        }
        Ok(())
    }

    /// Adds spread fields to the current set in the case where we're not on a union.
    fn add_spread_fields(
        &mut self,
        root: &'a MetaType,
        type_condition: Option<&str>,
        ctx: &ContextSelectionSet<'a>,
        current_node_id: Option<NodeID<'a>>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> Result<(), ServerError> {
        let introspection_type_name = root.name();
        let applies_concrete_object = type_condition.map_or(false, |condition| {
            typename_matches_condition(introspection_type_name, condition, root, &ctx.schema_env.registry)
        });

        if applies_concrete_object {
            collect_all_fields_graph_meta(root, ctx, self, current_node_id, parent_resolver_value)?;
        } else if type_condition.map_or(true, |condition| root.name() == condition) {
            // The fragment applies to an interface type.
            // self.add_set(&ctx, root)?;
            todo!()
        }

        Ok(())
    }
}

async fn resolve_spread_with_type_condition<'a>(
    ctx: ContextSelectionSet<'a>,
    type_condition: &str,
    containing_type: &'a MetaType,
    selection_set: &Positioned<dynaql_parser::types::SelectionSet>,
    current_node_id: Option<NodeID<'a>>,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<FieldExecutionOutput> {
    let registry = ctx.registry();
    let typename = resolve_typename(containing_type, parent_resolver_value.as_ref()).await;

    if !typename_matches_condition(&typename, type_condition, containing_type, registry) {
        return Ok(FieldExecutionOutput::MultipleFields(vec![]));
    }

    let subtype = registry
        .types
        .get(&typename)
        .ok_or_else(|| ServerError::new(format!(r#"Found an unknown typename: "{typename}"."#,), None))?;

    let mut subfields = FieldExecutionSet(Vec::new());
    subfields.add_selection_set(
        &ctx.with_selection_set(selection_set),
        subtype,
        current_node_id,
        parent_resolver_value,
    )?;

    Ok(FieldExecutionOutput::MultipleFields(
        futures_util::future::try_join_all(subfields.0).await?.flatten(),
    ))
}

fn typename_matches_condition(
    typename: &str,
    condition: &str,
    containing_type: &MetaType,
    registry: &Registry,
) -> bool {
    match containing_type {
        MetaType::Union(union) => typename == condition || condition == union.rust_typename,
        _ => {
            typename == condition
                || registry
                    .implements
                    .get(typename)
                    .map(|interfaces| interfaces.contains(condition))
                    .unwrap_or_default()
        }
    }
}

async fn resolve_typename<'a>(root: &'a MetaType, parent_resolver_value: Option<&ResolvedValue>) -> String {
    match root {
        MetaType::Union(_) | MetaType::Interface(_) => {
            if let Some(typename) = resolve_remote_typename(parent_resolver_value).await {
                return typename;
            }
        }
        _ => {}
    }

    root.name().to_string()
}

/// The `@openapi` & `@graphql` connectors, put the __typename into the JSON
/// they return.  This function returns that if present.
///
/// We should only need to call this for unions & interfaces - any other type and
/// we'll know the __typename ourselves based on context
async fn resolve_remote_typename<'a>(parent_resolver_value: Option<&ResolvedValue>) -> Option<String> {
    Some(
        parent_resolver_value?
            .data_resolved()
            .as_object()?
            .get("__typename")?
            .as_str()?
            .to_owned(),
    )
}

type BoxFieldFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<(Name, ResponseNodeId)>> + 'a + Send>>;
/// A set of fields on an container that are being selected.
pub struct Fields<'a>(Vec<BoxFieldFuture<'a>>);

impl<'a> Fields<'a> {
    /// Add another set of fields to this set of fields using the given container.
    /// Native way of resolving
    pub fn add_set_native<T: ContainerType + ?Sized>(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a T,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        // Get the typename
                        let ctx_field = ctx.with_field(field, None, Some(&ctx.item.node));
                        let field_name = ctx_field.item.node.response_key().node.clone();
                        let typename = root.introspection_type_name().into_owned();

                        self.0.push(Box::pin(async move {
                            let node = CompactValue::String(typename);
                            Ok((field_name, ctx_field.response_graph.write().await.insert_node(node)))
                        }));
                        continue;
                    }

                    let resolve_fut = Box::pin({
                        let ctx = ctx.clone();
                        async move {
                            let ctx_field = ctx.with_field(field, None, Some(&ctx.item.node));
                            let field_name = ctx_field.item.node.response_key().node.clone();
                            let extensions = &ctx.query_env.extensions;
                            let args_values: Vec<(Positioned<Name>, Option<Value>)> = ctx_field
                                .item
                                .node
                                .arguments
                                .clone()
                                .into_iter()
                                .map(|(key, val)| (key, ctx_field.resolve_input_value(val).ok()))
                                .collect();

                            if extensions.is_empty() && field.node.directives.is_empty() {
                                Ok((
                                    field_name,
                                    response_id_unwrap_or_null(&ctx_field, root.resolve_field(&ctx_field).await?).await,
                                ))
                            } else {
                                let type_name = T::type_name();
                                let meta_field = ctx_field
                                    .schema_env
                                    .registry
                                    .types
                                    .get(type_name.as_ref())
                                    .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()));

                                let resolve_info = ResolveInfo {
                                    path_node: ctx_field.path_node.as_ref().unwrap(),
                                    parent_type: &type_name,
                                    return_type: match meta_field.map(|field| &field.ty) {
                                        Some(ty) => &ty,
                                        None => {
                                            return Err(ServerError::new(
                                                format!(r#"Cannot query field "{field_name}" on type "{type_name}"."#),
                                                Some(ctx_field.item.pos),
                                            ));
                                        }
                                    },
                                    name: field.node.name.node.as_str(),
                                    alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                                    required_operation: meta_field.and_then(|f| f.required_operation),
                                    auth: meta_field.and_then(|f| f.auth.as_ref()),
                                    input_values: args_values,
                                };

                                let resolve_fut = async {
                                    let a = root.resolve_field(&ctx_field).await?;
                                    Ok(a)
                                };

                                if field.node.directives.is_empty() {
                                    futures_util::pin_mut!(resolve_fut);
                                    Ok((
                                        field_name,
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions.resolve(resolve_info, &mut resolve_fut).await?,
                                        )
                                        .await,
                                    ))
                                } else {
                                    let mut resolve_fut = resolve_fut.boxed();

                                    for directive in &field.node.directives {
                                        if let Some(directive_factory) =
                                            ctx.schema_env.custom_directives.get(directive.node.name.node.as_str())
                                        {
                                            let ctx_directive = ContextBase {
                                                path_node: ctx_field.path_node,
                                                resolver_node: ctx_field.resolver_node.clone(),
                                                item: directive,
                                                schema_env: ctx_field.schema_env,
                                                query_env: ctx_field.query_env,
                                                resolvers_data: ctx_field.resolvers_data.clone(),
                                                response_graph: ctx_field.response_graph.clone(),
                                            };
                                            let directive_instance =
                                                directive_factory.create(&ctx_directive, &directive.node)?;
                                            resolve_fut = Box::pin({
                                                let ctx_field = ctx_field.clone();
                                                async move {
                                                    directive_instance.resolve_field(&ctx_field, &mut resolve_fut).await
                                                }
                                            });
                                        }
                                    }

                                    Ok((
                                        field_name,
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions.resolve(resolve_info, &mut resolve_fut).await?,
                                        )
                                        .await,
                                    ))
                                }
                            }
                        }
                    });

                    self.0.push(resolve_fut);
                }
                selection => {
                    let FragmentDetails {
                        type_condition,
                        selection_set,
                        ..
                    } = FragmentDetails::from_fragment_selection(ctx, selection)?;

                    let introspection_type_name = root.introspection_type_name();

                    let applies_concrete_object = type_condition.map_or(false, |condition| {
                        introspection_type_name == condition
                            || ctx
                                .schema_env
                                .registry
                                .implements
                                .get(&*introspection_type_name)
                                .map_or(false, |interfaces| interfaces.contains(condition))
                    });
                    if applies_concrete_object {
                        root.collect_all_fields_native(&ctx.with_selection_set(selection_set), self)?;
                    } else if type_condition.map_or(true, |condition| T::type_name() == condition) {
                        // The fragment applies to an interface type.
                        self.add_set_native(&ctx.with_selection_set(selection_set), root)?;
                    }
                }
            }
        }
        Ok(())
    }
}

trait VecGraphFutureOutputExt {
    fn flatten(self) -> Vec<((Option<Name>, Name), ResponseNodeId)>;
}

impl VecGraphFutureOutputExt for Vec<FieldExecutionOutput> {
    fn flatten(self) -> Vec<((Option<Name>, Name), ResponseNodeId)> {
        self.into_iter()
            .flat_map(|result| match result {
                FieldExecutionOutput::Field(names, value) => vec![(names, value)],
                FieldExecutionOutput::MultipleFields(fields) => fields,
            })
            .collect()
    }
}

/// The details of a fragment spread/inline fragment.
///
/// Used to simplify handling each
struct FragmentDetails<'a> {
    position: Pos,
    type_condition: Option<&'a str>,
    selection_set: &'a Positioned<SelectionSet>,
}

impl<'a> FragmentDetails<'a> {
    fn from_fragment_selection(
        ctx: &ContextBase<'a, &Positioned<SelectionSet>>,
        selection: &'a Selection,
    ) -> Result<FragmentDetails<'a>, ServerError> {
        match selection {
            Selection::Field(_) => unreachable!("this should have been validated before calling this function"),
            Selection::FragmentSpread(spread) => {
                let fragment = ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                let fragment = match fragment {
                    Some(fragment) => fragment,
                    None => {
                        return Err(ServerError::new(
                            format!(r#"Unknown fragment "{}"."#, spread.node.fragment_name.node),
                            Some(spread.pos),
                        ));
                    }
                };
                Ok(FragmentDetails {
                    position: spread.pos,
                    type_condition: Some(fragment.node.type_condition.node.on.node.as_str()),
                    selection_set: &fragment.node.selection_set,
                })
            }
            Selection::InlineFragment(fragment) => Ok(FragmentDetails {
                position: fragment.pos,
                type_condition: fragment
                    .node
                    .type_condition
                    .as_ref()
                    .map(|positioned| positioned.node.on.node.as_str()),
                selection_set: &fragment.node.selection_set,
            }),
        }
    }
}
