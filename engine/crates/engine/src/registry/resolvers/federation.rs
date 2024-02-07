use engine_parser::{types::Field, Positioned};
use futures_util::future::join_all;
use serde::Deserialize;
use serde_json::Value;

use super::{ResolvedValue, ResolverContext};
use crate::{
    registry::{federation::FederationResolver, NamedType},
    resolver_utils::resolve_joined_field,
    Context, ContextExt, ContextField, Error,
};

#[derive(serde::Deserialize, serde::Serialize)]
struct Representation {
    #[serde(rename = "__typename")]
    ty: NamedType<'static>,
    #[serde(flatten)]
    data: Value,
}

pub async fn resolve_federation_entities(ctx: &ContextField<'_>) -> Result<ResolvedValue, Error> {
    let representations = Vec::<Representation>::deserialize(
        ctx.param_value_dynamic("representations", crate::resolver_utils::InputResolveMode::Default)?
            .ok_or_else(|| Error::new("The representations parameter was missing"))?,
    )
    .map_err(|error| Error::new(format!("Could not deserialize _Any: {error}")))?;

    let futures = representations.into_iter().enumerate().map(|(index, representation)| {
        let mut ctx = ctx.clone();
        ctx.path.push(index);

        async move {
            match resolve_representation(&ctx, representation).await {
                Ok(data) => data,
                Err(error) => {
                    ctx.add_error(ctx.set_error_path(error.into_server_error(ctx.item.pos)));
                    Value::Null
                }
            }
        }
    });

    Ok(ResolvedValue::new(serde_json::Value::Array(join_all(futures).await)))
}

async fn resolve_representation(ctx: &ContextField<'_>, representation: Representation) -> Result<Value, Error> {
    let entity = ctx
        .registry()
        .federation_entities
        .get(representation.ty.as_str()) // TODO: should this be keyed by NamedType?
        .ok_or_else(|| Error::new(format!("Unknown __typename in representation: {}", representation.ty)))?;

    let key_being_resolved = entity
        .find_key(&representation.data)
        .ok_or_else(|| Error::new("Could not find a matching key for the given representation"))?;

    // The ctx we're passed will have the generic Entity interface in it's type.
    // But a lot of the resolvers we're going to call into expect to get a context
    // with a concrete type.  Since we know the __typename here we can help
    // the resolvers out by passing them a more accurate ResolverContext.
    // Not sure this is the best idea, but lets find out.
    let actual_type = ctx.registry().lookup_expecting(&representation.ty).map_err(|error|
        // This probably indicates a malformed registry, lets return an error
        Error::new(format!("Internal error: {} could not be looked up in registry: {}", representation.ty, error.message))
    )?;
    let resolver_context = ResolverContext::new(ctx).with_ty(actual_type);

    let data = match key_being_resolved.resolver() {
        Some(FederationResolver::Http(resolver)) => {
            let last_resolver_value = Some(ResolvedValue::new(representation.data));
            resolver.resolve(ctx, &resolver_context, last_resolver_value).await
        }
        Some(FederationResolver::BasicType) => Ok(ResolvedValue::new(serde_json::to_value(&representation)?)),
        Some(FederationResolver::Join(join)) => {
            let mut field = ctx.item.clone();
            prune_entity_query(&mut field, representation.ty.borrow(), ctx);
            eprintln!("{field:#?}");
            let ctx = ctx.with_alternative_field(&field);

            let last_resolver_value = ResolvedValue::new(representation.data);
            resolve_joined_field(&ctx, join, last_resolver_value).await
        }
        None => {
            return Err(Error::new(format!(
                "Tried to resolve an unresolvable key for {}",
                actual_type.name()
            )))
        }
    }?;

    let mut data = data.take();

    if !data.is_null() && data["__typename"].is_null() {
        // The entities field is a union type, but most of the resolvers we're calling
        // above expect to be on a field with a concrete type - so they don't bother
        // setting __typename.  We know what type we're expecting though, so lets
        // augment the data with that
        data["__typename"] = representation.ty.to_string().into();
    }

    Ok(data)
}

/// Takes the query AST for selecting an Entity, and prunes any
/// selected fragments that aren't valid for the given type.
///
/// Subgraph queries generally look like:
///
/// ```graphql
/// _entities(representations: $reprs) {
///    ... on Foo {
///      fooFields
///    }
///    ... on Bar {
///      barFields
///    }
///    // etc
/// }
/// ```
///
/// Where `Foo` & `Bar` might be completely independent types.
///
/// Some
/// connectors (e.g. the GraphQL connector) pay attention to the
/// query AST, and those would fail if they saw selections for
/// `Bar` when they were trying to resolve `Foo`A
///
/// Pruning like this allows connectors to be unaware of this
/// complication.
fn prune_entity_query(field: &mut Positioned<Field>, ty: NamedType<'_>, context: &dyn Context) {
    let is_type_compatible: Box<dyn Fn(&str) -> bool> = match context.registry().implements.get(ty.as_str()) {
        Some(implements) => Box::new(|name: &str| implements.contains(name)),
        None => Box::new(|name: &str| name == ty.as_str()),
    };

    field.node.selection_set.node.items = std::mem::take(&mut field.node.selection_set.node.items)
        .into_iter()
        .filter(|selection| match &selection.node {
            engine_parser::types::Selection::Field(_) => {
                // Entity is a union so shouldn't have fields of its own...
                // lets assume this is the __typename and let it through
                true
            }
            engine_parser::types::Selection::FragmentSpread(spread) => {
                let Some(fragment) = context.get_fragment(spread.node.fragment_name()) else {
                    return false;
                };
                is_type_compatible(fragment.type_condition.node.on.node.as_str())
            }
            engine_parser::types::Selection::InlineFragment(fragment) => {
                let Some(condition) = &fragment.node.type_condition else {
                    // No type condition would be very weird.
                    // Not sure how to handle that so for now lets just filter out
                    return false;
                };
                is_type_compatible(condition.node.on.node.as_str())
            }
        })
        .collect();
}
