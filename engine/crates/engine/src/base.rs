use std::{borrow::Cow, sync::Arc};

use engine_value::ConstValue;
use graph_entities::ResponseNodeId;

use crate::{
    context::ContextExt,
    parser::types::Field,
    registry::{self, InputValueType, Registry},
    ContainerType, ContextField, ContextSelectionSetLegacy, Error, InputValueError, InputValueResult, Positioned,
    Result, ServerResult, Value,
};

#[doc(hidden)]
pub trait Description {
    fn description() -> &'static str;
}

/// Represents a GraphQL input type.
pub trait LegacyInputType: Send + Sync + Sized {
    /// The raw type used for validator.
    ///
    /// Usually it is `Self`, but the wrapper type is its internal type.
    ///
    /// For example:
    ///
    /// `i32::RawValueType` is `i32`
    /// `Option<i32>::RawValueType` is `i32`.
    type RawValueType;

    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> InputValueType {
        format!("{}!", Self::type_name()).into()
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> InputValueType;

    /// Parse from `Value`. None represents undefined.
    fn parse(value: Option<Value>) -> InputValueResult<Self>;

    /// Convert to a `Value` for introspection.
    fn to_value(&self) -> Value;

    /// Get the federation fields, only for InputObject.
    #[doc(hidden)]
    fn federation_fields() -> Option<String> {
        None
    }

    /// Returns a reference to the raw value.
    fn as_raw_value(&self) -> Option<&Self::RawValueType>;
}

/// Represents a GraphQL output type.
///
/// This is mostly unused these days - a leftover from the origins of this crate.
/// Introspection does still run through this trait though, so until we replace that
/// we can't get rid of it.
#[async_trait::async_trait]
pub trait LegacyOutputType: Send + Sync {
    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> crate::registry::MetaFieldType {
        format!("{}!", Self::type_name()).into()
    }

    /// Introspection type name
    ///
    /// Is the return value of field `__typename`, the interface and union should return the current type, and the others return `Type::type_name`.
    fn introspection_type_name(&self) -> Cow<'static, str> {
        Self::type_name()
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType;

    /// Resolve an output value to `engine::Value`.
    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId>;
}

#[async_trait::async_trait]
impl<T: LegacyOutputType + ?Sized> LegacyOutputType for &T {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        T::resolve(*self, ctx, field).await
    }
}

#[async_trait::async_trait]
impl<T: LegacyOutputType + Sync, E: Into<Error> + Send + Sync + Clone> LegacyOutputType for Result<T, E> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry)
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        match self {
            Ok(value) => value.resolve(ctx, field).await,
            Err(err) => return Err(ctx.set_error_path(err.clone().into().into_server_error(field.pos))),
        }
    }
}

/// A GraphQL object.
pub trait ObjectType: ContainerType {}

#[async_trait::async_trait]
impl<T: ObjectType + ?Sized> ObjectType for &T {}

#[async_trait::async_trait]
impl<T: ObjectType + ?Sized> ObjectType for Box<T> {}

#[async_trait::async_trait]
impl<T: ObjectType + ?Sized> ObjectType for Arc<T> {}

/// A GraphQL interface.
///
/// This is mostly unused these days - a leftover from the origins of this crate.
/// Introspection does still run through this trait though, so until we replace that
/// we can't get rid of it.
pub trait LegacyInterfaceType: ContainerType {}

/// A GraphQL interface.
///
/// This is mostly unused these days - a leftover from the origins of this crate.
/// Introspection does still run through this trait though, so until we replace that
/// we can't get rid of it.
pub trait LegacyUnionType: ContainerType {}

/// A GraphQL input object.
///
/// This is mostly unused these days - a leftover from the origins of this crate.
/// Introspection does still run through this trait though, so until we replace that
/// we can't get rid of it.
pub trait LegacyInputObjectType: LegacyInputType {}

#[async_trait::async_trait]
impl<T: LegacyOutputType + ?Sized> LegacyOutputType for Box<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        T::resolve(&**self, ctx, field).await
    }
}

#[async_trait::async_trait]
impl<T: LegacyInputType> LegacyInputType for Box<T> {
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
        T::create_type_info(registry)
    }

    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value).map(Box::new).map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(self)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.as_ref().as_raw_value()
    }
}

#[async_trait::async_trait]
impl<T: LegacyOutputType + ?Sized> LegacyOutputType for Arc<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        T::resolve(&**self, ctx, field).await
    }
}

impl<T: LegacyInputType> LegacyInputType for Arc<T> {
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
        T::create_type_info(registry)
    }

    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value).map(Arc::new).map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(self)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.as_ref().as_raw_value()
    }
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait ComplexObject {
    fn fields(registry: &mut registry::Registry) -> Vec<(String, registry::MetaField)>;

    async fn resolve_field(&self, ctx: &ContextField<'_>) -> ServerResult<Option<Value>>;
}
