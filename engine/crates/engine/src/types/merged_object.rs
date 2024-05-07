use std::{borrow::Cow, pin::Pin};

use graph_entities::ResponseNodeId;
use indexmap::IndexMap;

use crate::{
    futures_util::Stream,
    parser::types::Field,
    registry::{LegacyRegistryExt, MetaType, ObjectType, Registry},
    CacheControl, ContainerType, ContextField, ContextSelectionSetLegacy, LegacyOutputType, Positioned, Response,
    ServerResult, SimpleObject, SubscriptionType, Value,
};

#[doc(hidden)]
pub struct MergedObject<A, B>(pub A, pub B);

#[async_trait::async_trait]
impl<A, B> ContainerType for MergedObject<A, B>
where
    A: ContainerType,
    B: ContainerType,
{
    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<ResponseNodeId>> {
        match self.0.resolve_field(ctx).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.resolve_field(ctx).await,
            Err(err) => Err(err),
        }
    }

    async fn find_entity(&self, ctx: &ContextField<'_>, params: &Value) -> ServerResult<Option<Value>> {
        match self.0.find_entity(ctx, params).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.find_entity(ctx, params).await,
            Err(err) => Err(err),
        }
    }
}

#[async_trait::async_trait]
impl<A, B> LegacyOutputType for MergedObject<A, B>
where
    A: LegacyOutputType,
    B: LegacyOutputType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", A::type_name(), B::type_name()))
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        registry.create_output_type::<Self, _>(|registry| {
            let mut fields = IndexMap::new();
            let _cc = CacheControl::default();

            if let MetaType::Object(ObjectType {
                fields: b_fields,
                // cache_control: b_cc,
                ..
            }) = registry.create_fake_output_type::<B>()
            {
                fields.extend(b_fields);
                // cc.merge(b_cc);
            }

            if let MetaType::Object(ObjectType {
                fields: a_fields,
                // cache_control: a_cc,
                ..
            }) = registry.create_fake_output_type::<A>()
            {
                fields.extend(a_fields);
                // cc.merge(a_cc);
            }

            let mut object = ObjectType::new(Self::type_name().to_string(), []);
            object.fields = fields;
            std::any::type_name::<Self>().clone_into(&mut object.rust_typename);
            object.into()
        })
    }

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSetLegacy<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl<A, B> SubscriptionType for MergedObject<A, B>
where
    A: SubscriptionType,
    B: SubscriptionType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", A::type_name(), B::type_name()))
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
        registry
            .create_subscription_type::<Self, _>(|registry| {
                let mut fields = IndexMap::new();

                if let MetaType::Object(ObjectType {
                    fields: b_fields,
                    // cache_control: b_cc,
                    ..
                }) = registry.create_fake_subscription_type::<B>()
                {
                    fields.extend(b_fields);
                    // cc.merge(b_cc);
                }

                if let MetaType::Object(ObjectType {
                    fields: a_fields,
                    // cache_control: a_cc,
                    ..
                }) = registry.create_fake_subscription_type::<A>()
                {
                    fields.extend(a_fields);
                    // cc.merge(a_cc);
                }

                let mut object = ObjectType::new(Self::type_name().to_string(), []);
                object.fields = fields;
                std::any::type_name::<Self>().clone_into(&mut object.rust_typename);
                object.into()
            })
            .into()
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a ContextField<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>> {
        unreachable!()
    }
}

#[doc(hidden)]
#[derive(SimpleObject, Default)]
#[graphql(internal, fake)]
pub struct MergedObjectTail;

impl SubscriptionType for MergedObjectTail {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("MergedSubscriptionTail")
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
        registry
            .create_subscription_type::<Self, _>(|_| {
                let mut object = ObjectType::new("MergedSubscriptionTail", []);
                std::any::type_name::<Self>().clone_into(&mut object.rust_typename);
                object.into()
            })
            .into()
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a ContextField<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>> {
        unreachable!()
    }
}
