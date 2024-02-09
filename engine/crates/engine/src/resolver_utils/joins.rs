use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use engine_parser::{
    types::{Field, Selection, SelectionSet},
    Positioned,
};
use engine_value::{argument_set::ArgumentSet, ConstValue, Name, Value};

use crate::{
    registry::resolvers::{join::JoinResolver, ResolvedValue},
    resolver_utils::field::run_field_resolver,
    Context, ContextField, Error,
};

#[async_recursion::async_recursion]
pub async fn resolve_joined_field(
    ctx: &ContextField<'_>,
    join: &JoinResolver,
    parent_resolve_value_for_join: ResolvedValue,
) -> Result<ResolvedValue, Error> {
    let mut query_field = fake_query_ast(ctx.item, join, parent_resolve_value_for_join)?;
    let mut field_iter = join.fields.iter().peekable();
    let mut resolved_value = ResolvedValue::default();
    let mut current_type = ctx
        .schema_env()
        .registry
        .root_type(engine_parser::types::OperationType::Query);

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

// The GraphQL connector might end up batching multiple joined fields into a single operation.
// We use this counter to generate unique aliases for those to avoid clashes.
// See its use in fake_query_ast below for more details
static FIELD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fake_query_ast(
    actual_field: &Positioned<Field>,
    join: &JoinResolver,
    parent_resolve_value_for_join: ResolvedValue,
) -> Result<Positioned<Field>, Error> {
    let pos = actual_field.pos;

    let mut selection_set = actual_field.selection_set.clone();

    let mut iter = join.fields.iter().rev().peekable();
    let mut field = loop {
        let Some((name, arguments)) = iter.next() else {
            return Err(Error::new("internal error in join directive"));
        };
        let arguments = resolve_arguments(arguments, &parent_resolve_value_for_join)?;
        let field = Positioned::new(
            Field {
                name: Positioned::new(Name::new(name), pos),
                alias: None,
                arguments: arguments
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
    arguments: &ArgumentSet,
    parent_resolve_value_for_join: &ResolvedValue,
) -> Result<Vec<(String, Value)>, Error> {
    let serde_json::Value::Object(parent_object) = parent_resolve_value_for_join.data_resolved() else {
        // This might be an error but I'm going to defer reporting to the child resolver for now.
        // Saves us some work here.  Can revisit if it doesn't work very well (which is very possible)
        return Ok(vec![]);
    };

    let mut output = BTreeMap::new();

    for (name, value) in arguments.clone() {
        // Any variables this value refers to are actually fields on the last_resolver_value
        // so we need to resolve with into_const_with then convert back to a value.
        output.insert(
            name.to_string(),
            value
                .into_const_with(|variable_name| {
                    let value = parent_object.get(variable_name.as_str()).cloned().ok_or_else(|| {
                        Error::new(format!(
                            "Internal error: couldn't find {variable_name} in parent_resolver_value"
                        ))
                    })?;

                    ConstValue::from_json(value)
                        .map_err(|_| Error::new("Internal error converting intermediate values"))
                })?
                .into_value(),
        );
    }

    // At some point we'll probably want to think about forwarding arguments from the current field
    // to the joined field.  But we can handle that when the need comes up.

    Ok(output.into_iter().collect())
}
