#![allow(deprecated)]

use std::hash::Hash;

use engine_parser::Positioned;
use grafbase_sql_ast::ast::Order;
use graphql_cursor::GraphqlCursor;
use indexmap::IndexMap;
use postgres_connector_types::{cursor::SQLCursor, database_definition::TableId};
use serde_json::Value;

use super::{ResolvedPaginationInfo, ResolvedValue, Resolver};
use crate::{
    registry::{
        resolvers::{postgres::CollectionArgs, resolved_value::SelectionData, ResolverContext},
        type_kinds::OutputType,
        MetaEnumValue, UnionDiscriminator,
    },
    ContextExt, ContextField, Error,
};

#[non_exhaustive]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Transformer {
    GraphqlField,
    /// Key based Resolver for ResolverContext
    Select {
        key: String,
    },
    /// This resolver get the PaginationData
    PaginationData,
    /// Resolves the correct values of a remote enum using the given enum name
    RemoteEnum,
    /// Resolves the __typename of a remote union type
    RemoteUnion,
    /// Convert MongoDB timestamp as number
    MongoTimestamp,
    /// A special transformer to fetch Postgres page info for the current results.
    PostgresPageInfo,
    /// Calculate cursor value for a Postgres row.
    PostgresCursor,
    /// Set Postgres selection data.
    PostgresSelectionData {
        directive_name: String,
        table_id: TableId,
    },
}

impl From<Transformer> for Resolver {
    fn from(value: Transformer) -> Self {
        Resolver::Transformer(value)
    }
}

impl Transformer {
    pub fn and_then(self, resolver: impl Into<Resolver>) -> Resolver {
        Resolver::Transformer(self).and_then(resolver)
    }

    pub fn select(key: &str) -> Self {
        Self::Select { key: key.to_string() }
    }

    pub(super) async fn resolve(
        &self,
        ctx: &ContextField<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            Transformer::GraphqlField => {
                let key = ctx
                    .item
                    .node
                    .alias
                    .as_ref()
                    .map(|Positioned { node: alias, .. }| alias.as_str())
                    .unwrap_or(ctx.field.name.as_str());
                let new_value = last_resolver_value.and_then(|x| x.get_field(key)).unwrap_or_default();

                Ok(new_value)
            }
            Transformer::Select { key } => {
                let new_value = last_resolver_value.and_then(|x| x.get_field(key)).unwrap_or_default();

                Ok(new_value)
            }
            Transformer::RemoteEnum => {
                let enum_values = ctx
                    .current_enum_values()
                    .ok_or_else(|| Error::new("Internal error resolving remote enum"))?;

                let resolved_value =
                    last_resolver_value.ok_or_else(|| Error::new("Internal error resolving remote enum"))?;

                let new_value = ResolvedValue::new(resolve_enum_value(resolved_value.data_resolved(), enum_values)?);

                Ok(new_value)
            }
            Transformer::PaginationData => {
                let pagination = last_resolver_value
                    .as_ref()
                    .and_then(|x| x.pagination.as_ref())
                    .map(ResolvedPaginationInfo::output);
                Ok(ResolvedValue::new(serde_json::to_value(pagination)?))
            }
            Transformer::RemoteUnion => {
                let discriminators = ctx
                    .current_discriminators()
                    .ok_or_else(|| Error::new("Internal error resolving remote union"))?;

                let resolved_value =
                    last_resolver_value.ok_or_else(|| Error::new("Internal error resolving remote union"))?;

                let typename = discriminators
                    .iter()
                    .find(|(_, discriminator)| discriminator.matches(resolved_value.data_resolved()))
                    .map(|(name, _)| name)
                    .ok_or_else(|| Error::new("Could not determine __typename on remote union"))?;

                let mut new_value = resolved_value.clone().take();
                if !new_value.is_object() {
                    // The OpenAPI integration has union members that are not objects.
                    //
                    // We've handled those by wrapping them in fake objects in our schema.
                    // So we're also implementing that transform here.
                    new_value = serde_json::json!({ "data": new_value });
                }

                new_value
                    .as_object_mut()
                    .unwrap()
                    .insert("__typename".into(), Value::String(typename.clone()));

                Ok(ResolvedValue::new(new_value))
            }
            Transformer::MongoTimestamp => {
                let resolved_value =
                    last_resolver_value.ok_or_else(|| Error::new("Internal error resolving mongo timestamp"))?;

                let value = match resolved_value.data_resolved() {
                    Value::Null => Value::Null,
                    Value::Number(num) => Value::Number(num.clone()),
                    Value::Object(object) => match object.get("T") {
                        Some(Value::Number(ms)) if ms.is_u64() => Value::Number(ms.clone()),
                        _ => return Err(Error::new("Cannot coerce the initial value into a valid Timestamp")),
                    },
                    _ => return Err(Error::new("Cannot coerce the initial value into a valid Timestamp")),
                };

                Ok(ResolvedValue::new(value))
            }
            Transformer::PostgresPageInfo => {
                let mut resolved_value =
                    last_resolver_value.ok_or_else(|| Error::new("Internal error resolving postgres page info"))?;

                let selection_data = resolved_value
                    .selection_data
                    .take()
                    .expect("we must have selection data set before this");

                let mut rows = match resolved_value.take() {
                    Value::Array(rows) => rows,
                    _ => return Err(Error::new("cannot calculate page info for non-array data")),
                };

                let mut has_next_page = false;
                let mut has_previous_page = false;

                if let Some(first) = selection_data.first() {
                    if (rows.len() as u64) > first {
                        has_next_page = true;
                        rows.pop();
                    }
                }

                if let Some(last) = selection_data.last() {
                    if (rows.len() as u64) > last {
                        has_previous_page = true;
                        rows.remove(0);
                    }
                }

                let start_cursor = rows.first().and_then(Value::as_object).map(|row| {
                    let cursor = SQLCursor::new(row.clone(), selection_data.order_by());
                    GraphqlCursor::try_from(cursor).unwrap()
                });

                let end_cursor = rows.last().and_then(Value::as_object).map(|row| {
                    let cursor = SQLCursor::new(row.clone(), selection_data.order_by());
                    GraphqlCursor::try_from(cursor).unwrap()
                });

                let page_info = ResolvedPaginationInfo {
                    start_cursor,
                    end_cursor,
                    has_next_page,
                    has_previous_page,
                };

                let mut new_value = ResolvedValue::new(Value::Array(rows));
                new_value.selection_data = Some(selection_data.clone());
                new_value.pagination = Some(page_info);

                Ok(new_value)
            }
            Transformer::PostgresSelectionData {
                directive_name,
                table_id,
            } => {
                let database_definition = ctx
                    .get_postgres_definition(directive_name)
                    .expect("we must have an introspected database");

                let table = database_definition.walk(*table_id);

                let root_field = ctx
                    .look_ahead()
                    .iter_selection_fields()
                    .next()
                    .expect("we always have at least one field in the query");

                let args = CollectionArgs::new(database_definition, table, &root_field)?;
                let mut selection_data = SelectionData::default();

                if let Some(first) = args.first() {
                    selection_data.set_first(first);
                }

                if let Some(last) = args.last() {
                    selection_data.set_last(last);
                }

                let explicit_order = args
                    .order_by()
                    .raw_order()
                    .map(|(column, order)| {
                        let order = order.map(|order| match order {
                            Order::DescNullsFirst => "DESC",
                            _ => "ASC",
                        });

                        (column.to_string(), order)
                    })
                    .collect();

                selection_data.set_order_by(explicit_order);

                let mut resolved_value = last_resolver_value
                    .ok_or_else(|| Error::new("Internal error resolving postgres selection data"))?
                    .clone();

                resolved_value.selection_data = Some(selection_data);

                Ok(resolved_value)
            }
            Transformer::PostgresCursor => {
                let mut resolved_value =
                    last_resolver_value.ok_or_else(|| Error::new("Internal error resolving postgres cursor"))?;

                let selection_data = resolved_value
                    .selection_data
                    .take()
                    .expect("we must set selection data for cursors to work");

                let cursor = match resolved_value.take() {
                    Value::Object(row) => {
                        let cursor = SQLCursor::new(row, selection_data.order_by());

                        GraphqlCursor::try_from(cursor)
                            .ok()
                            .and_then(|cursor| serde_json::to_value(cursor).ok())
                            .unwrap_or_default()
                    }
                    _ => Default::default(),
                };

                let mut new_value = ResolvedValue::new(cursor);
                new_value.selection_data = Some(selection_data.clone());

                Ok(new_value)
            }
        }
    }
}

impl ContextField<'_> {
    fn current_enum_values(&self) -> Option<&IndexMap<String, MetaEnumValue>> {
        match self.field_base_type() {
            OutputType::Enum(enum_type) => Some(&enum_type.enum_values),
            _ => None,
        }
    }

    fn current_discriminators(&self) -> Option<&Vec<(String, UnionDiscriminator)>> {
        match self.field_base_type() {
            OutputType::Union(union_type) => union_type.discriminators.as_ref(),
            _ => None,
        }
    }
}

/// Resolves an Enum value from a remote server where the actual value of each enum doesn't
/// match that presented by our API.
fn resolve_enum_value(remote_value: &Value, enum_values: &IndexMap<String, MetaEnumValue>) -> Result<Value, Error> {
    match remote_value {
        Value::String(remote_string) => Ok(Value::String(
            enum_values
                .values()
                .find(|meta_value| meta_value.value.as_ref() == Some(remote_string))
                .map(|meta_value| meta_value.name.clone())
                .ok_or_else(|| {
                    Error::new(format!(
                        "Expected a valid enum value from the remote API but got {remote_value}"
                    ))
                })?,
        )),
        Value::Array(array) => Ok(Value::Array(
            array
                .iter()
                .map(|value| resolve_enum_value(value, enum_values))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Null => Ok(remote_value.clone()),
        Value::Bool(_) => Err(enum_type_mismatch_error("bool")),
        Value::Number(_) => Err(enum_type_mismatch_error("number")),
        Value::Object(_) => Err(enum_type_mismatch_error("object")),
    }
}

fn enum_type_mismatch_error(received: &str) -> Error {
    Error::new(format!(
        "Received an unexpected type from the remote API.  Expected a string but received {received}"
    ))
}
