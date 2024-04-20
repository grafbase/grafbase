use std::{collections::HashMap, future::Future, ops::DerefMut, pin::Pin, sync::Arc};

use engine_parser::Positioned;
use futures_util::FutureExt;
use graph_entities::{CompactValue, QueryResponse, ResponseContainer, ResponseNodeId};
use internment::ArcIntern;

use super::{field::resolve_field, fragment::FragmentDetails};
use crate::{
    deferred::DeferredWorkload,
    extensions::ResolveInfo,
    parser::types::Selection,
    registry::{
        resolvers::ResolvedValue,
        type_kinds::{OutputType, SelectionSetTarget},
    },
    Context, ContextExt, ContextField, ContextSelectionSet, ContextSelectionSetLegacy, Error, LegacyOutputType, Name,
    ServerError, ServerResult, Value,
};

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

    /// Resolves a field value and outputs it as a json value `engine::Value`.
    ///
    /// If the field was not found returns None.
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>>;

    /// Collect all the fields of the container that are queried in the selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must call it on their
    /// internal type.
    fn collect_all_fields_native<'a>(
        &'a self,
        ctx: &ContextSelectionSetLegacy<'a>,
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
    async fn find_entity(&self, _: &ContextField<'_>, _params: &Value) -> ServerResult<Option<Value>> {
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for &T {
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(*self, ctx).await
    }

    async fn find_entity(&self, ctx: &ContextField<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(*self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Arc<T> {
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &ContextField<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType + ?Sized> ContainerType for Box<T> {
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &ContextField<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[async_trait::async_trait]
impl<T: ContainerType, E: Into<Error> + Send + Sync + Clone> ContainerType for Result<T, E> {
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        match self {
            Ok(value) => T::resolve_field(value, ctx).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }

    async fn find_entity(&self, ctx: &ContextField<'_>, params: &Value) -> ServerResult<Option<Value>> {
        match self {
            Ok(value) => T::find_entity(value, ctx, params).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_root_container<'a>(ctx: &ContextSelectionSet<'a>) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, None).await
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_root_container_serial<'a>(ctx: &ContextSelectionSet<'a>) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, false, None).await
}

pub async fn resolve_deferred_container<'a>(
    ctx: &ContextSelectionSet<'a>,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, parent_resolver_value).await
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_container_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSetLegacy<'a>,
    root: &'a T,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner_native(ctx, root, true).await
}

pub(super) async fn resolve_container<'a>(
    ctx: &ContextSelectionSet<'a>,
    parent_resolver_value: ResolvedValue,
) -> ServerResult<ResponseNodeId> {
    resolve_container_inner(ctx, true, Some(parent_resolver_value)).await
}

async fn resolve_container_inner<'a>(
    ctx: &ContextSelectionSet<'a>,
    parallel: bool,
    parent_resolver_value: Option<ResolvedValue>,
) -> ServerResult<ResponseNodeId> {
    tracing::trace!("Where: {}", ctx.ty.name());

    let mut fields = FieldExecutionSet(Vec::new());
    fields.add_selection_set(ctx, parent_resolver_value)?;

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

    let results = results
        .into_iter()
        .map(|((alias, name), node)| (alias.unwrap_or(name), node))
        .collect::<Vec<_>>();

    let results = merge_duplicate_fields(ctx.response().await.deref_mut(), results);

    let mut container = ResponseContainer::new_container();
    for (name, value) in results {
        container.insert(name.as_str(), value);
    }
    Ok(ctx.response().await.insert_node(container))
}

fn merge_duplicate_fields(
    response: &mut QueryResponse,
    fields: Vec<(Name, ResponseNodeId)>,
) -> Vec<(Name, ResponseNodeId)> {
    let mut dedup_map = HashMap::with_capacity(fields.len());
    let mut results = Vec::with_capacity(fields.len());

    for (name, node_id) in fields {
        if let Some(existing_id) = dedup_map.get(&name) {
            response.merge(node_id, *existing_id);
            response.delete_node(node_id).ok();
            continue;
        }

        dedup_map.insert(name.clone(), node_id);
        results.push((name, node_id));
    }

    results
}

async fn resolve_container_inner_native<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSetLegacy<'a>,
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

    let container = ResponseContainer::with_children(
        res.into_iter()
            .map(|(name, value)| (ArcIntern::new(name.to_string()), value)),
    );

    Ok(ctx.response().await.insert_node(container))
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

async fn response_id_unwrap_or_null(ctx: &ContextField<'_>, opt_id: Option<ResponseNodeId>) -> ResponseNodeId {
    if let Some(id) = opt_id {
        id
    } else {
        ctx.response().await.insert_node(CompactValue::Null)
    }
}

impl<'a> FieldExecutionSet<'a> {
    /// Creates futures for all the fields in the given selection set and adds them
    /// to the field execution set
    pub fn add_selection_set(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        parent_resolver_value: Option<ResolvedValue>,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            let parent_resolver_value = parent_resolver_value.clone();
            match &selection.node {
                Selection::Field(field) => {
                    self.add_field(ctx, field, parent_resolver_value);
                }
                Selection::FragmentSpread(_) | Selection::InlineFragment(_) => {
                    self.add_spread(
                        ctx,
                        FragmentDetails::from_fragment_selection(ctx, &selection.node)?,
                        parent_resolver_value,
                    );
                }
            }
        }
        Ok(())
    }

    // Adds a field to the FieldsGraph
    fn add_field(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        field: &'a Positioned<engine_parser::types::Field>,
        parent_resolver_value: Option<ResolvedValue>,
    ) {
        if field.node.name.node == "__typename" {
            let ctx = ctx.clone();
            let field_name = field.node.name.node.clone();
            let alias = field.node.alias.clone().map(|x| x.node);

            self.0.push(Box::pin({
                async move {
                    let node = CompactValue::String(resolve_typename(ctx.ty, parent_resolver_value.as_ref()).await);
                    Ok(FieldExecutionOutput::Field(
                        (alias, field_name),
                        ctx.response().await.insert_node(node),
                    ))
                }
            }));
            return;
        }
        self.0.push(Box::pin({
            let ctx = ctx.clone();
            async move {
                let ctx_field = ctx.with_field(field);
                let field_name = ctx_field.item.node.name.node.clone();
                let alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                let extensions = &ctx.query_env.extensions;

                let resolve_fut = resolve_field(&ctx_field, parent_resolver_value);

                if extensions.is_empty() && field.node.directives.is_empty() {
                    // If we've no extensions or directives, just return the data
                    return Ok(FieldExecutionOutput::Field((alias, field_name), resolve_fut.await?));
                }

                // Convert resolve_fut to a Result<Option<_>> for some reason
                let resolve_fut = resolve_fut.map(|result| result.map(Some));

                let type_name = ctx.ty.name();
                tracing::trace!(
                    "Resolving {field} on {type_name}",
                    field = field.node.name.node.as_str()
                );

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
                    path: ctx_field.path.clone(),
                    parent_type: type_name,
                    return_type: match meta_field.map(|field| &field.ty) {
                        Some(ty) => ty,
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
                    auth: meta_field.and_then(|f| f.auth.as_deref()),
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
        fragment_details: FragmentDetails<'a>,
        parent_resolver_value: Option<ResolvedValue>,
    ) {
        let ctx = ctx.clone();
        self.0.push(Box::pin({
            async move {
                let registry = ctx.registry();
                let typename = resolve_typename(ctx.ty, parent_resolver_value.as_ref()).await;
                if !fragment_details.type_condition_matches(&ctx, &typename) {
                    return Ok(FieldExecutionOutput::MultipleFields(vec![]));
                }

                let subtype = registry
                    .types
                    .get(&typename)
                    .ok_or_else(|| ServerError::new(format!(r#"Found an unknown typename: "{typename}"."#,), None))?
                    .try_into()
                    .map_err(|_| ServerError::new(format!("Tried to spread on a leaf type: {typename}"), None))?;

                if fragment_details.should_defer(&ctx)
                    && defer_fragment(&ctx, &fragment_details, subtype, &parent_resolver_value).is_ok()
                {
                    // If we succesfully deferred, then return no fields.  Otherwise fall through to handling the
                    // spread as if it didn't even have `@defer` on it.
                    return Ok(FieldExecutionOutput::MultipleFields(vec![]));
                }

                let mut subfields = FieldExecutionSet(Vec::new());
                subfields.add_selection_set(
                    &ctx.with_selection_set(fragment_details.selection_set, subtype),
                    parent_resolver_value,
                )?;

                Ok(FieldExecutionOutput::MultipleFields(
                    futures_util::future::try_join_all(subfields.0).await?.flatten(),
                ))
            }
        }));
    }
}

/// Defers a fragment for later execution.
///
/// This shouldn't generally fail, but if it does we should just handle the fragment as if it
/// doesn't have defer on it.  This avoids returning internal errors for these cases.
fn defer_fragment(
    ctx: &ContextSelectionSet<'_>,
    fragment_details: &FragmentDetails<'_>,
    target_ty: SelectionSetTarget,
    parent_resolver_value: &Option<ResolvedValue>,
) -> Result<(), ()> {
    let deferred_sender = ctx.deferred_workloads().ok_or(())?;
    let Some(directive) = fragment_details.defer.as_ref() else {
        return Err(());
    };

    let workload = DeferredWorkload::new(
        directive.label.clone(),
        fragment_details.selection_set.clone(),
        ctx.path.clone(),
        target_ty.name().to_string().into(),
        parent_resolver_value.clone(),
    );

    // Sending _shouldn't_ fail, but if it does lets return an Err so we treat the spread
    // as if it didn't even have `@defer` on it (rather than immediately
    // erroring out on what is almost certainly an internal error)
    deferred_sender.send(workload).map_err(|_| ())?;
    Ok(())
}

async fn resolve_typename<'a>(root: SelectionSetTarget<'a>, parent_resolver_value: Option<&ResolvedValue>) -> String {
    match root {
        SelectionSetTarget::Union(_) | SelectionSetTarget::Interface(_) => {
            if let Some(typename) = resolve_remote_typename(parent_resolver_value).await {
                return typename;
            }
        }
        SelectionSetTarget::Object(_) => {}
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
        ctx: &ContextSelectionSetLegacy<'a>,
        root: &'a T,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        // Get the typename
                        let field_name = field.node.response_key().node.clone();
                        let typename = root.introspection_type_name().into_owned();

                        let ctx = ctx.clone();
                        self.0.push(Box::pin(async move {
                            let node = CompactValue::String(typename);
                            Ok((field_name, ctx.response().await.insert_node(node)))
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
                                    path: ctx_field.path.clone(),
                                    parent_type: &type_name,
                                    return_type: match meta_field.map(|field| &field.ty) {
                                        Some(ty) => ty,
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
                                    auth: meta_field.and_then(|f| f.auth.as_deref()),
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

                    // Note: this is the "native" resolution mechanism that's only used for
                    // introspection.  We're not going to support defer or stream here.

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
                    let new_target = type_condition
                        .and_then(|name| {
                            ctx.registry()
                                .types
                                .get(name)
                                .and_then(|ty| OutputType::try_from(ty).ok())
                        })
                        .unwrap_or(ctx.ty);

                    if applies_concrete_object {
                        root.collect_all_fields_native(&ctx.with_selection_set(selection_set, new_target), self)?;
                    } else if type_condition.map_or(true, |condition| T::type_name() == condition) {
                        // The fragment applies to an interface type.
                        self.add_set_native(&ctx.with_selection_set(selection_set, new_target), root)?;
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
