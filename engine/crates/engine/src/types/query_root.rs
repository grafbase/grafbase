use std::borrow::Cow;

use graph_entities::ResponseNodeId;
use indexmap::map::IndexMap;

use crate::{
    graph::field_into_node,
    model::{__Schema, __Type},
    parser::types::Field,
    registry,
    request::IntrospectionState,
    resolver_utils::ContainerType,
    Any, ContextField, ContextSelectionSetLegacy, LegacyOutputType, ObjectType, Positioned, ServerError, ServerResult,
    SimpleObject, Value,
};

/// Federation service
#[derive(SimpleObject)]
#[graphql(internal, name = "_Service")]
struct Service {
    sdl: Option<String>,
}

pub(crate) struct QueryRoot<T> {
    pub(crate) inner: T,
}

#[async_trait::async_trait]
impl<T: ObjectType> ContainerType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        dbg!(ctx.query_env.introspection_state);
        let introspection_enabled = match ctx.query_env.introspection_state {
            IntrospectionState::ForceEnabled => true,
            IntrospectionState::ForceDisabled => false,
            IntrospectionState::UserPreference => !ctx.schema_env.registry.disable_introspection,
        };

        if ctx.item.node.name.node == "__schema" {
            if introspection_enabled {
                let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
                let visible_types = ctx.schema_env.registry.find_visible_types(ctx);

                return LegacyOutputType::resolve(
                    &__Schema::new(&ctx.schema_env.registry, &visible_types),
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            } else {
                return Err(ServerError::new(
                    "Unauthorized for introspection.",
                    Some(ctx.item.node.name.pos),
                ));
            }
        } else if ctx.item.node.name.node == "__type" {
            if introspection_enabled {
                let (_, type_name) = ctx.param_value::<String>("name", None)?;
                let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
                let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
                return LegacyOutputType::resolve(
                    &ctx.schema_env
                        .registry
                        .types
                        .get(&type_name)
                        .filter(|_| visible_types.contains(type_name.as_str()))
                        .map(|ty| __Type::new_simple(&ctx.schema_env.registry, &visible_types, ty)),
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            } else {
                return Err(ServerError::new(
                    "Unauthorized for introspection.",
                    Some(ctx.item.node.name.pos),
                ));
            }
        }

        if ctx.schema_env.registry.enable_federation && ctx.schema_env.registry.has_entities() {
            if ctx.item.node.name.node == "_entities" {
                let (_, representations) = ctx.param_value::<Vec<Any>>("representations", None)?;
                let values = futures_util::future::try_join_all(representations.iter().map(|item| async move {
                    self.inner
                        .find_entity(ctx, &item.0)
                        .await?
                        .ok_or_else(|| ServerError::new("Entity not found.", Some(ctx.item.pos)))
                }))
                .await?;

                return Ok(Some(field_into_node(Value::List(values), ctx).await));
            } else if ctx.item.node.name.node == "_service" {
                let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
                return LegacyOutputType::resolve(
                    &Service {
                        sdl: Some(ctx.schema_env.registry.export_sdl(true)),
                    },
                    &ctx_obj,
                    ctx.item,
                )
                .await
                .map(Some);
            }
        }

        self.inner.resolve_field(ctx).await
    }

    fn is_empty() -> bool {
        false
    }

    fn collect_all_fields_native<'a>(
        &'a self,
        ctx: &ContextSelectionSetLegacy<'a>,
        fields: &mut crate::resolver_utils::Fields<'a>,
    ) -> ServerResult<()>
    where
        Self: Send + Sync,
    {
        fields.add_set_native(ctx, self)
    }

    async fn find_entity(&self, _: &ContextField<'_>, _params: &Value) -> ServerResult<Option<Value>> {
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<T: ObjectType> LegacyOutputType for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
        let root = T::create_type_info(registry);

        if !registry.disable_introspection {
            let schema_type = __Schema::create_type_info(registry);
            if let Some(registry::MetaType::Object(object)) = registry.types.get_mut(T::type_name().as_ref()) {
                object.fields.insert(
                    "__schema".to_string(),
                    registry::MetaField {
                        name: "__schema".to_string(),
                        mapped_name: None,
                        description: Some("Access the current type schema of this server.".to_string()),
                        ty: schema_type,
                        ..Default::default()
                    },
                );

                object.fields.insert(
                    "__type".to_string(),
                    registry::MetaField {
                        name: "__type".to_string(),
                        mapped_name: None,
                        description: Some("Request the type information of a single type.".to_string()),
                        args: {
                            let mut args = IndexMap::new();
                            args.insert("name".to_string(), registry::MetaInputValue::new("name", "String!"));
                            args
                        },
                        ty: "__Type".into(),
                        ..Default::default()
                    },
                );
            }
        }

        root
    }

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSetLegacy<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        todo!("node_step");
        /*
        resolve_container(
            ctx,
            ctx.registry()
                .types
                .get(Self::type_name().as_ref())
                .unwrap(),
                todo!("node_step"),
        )
        .await
        */
    }
}

impl<T: ObjectType> ObjectType for QueryRoot<T> {}
