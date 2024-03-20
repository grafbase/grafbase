use std::sync::atomic::{AtomicU64, Ordering};

use engine_parser::{
    types::{Field, Selection, SelectionSet},
    Positioned,
};
use engine_value::{ConstValue, Name, Value};

use crate::{
    registry::{
        resolvers::{join::JoinResolver, ResolvedValue},
        type_kinds::SelectionSetTarget,
        MetaField,
    },
    resolver_utils::field::run_field_resolver,
    Context, ContextExt, ContextField, Error, Registry,
};

use super::{resolve_input, InputResolveMode};

#[async_recursion::async_recursion]
pub async fn resolve_joined_field(
    ctx: &ContextField<'_>,
    join: &JoinResolver,
    parent_resolve_value_for_join: ResolvedValue,
) -> Result<ResolvedValue, Error> {
    let mut query_field = fake_query_ast(ctx.item, join)?;

    let current_type = ctx
        .schema_env()
        .registry
        .root_type(engine_parser::types::OperationType::Query);

    resolve_arguments_recursively(
        ctx,
        &mut query_field.node,
        &parent_resolve_value_for_join,
        current_type,
        join,
        ctx.registry(),
    )?;

    let mut current_type = current_type;
    let mut field_iter = join.fields.iter().peekable();
    let mut resolved_value = ResolvedValue::default();

    while let Some((name, _)) = field_iter.next() {
        let meta_field = current_type.field(name).ok_or_else(|| {
            Error::new(format!(
                "Internal error: could not find joined field {}.{}",
                current_type.name(),
                &name
            ))
        })?;

        let join_context = ctx.to_join_context(&query_field, meta_field, current_type);

        resolved_value = match run_field_resolver(&join_context, resolved_value).await? {
            Some(value) => value,
            None => {
                // None indicates we should stop traversing so break out with null as our result
                resolved_value = ResolvedValue::default();
                break;
            }
        };

        if field_iter.peek().is_some() {
            current_type = ctx.registry().lookup_expecting(&meta_field.ty)?;

            let Selection::Field(inner_field) = query_field
                .node
                .selection_set
                .node
                .items
                .into_iter()
                .next()
                .expect("joined selection sets always have one field")
                .node
            else {
                unreachable!("join selection sets only have fields");
            };

            query_field = inner_field
        }
    }

    Ok(resolved_value)
}

fn resolve_arguments_recursively<'a>(
    join_field_context: &ContextField<'a>,
    mut query_field: &mut Field,
    parent_resolve_value_for_join: &ResolvedValue,
    mut current_type: SelectionSetTarget<'a>,
    join: &JoinResolver,
    registry: &'a Registry,
) -> Result<(), Error> {
    let mut field_iter = join.fields.iter().peekable();

    while let Some((name, _)) = field_iter.next() {
        let meta_field = current_type.field(name).ok_or_else(|| {
            Error::new(format!(
                "Internal error: could not find joined field {}.{}",
                current_type.name(),
                &name
            ))
        })?;

        resolve_arguments(
            join_field_context,
            &mut query_field.arguments,
            parent_resolve_value_for_join,
            meta_field,
            registry,
        );

        if field_iter.peek().is_some() {
            current_type = registry.lookup_expecting(&meta_field.ty)?;

            let Selection::Field(new_field) = &mut query_field
                .selection_set
                .node
                .items
                .iter_mut()
                .next()
                .expect("joined selection sets always have one field")
                .node
            else {
                unreachable!("join selection sets only have fields");
            };

            query_field = &mut new_field.node;
        }
    }

    Ok(())
}

// The GraphQL connector might end up batching multiple joined fields into a single operation.
// We use this counter to generate unique aliases for those to avoid clashes.
// See its use in fake_query_ast below for more details
static FIELD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fake_query_ast(actual_field: &Positioned<Field>, join: &JoinResolver) -> Result<Positioned<Field>, Error> {
    let pos = actual_field.pos;

    let mut selection_set = actual_field.selection_set.clone();

    let mut iter = join.fields.iter().rev().peekable();
    let mut field = loop {
        let Some((name, arguments)) = iter.next() else {
            return Err(Error::new("internal error in join directive"));
        };
        let field = Positioned::new(
            Field {
                name: Positioned::new(Name::new(name), pos),
                alias: None,
                arguments: arguments
                    .clone()
                    .into_iter()
                    .map(|(name, value)| (Positioned::new(Name::new(name), pos), Positioned::new(value, pos)))
                    .collect(),
                directives: vec![],
                selection_set,
            },
            pos,
        );

        if iter.peek().is_none() {
            break field;
        }

        selection_set = Positioned::new(
            SelectionSet {
                items: vec![Positioned::new(engine_parser::types::Selection::Field(field), pos)],
            },
            pos,
        );
    };

    // The GraphQL connector might end up batching multiple joined fields into a single operation.
    // In order for that to work we need to make sure we put an alias on our top level field.  This
    // means we won't clash with any other joins for the same field that are batched into the same
    // operation.
    field.node.alias = Some(Positioned::new(
        Name::new(format!("field_{}", FIELD_COUNTER.fetch_add(1, Ordering::Relaxed))),
        pos,
    ));

    Ok(field)
}

fn resolve_arguments(
    join_field_context: &ContextField<'_>,
    arguments: &mut Vec<(Positioned<Name>, Positioned<Value>)>,
    parent_resolve_value_for_join: &ResolvedValue,
    meta_field: &MetaField,
    registry: &Registry,
) {
    let serde_json::Value::Object(parent_object) = parent_resolve_value_for_join.data_resolved() else {
        // This might be an error but I'm going to defer reporting to the child resolver for now.
        // Saves us some work here.  Can revisit if it doesn't work very well (which is very possible)
        return;
    };

    for (name, value) in arguments {
        // Any variables this value refers to are either:
        // 1. Arguments on the join field
        // 2. Fields from the last_resolver_value
        //
        // We need to resolve these using into_const_with then convert back to a value.
        let const_value = value
            .node
            .clone()
            .into_const_with(|variable_name| {
                if join_field_context.field.args.contains_key(variable_name.as_str()) {
                    Ok(join_field_context
                        .input_by_name(variable_name.to_string())
                        .inspect_err(|error| {
                            log::warn!(
                                join_field_context.trace_id(),
                                "Error resolving argument on joined field: {error}"
                            )
                        })
                        .unwrap_or_default())
                } else {
                    let value = parent_object.get(variable_name.as_str()).cloned().ok_or_else(|| {
                        Error::new(format!(
                            "Internal error: couldn't find {variable_name} in parent_resolver_value"
                        ))
                    })?;

                    ConstValue::from_json(value)
                        .map_err(|_| Error::new("Internal error converting intermediate values"))
                }
            })
            .unwrap_or_default();

        value.node = meta_field
            .args
            .get(name.as_str())
            .and_then(|meta_input_value| {
                // Run things through resolve_input, which will make sure any
                // enum arguments are actually enums rather than strings
                resolve_input(
                    registry,
                    Default::default(),
                    name.as_str(),
                    meta_input_value,
                    Some(const_value.clone()),
                    InputResolveMode::Default,
                )
                .ok()
                .flatten()
            })
            .unwrap_or(const_value)
            .into_value();
    }

    // At some point we'll probably want to think about forwarding arguments from the current field
    // to the joined field.  But we can handle that when the need comes up.
}
