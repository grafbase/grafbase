use std::borrow::Cow;

use graph_entities::ResponseNodeId;

use crate::{
    parser::types::Field,
    registry::{self, LegacyRegistryExt},
    resolver_utils::ContainerType,
    ContextField, ContextSelectionSetLegacy, LegacyOutputType, ObjectType, Positioned, ServerError, ServerResult,
};

/// Empty mutation
///
/// Only the parameters used to construct the Schema, representing an unconfigured mutation.
///
/// # Examples
///
/// ```rust, ignore
/// use engine::*;
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self) -> i32 {
///         // A GraphQL Object type must define one or more fields.
///         100
///     }
/// }
///
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
/// ```
#[derive(Default, Copy, Clone)]
pub struct EmptyMutation;

#[async_trait::async_trait]
impl ContainerType for EmptyMutation {
    fn is_empty() -> bool {
        true
    }

    async fn resolve_field(&self, _ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        Ok(None)
    }
}

#[async_trait::async_trait]
impl LegacyOutputType for EmptyMutation {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
        registry.create_output_type::<Self, _>(|_| {
            registry::ObjectType {
                name: "EmptyMutation".to_string(),
                description: None,
                fields: Default::default(),
                cache_control: Default::default(),
                extends: false,
                is_subscription: false,
                is_node: false,
                rust_typename: std::any::type_name::<Self>().to_owned(),
                constraints: vec![],
                external: false,
                shareable: false,
            }
            .into()
        })
    }

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSetLegacy<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        Err(ServerError::new("Schema is not configured for mutations.", None))
    }
}

impl ObjectType for EmptyMutation {}
