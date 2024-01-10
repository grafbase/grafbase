use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use engine_parser::{types::Field, Positioned};
use engine_value::{argument_set::ArgumentSet, ConstValue, Name, Value};

use super::{ResolvedValue, ResolverContext};
use crate::{Context, ContextField, Error};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
pub struct JoinResolver {
    pub field_name: String,
    pub arguments: ArgumentSet,
}

// ArgumentSet can't be hashed so we've got a manual impl here that goes via JSON.
// Would be nice to get rid of the Hash requirement from these types
impl core::hash::Hash for JoinResolver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.field_name.hash(state);
        serde_json::to_string(&self.arguments).unwrap_or_default().hash(state);
    }
}

impl JoinResolver {
    pub fn new(field_name: String, arguments: Vec<(Name, Value)>) -> Self {
        JoinResolver {
            field_name,
            arguments: ArgumentSet::new(arguments),
        }
    }
}

impl JoinResolver {
    pub async fn resolve(
        &self,
        ctx: &ContextField<'_>,
        last_resolver_value: Option<ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let root_type = ctx
            .schema_env()
            .registry
            .root_type(engine_parser::types::OperationType::Query);

        let last_resolver_value = last_resolver_value.unwrap_or_default();
        let meta_field = root_type.field(&self.field_name).ok_or_else(|| {
            Error::new(format!(
                "Internal error: could not find joined field {}",
                &self.field_name
            ))
        })?;

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
