use std::{
    collections::BTreeMap,
    fmt::Write,
    sync::atomic::{AtomicU64, Ordering},
};

use engine_parser::{
    types::{Field, SelectionSet},
    Positioned,
};
use engine_value::{argument_set::ArgumentSet, ConstValue, Name, Value};

use super::{ResolvedValue, ResolverContext};
use crate::{resolver_utils, Context, ContextExt, ContextField, Error};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
pub struct JoinResolver {
    pub fields: Vec<(String, ArgumentSet)>,
}

// ArgumentSet can't be hashed so we've got a manual impl here that goes via JSON.
// Would be nice to get rid of the Hash requirement from these types
impl core::hash::Hash for JoinResolver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (name, arguments) in &self.fields {
            name.hash(state);
            serde_json::to_string(&arguments).unwrap_or_default().hash(state);
        }
    }
}

impl JoinResolver {
    pub fn new(fields: impl IntoIterator<Item = (String, Vec<(Name, Value)>)>) -> Self {
        JoinResolver {
            fields: fields
                .into_iter()
                .map(|(name, arguments)| (name, ArgumentSet::new(arguments)))
                .collect(),
        }
    }
}

impl JoinResolver {
    pub async fn resolve(
        &self,
        ctx: &ContextField<'_>,
        last_resolver_value: Option<ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let ray_id = ctx.data::<runtime::Context>()?.ray_id();
        let selection_set = self.selection_set(ray_id)?;

        let ctx_selection_set = ctx.with_joined_selection_set(&selection_set);

        let last_resolver_value = last_resolver_value.unwrap_or_default();

        let id = resolver_utils::resolve_root_container(&ctx_selection_set).await?;

        // The above will have written straight into the response which is unfortuantely
        // not what we need, we need a ResolvedValue.
        // So take stuff back out of the response and return it.
        //
        // ofc this isn't actually good enough is it.
        // because the query we're building above isn't the _full_ query.
        // FUCKERY DOO
        let value = ctx
            .response()
            .await
            .take_node_into_compact_value(id)
            .map(|compact_value| ResolvedValue::new(compact_value.into()));

        let meta_field = root_type.field(&self.field_name).ok_or_else(|| {
            Error::new(format!(
                "Internal error: could not find joined field {}",
                &self.field_name
            ))
        })?;

        // TODO: OK so this probably needs to create a full on selection set,
        // pump that into a ContextSelectionSet,
        // then call resolver_utils::resolve_container
        // URGH, who can be arsed

        let fake_query_field = self.field_for_join(ctx.item, last_resolver_value)?;
        let join_context = ctx.to_join_context(&fake_query_field, meta_field, root_type);
        let resolver_context = ResolverContext::new(&join_context);

        meta_field
            .resolver
            .resolve(&join_context, &resolver_context, None)
            .await
    }
}

static FIELD_COUNTER: AtomicU64 = AtomicU64::new(0);

impl JoinResolver {
    fn selection_set(&self, ray_id: &str) -> Result<Positioned<SelectionSet>, Error> {
        self.selection_set_str()
            .map_err(|_| ())
            .and_then(|selection_set| {
                engine_parser::parse_selection_set(&selection_set).map_err(|error| {
                    log::error!(ray_id, "Error parsing Join selection set: {error}");
                })
            })
            .map_err(|_| Error::new("internal error performing join"))
    }

    fn selection_set_str(&self) -> Result<String, std::fmt::Error> {
        let mut output = String::with_capacity(
            // No way this will be accurate, but seems better than starting empty.
            self.fields.len() * 10,
        );

        let mut field_iter = self.fields.iter().peekable();

        while let Some((name, arguments)) = field_iter.next() {
            write!(&mut output, "{name}")?;
            if !arguments.is_empty() {
                write!(&mut output, "(")?;
                for name in arguments.iter_names() {
                    write!(&mut output, "${name} ")?;
                }
                write!(&mut output, ")")?;
            }
            if field_iter.peek().is_some() {
                write!(&mut output, "{{ ")?;
            }
        }

        for _ in 1..self.fields.len() {
            write!(&mut output, " }}")?;
        }

        Ok(output)
    }

    fn field_for_join(
        &self,
        actual_field: &Positioned<Field>,
        last_resolver_value: ResolvedValue,
    ) -> Result<Positioned<Field>, Error> {
        let arguments = self.resolve_arguments(last_resolver_value)?;

        let Positioned { pos, node: field } = actual_field;

        Ok(Positioned::new(
            Field {
                alias: Some(Positioned::new(
                    Name::new(format!("field_{}", FIELD_COUNTER.fetch_add(1, Ordering::Relaxed))),
                    *pos,
                )),
                name: Positioned::new(Name::new(&self.field_name), *pos),
                arguments: arguments
                    .into_iter()
                    .map(|(name, value)| (Positioned::new(Name::new(name), *pos), Positioned::new(value, *pos)))
                    .collect(),
                directives: field.directives.clone(),
                selection_set: field.selection_set.clone(),
            },
            *pos,
        ))
    }

    fn resolve_arguments(&self, last_resolver_value: ResolvedValue) -> Result<Vec<(String, Value)>, Error> {
        let serde_json::Value::Object(parent_object) = last_resolver_value.data_resolved() else {
            // This might be an error but I'm going to defer reporting to the child resolver for now.
            // Saves us some work here.  Can revisit if it doesn't work very well (which is very possible)
            return Ok(vec![]);
        };

        let mut arguments = BTreeMap::new();

        for (name, value) in self.arguments.clone() {
            // Any variables this value refers to are actually fields on the last_resolver_value
            // so we need to resolve with into_const_with then convert back to a value.
            arguments.insert(
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
        // to the joined field.  But we can handle that when the need comes up, this PR is already too
        // big

        Ok(arguments.into_iter().collect())
    }
}
