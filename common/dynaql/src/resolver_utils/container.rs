use dynaql_parser::Positioned;
use futures_util::FutureExt;
use graph_entities::{
    QueryResponseNode, ResponseContainer, ResponseNodeId, ResponseNodeRelation, ResponsePrimitive,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::dynamic::DynamicSelectionSetContext;
use crate::extensions::ResolveInfo;
use crate::graph::field_into_node;
use crate::parser::types::Selection;

use crate::registry::resolvers::{ResolvedContainer, ResolvedValue};
use crate::{
    relations_edges, Context, ContextBase, ContextSelectionSet, Error, Name, OutputType,
    ServerError, ServerResult, Value,
};

use super::{resolve_field, resolve_introspection_field};

/// Represents a GraphQL container object.
///
/// This helper trait allows the type to call `resolve_container` on itself in its
/// `OutputType::resolve` implementation.
#[async_trait::async_trait]
pub trait ContainerType: OutputType {
    /// This function returns true of type `EmptyMutation` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value `dynaql::Value`.
    ///
    /// If the field was not found returns None.
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>>;

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

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(*self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(*self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Arc<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Box<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType, E: Into<Error> + Send + Sync + Clone> ContainerType for Result<T, E> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
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
pub async fn resolve_container<'a>(
    ctx: &DynamicSelectionSetContext<'a>,
    maybe_parent_resolved_container: Option<ResolvedContainer<'a>>,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, maybe_parent_resolved_container).await
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_container_serial<'a>(
    ctx: &DynamicSelectionSetContext<'a>,
    maybe_parent_resolved_container: Option<ResolvedContainer<'a>>,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, false, maybe_parent_resolved_container).await
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_container_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    resolve_container_inner_native(ctx, root, true).await
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_container_serial_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    resolve_container_inner_native(ctx, root, false).await
}

fn insert_value(target: &mut IndexMap<Name, Value>, name: Name, value: Value) {
    if let Some(prev_value) = target.get_mut(&name) {
        if let Value::Object(target_map) = prev_value {
            if let Value::Object(obj) = value {
                for (key, value) in obj {
                    insert_value(target_map, key, value);
                }
            }
        } else if let Value::List(target_list) = prev_value {
            if let Value::List(list) = value {
                for (idx, value) in list.into_iter().enumerate() {
                    if let Some(Value::Object(target_map)) = target_list.get_mut(idx) {
                        if let Value::Object(obj) = value {
                            for (key, value) in obj {
                                insert_value(target_map, key, value);
                            }
                        }
                    }
                }
            }
        }
    } else {
        target.insert(name, value);
    }
}

async fn resolve_container_inner<'a>(
    ctx: &DynamicSelectionSetContext<'a>,
    parallel: bool,
    maybe_parent_resolved_container: Option<ResolvedContainer<'a>>,
) -> ServerResult<ResponseNodeId> {
    // Forcing the lifetime of fields to be inferior to maybe_parent_resolved_container
    let res = {
        let mut fields = FieldsGraph::new();
        fields.add_set(
            maybe_parent_resolved_container.as_ref().map(|rc| &rc.value),
            ctx,
        )?;
        if parallel {
            futures_util::future::try_join_all(fields.0).await?
        } else {
            let mut results = Vec::with_capacity(fields.0.len());
            for field in fields.0 {
                results.push(field.await?);
            }
            results
        }
    };

    let relations = relations_edges(ctx);
    #[cfg(feature = "tracing_worker")]
    {
        logworker::info!("", "Relations for {} {:?}", ctx.root_type.name(), relations);
    }

    let mut container = maybe_parent_resolved_container
        .and_then(|rc| rc.maybe_node_id)
        .map(ResponseContainer::new_node)
        .unwrap_or_else(ResponseContainer::new_container);

    for ResponseFieldNode {
        alias,
        name,
        node_id,
    } in res
    {
        let name = name.to_string();
        let alias = alias.map(|x| x.to_string().into());

        if let Some(relation) = relations.get(&name) {
            container.insert(
                ResponseNodeRelation::relation(
                    name,
                    relation.name.clone(),
                    relation.relation.0.clone(),
                    relation.relation.1.clone(),
                ),
                node_id,
            );
        } else {
            container.insert(
                ResponseNodeRelation::NotARelation {
                    field: name.into(),
                    response_key: alias,
                },
                node_id,
            );
        }
    }
    Ok(ctx
        .response_graph
        .write()
        .await
        .new_node_unchecked(QueryResponseNode::from(container)))
}

async fn resolve_container_inner_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
    parallel: bool,
) -> ServerResult<Value> {
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

    let mut map = IndexMap::new();
    let response = ctx.response_graph.read().await;
    for (name, value) in res {
        if let Some(node) = response.get_node(&value) {
            let const_value = response.transform_node_to_const_value(node).map_err(|_| {
                ctx.set_error_path(ServerError::new("JSON serialization failure.", None))
            })?;

            insert_value(&mut map, name, const_value);
        }
    }
    Ok(Value::Object(map))
}

type BoxFieldGraphFuture<'f> =
    Pin<Box<dyn Future<Output = ServerResult<ResponseFieldNode>> + 'f + Send>>;
/// A set of fields on an container that are being selected.
pub struct FieldsGraph<'f>(Vec<BoxFieldGraphFuture<'f>>);

pub struct ResponseFieldNode {
    pub alias: Option<Name>,
    pub name: Name,
    pub node_id: ResponseNodeId,
}

async fn response_id_unwrap_or_null(
    ctx: &Context<'_>,
    opt_id: Option<ResponseNodeId>,
) -> ResponseNodeId {
    if let Some(id) = opt_id {
        id
    } else {
        ctx.response_graph
            .write()
            .await
            .new_node_unchecked(QueryResponseNode::Primitive(ResponsePrimitive::new(
                Value::Null,
            )))
    }
}

impl<'f> FieldsGraph<'f> {
    fn new() -> Self {
        FieldsGraph(Vec::new())
    }

    /// Add another set of fields to this set of fields using the given container.
    pub fn add_set<'ctx>(
        &mut self,
        maybe_parent_resolved_container_value: Option<&'ctx ResolvedValue>,
        ctx: &DynamicSelectionSetContext<'ctx>,
    ) -> ServerResult<()>
    where
        'ctx: 'f,
    {
        let registry = ctx.registry();
        for selection in &ctx.item.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        // Get the typename
                        let ctx_field = ctx.with_field(field);
                        let name = ctx_field.item.node.name.node.clone();
                        let alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                        let typename = registry.introspection_type_name(ctx.root_type).to_owned();

                        self.0.push(Box::pin(async move {
                            let node = QueryResponseNode::from(ResponsePrimitive::new(
                                Value::String(typename),
                            ));
                            Ok(ResponseFieldNode {
                                alias,
                                name,
                                node_id: ctx_field
                                    .response_graph
                                    .write()
                                    .await
                                    .new_node_unchecked(node),
                            })
                        }));
                        continue;
                    }

                    let resolve_fut = Box::pin({
                        let ctx = ctx.clone();
                        async move {
                            let maybe_meta = ctx.root_type.field_by_name(&field.node.name.node);
                            let ctx_field = ctx.with_field(field);
                            let name = ctx_field.item.node.name.node.clone();
                            let alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                            let node_id = match maybe_meta {
                                Some(meta) if !name.starts_with("__") => {
                                    let extensions = &ctx_field.query_env.extensions;
                                    let args_values: Vec<(Positioned<Name>, Option<Value>)> =
                                        ctx_field
                                            .item
                                            .node
                                            .arguments
                                            .clone()
                                            .into_iter()
                                            .map(|(key, val)| {
                                                (key, ctx_field.resolve_input_value(val).ok())
                                            })
                                            .collect();
                                    let resolve_info = ResolveInfo {
                                        path_node: ctx_field.path_node.as_ref().unwrap(),
                                        parent_type: &ctx.root_type.name(),
                                        return_type: &meta.ty,
                                        name: name.as_str(),
                                        alias: alias.as_ref().map(Name::as_str),
                                        required_operation: meta.required_operation,
                                        auth: meta.auth.as_ref(),
                                        input_values: args_values,
                                    };
                                    let ctx_field = ctx.dynamic_with_field(field, meta);
                                    let resolve_fut = resolve_field(
                                        &ctx_field,
                                        maybe_parent_resolved_container_value,
                                    );

                                    if field.node.directives.is_empty() {
                                        futures_util::pin_mut!(resolve_fut);
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions
                                                .resolve(resolve_info, &mut resolve_fut)
                                                .await?,
                                        )
                                        .await
                                    } else {
                                        let mut resolve_fut = resolve_fut.boxed();
                                        for directive in &field.node.directives {
                                            if let Some(directive_factory) = ctx_field
                                                .schema_env
                                                .custom_directives
                                                .get(directive.node.name.node.as_str())
                                            {
                                                let ctx_directive = ContextBase {
                                                    path_node: ctx_field.path_node,
                                                    item: directive,
                                                    schema_env: ctx_field.schema_env,
                                                    query_env: ctx_field.query_env,
                                                    response_graph: ctx_field
                                                        .response_graph
                                                        .clone(),
                                                };
                                                let directive_instance = directive_factory
                                                    .create(&ctx_directive, &directive.node)?;
                                                resolve_fut = Box::pin({
                                                    let ctx_field = ctx_field.clone();
                                                    async move {
                                                        directive_instance
                                                            .resolve_field(
                                                                &ctx_field,
                                                                &mut resolve_fut,
                                                            )
                                                            .await
                                                    }
                                                });
                                            }
                                        }
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions
                                                .resolve(resolve_info, &mut resolve_fut)
                                                .await?,
                                        )
                                        .await
                                    }
                                }
                                _ => {
                                    response_id_unwrap_or_null(
                                        &ctx_field,
                                        resolve_introspection_field(&ctx_field, ctx.root_type)
                                            .await?,
                                    )
                                    .await
                                }
                            };

                            Ok(ResponseFieldNode {
                                alias,
                                name,
                                node_id,
                            })
                        }
                    });

                    self.0.push(resolve_fut);
                }
                selection => {
                    let (type_condition, selection_set) = match selection {
                        Selection::Field(_) => unreachable!(),
                        Selection::FragmentSpread(spread) => {
                            let fragment =
                                ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                            let fragment = match fragment {
                                Some(fragment) => fragment,
                                None => {
                                    return Err(ServerError::new(
                                        format!(
                                            r#"Unknown fragment "{}"."#,
                                            spread.node.fragment_name.node
                                        ),
                                        Some(spread.pos),
                                    ));
                                }
                            };
                            (
                                Some(&fragment.node.type_condition),
                                &fragment.node.selection_set,
                            )
                        }
                        Selection::InlineFragment(fragment) => (
                            fragment.node.type_condition.as_ref(),
                            &fragment.node.selection_set,
                        ),
                    };
                    let type_condition =
                        type_condition.map(|condition| condition.node.on.node.as_str());

                    let introspection_type_name = registry.introspection_type_name(ctx.root_type);

                    let applies_concrete_object = type_condition.map_or(false, |condition| {
                        introspection_type_name == condition
                            || ctx
                                .schema_env
                                .registry
                                .implements
                                .get(introspection_type_name)
                                .map_or(false, |interfaces| interfaces.contains(condition))
                    });
                    if applies_concrete_object
                        || type_condition
                            .map_or(true, |condition| ctx.root_type.name() == condition)
                    {
                        self.add_set(
                            maybe_parent_resolved_container_value,
                            &ctx.dynamic_with_selection_set(selection_set),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

type BoxFieldFuture<'a> =
    Pin<Box<dyn Future<Output = ServerResult<(Name, ResponseNodeId)>> + 'a + Send>>;
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
                        let ctx_field = ctx.with_field(field);
                        let field_name = ctx_field.item.node.response_key().node.clone();
                        let typename = root.introspection_type_name().into_owned();

                        self.0.push(Box::pin(async move {
                            let node = QueryResponseNode::from(ResponsePrimitive::new(
                                Value::String(typename),
                            ));
                            Ok((
                                field_name,
                                ctx_field
                                    .response_graph
                                    .write()
                                    .await
                                    .new_node_unchecked(node),
                            ))
                        }));
                        continue;
                    }

                    let resolve_fut = Box::pin({
                        let ctx = ctx.clone();
                        async move {
                            let ctx_field = ctx.with_field(field);
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
                                    field_into_node(
                                        root.resolve_field(&ctx_field).await?.unwrap_or_default(),
                                        &ctx_field,
                                    )
                                    .await,
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
                                                format!(
                                                    r#"Cannot query field "{}" on type "{}"."#,
                                                    field_name, type_name
                                                ),
                                                Some(ctx_field.item.pos),
                                            ));
                                        }
                                    },
                                    name: field.node.name.node.as_str(),
                                    alias: field
                                        .node
                                        .alias
                                        .as_ref()
                                        .map(|alias| alias.node.as_str()),
                                    required_operation: meta_field
                                        .and_then(|f| f.required_operation),
                                    auth: meta_field.and_then(|f| f.auth.as_ref()),
                                    input_values: args_values,
                                };

                                let resolve_fut = async {
                                    let a = field_into_node(
                                        root.resolve_field(&ctx_field).await?.unwrap_or_default(),
                                        &ctx_field,
                                    )
                                    .await;
                                    Ok(Some(a))
                                };

                                if field.node.directives.is_empty() {
                                    futures_util::pin_mut!(resolve_fut);
                                    Ok((
                                        field_name,
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions
                                                .resolve(resolve_info, &mut resolve_fut)
                                                .await?,
                                        )
                                        .await,
                                    ))
                                } else {
                                    let mut resolve_fut = resolve_fut.boxed();

                                    for directive in &field.node.directives {
                                        if let Some(directive_factory) = ctx
                                            .schema_env
                                            .custom_directives
                                            .get(directive.node.name.node.as_str())
                                        {
                                            let ctx_directive = ContextBase {
                                                path_node: ctx_field.path_node,
                                                item: directive,
                                                schema_env: ctx_field.schema_env,
                                                query_env: ctx_field.query_env,
                                                response_graph: ctx_field.response_graph.clone(),
                                            };
                                            let directive_instance = directive_factory
                                                .create(&ctx_directive, &directive.node)?;
                                            resolve_fut = Box::pin({
                                                let ctx_field = ctx_field.clone();
                                                async move {
                                                    directive_instance
                                                        .resolve_field(&ctx_field, &mut resolve_fut)
                                                        .await
                                                }
                                            });
                                        }
                                    }

                                    Ok((
                                        field_name,
                                        response_id_unwrap_or_null(
                                            &ctx_field,
                                            extensions
                                                .resolve(resolve_info, &mut resolve_fut)
                                                .await?,
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
                    let (type_condition, selection_set) = match selection {
                        Selection::Field(_) => unreachable!(),
                        Selection::FragmentSpread(spread) => {
                            let fragment =
                                ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                            let fragment = match fragment {
                                Some(fragment) => fragment,
                                None => {
                                    return Err(ServerError::new(
                                        format!(
                                            r#"Unknown fragment "{}"."#,
                                            spread.node.fragment_name.node
                                        ),
                                        Some(spread.pos),
                                    ));
                                }
                            };
                            (
                                Some(&fragment.node.type_condition),
                                &fragment.node.selection_set,
                            )
                        }
                        Selection::InlineFragment(fragment) => (
                            fragment.node.type_condition.as_ref(),
                            &fragment.node.selection_set,
                        ),
                    };
                    let type_condition =
                        type_condition.map(|condition| condition.node.on.node.as_str());

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
                        root.collect_all_fields_native(
                            &ctx.with_selection_set(selection_set),
                            self,
                        )?;
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
