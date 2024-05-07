pub mod builder;
mod cache_control;
mod connector_headers;
pub mod enums;
mod export_sdl;
mod export_sdl_v2;
pub mod federation;
pub mod field_set;
pub mod resolvers;
pub mod type_kinds;
mod type_names;
pub mod union_discriminator;
pub mod utils;
pub mod variables;

#[cfg(test)]
mod tests;

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    hash::Hash,
    sync::atomic::AtomicU16,
};

use engine_value::ConstValue;
use serde::{Deserialize, Serialize};

use self::type_kinds::TypeKind;
pub use self::{
    cache_control::{CacheAccessScope, CacheControl, CacheControlError, CacheInvalidationPolicy},
    export_sdl_v2::RegistrySdlExt,
    type_names::{
        ModelName, NamedType, TypeCondition, TypeReference, WrappingType, WrappingTypeIter,
    },
};
pub use crate::model::__DirectiveLocation;
use crate::{ContextExt, ContextField, Error, LegacyInputType, LegacyOutputType, SubscriptionType};

pub use registry_v1::*;
pub use registry_v2::{Deprecation, OperationLimits, ScalarParser};

/// Utility function
/// From a given type, check if the type is an Array Nullable/NonNullable.
pub fn is_array_basic_type(meta: &str) -> bool {
    let mut nested = Some(meta);

    if meta.starts_with('[') && meta.ends_with(']') {
        return true;
    }

    if meta.ends_with('!') {
        nested = nested.and_then(|x| x.strip_suffix('!'));
        return is_array_basic_type(nested.expect("Can't fail"));
    }

    false
}

pub enum CacheTag {
    Type {
        type_name: String,
    },
    List {
        type_name: String,
    },
    Field {
        type_name: String,
        field_name: String,
        value: String,
    },
}

impl From<CacheTag> for String {
    fn from(value: CacheTag) -> Self {
        value.to_string()
    }
}

impl Display for CacheTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CacheTag::Type { type_name } => f.write_str(type_name),
            CacheTag::List { type_name } => write!(f, "{type_name}#List"),
            CacheTag::Field {
                type_name,
                field_name,
                value,
            } => write!(f, "{type_name}#{field_name}:{value}"),
        }
    }
}

pub async fn check_field_cache_tag(
    ctx: &ContextField<'_>,
    resolved_field_type: &str,
    resolved_field_name: &str,
    resolved_field_value: Option<&ConstValue>,
) {
    use crate::names::{
        DELETE_PAYLOAD_RETURN_TY_SUFFIX, OUTPUT_FIELD_DELETED_ID, OUTPUT_FIELD_DELETED_IDS,
        OUTPUT_FIELD_ID,
    };
    let cache_invalidation = ctx
        .query_env
        .cache_invalidations
        .iter()
        .find(|cache_invalidation| cache_invalidation.ty == resolved_field_type);

    if let Some(cache_invalidation) = cache_invalidation {
        let mut cache_type = cache_invalidation.ty.clone();

        // This is very specific to deletions, not all queries return the @cache type ...
        // Reads, Creates and Updates do return the @cache type but Deletes do not.
        // Deletions return a `xDeletionPayload` with only a `deletedId`
        if cache_invalidation
            .ty
            .ends_with(DELETE_PAYLOAD_RETURN_TY_SUFFIX)
        {
            cache_type = cache_invalidation
                .ty
                .replace(DELETE_PAYLOAD_RETURN_TY_SUFFIX, "");
        }

        let cache_tags = match &cache_invalidation.policy {
            CacheInvalidationPolicy::Entity {
                field: target_field,
            } => {
                if target_field == resolved_field_name
                    // Deletions return a `xDeletionPayload` with only a `deletedId`
                    // If an invalidation policy is of type `entity.id`, on deletes `id` is the `deletedId`
                    || (target_field == OUTPUT_FIELD_ID && resolved_field_name == OUTPUT_FIELD_DELETED_ID)
                {
                    let Some(resolved_field_value) = resolved_field_value else {
                        tracing::warn!(
                            "missing field valued for resolved {}#{} and cache type {}",
                            resolved_field_type,
                            resolved_field_name,
                            cache_invalidation.ty,
                        );

                        return;
                    };

                    let resolved_field_value = match resolved_field_value {
                        // remove double quotes
                        ConstValue::String(quoted_string) => quoted_string.as_str().to_string(),
                        value => value.to_string(),
                    };

                    vec![CacheTag::Field {
                        type_name: cache_type,
                        field_name: target_field.to_string(),
                        value: resolved_field_value,
                    }]
                } else if target_field == OUTPUT_FIELD_ID
                    && OUTPUT_FIELD_DELETED_IDS == resolved_field_name
                {
                    let ids = Vec::<String>::deserialize(
                        resolved_field_value.unwrap_or(&ConstValue::Null).clone(),
                    )
                    .unwrap_or_default();

                    ids.into_iter()
                        .map(|value| CacheTag::Field {
                            type_name: cache_type.clone(),
                            field_name: target_field.to_string(),
                            value,
                        })
                        .collect()
                } else {
                    return;
                }
            }
            CacheInvalidationPolicy::List => vec![CacheTag::List {
                type_name: cache_type,
            }],
            CacheInvalidationPolicy::Type => vec![CacheTag::Type {
                type_name: cache_type,
            }],
        };

        ctx.response().await.add_cache_tags(cache_tags);
    }
}

/// Define an Edge for a Node.
#[derive(Debug)]
pub struct Edge<'a>(pub &'a str);

#[allow(clippy::to_string_trait_impl)]
impl<'a> ToString for Edge<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Error {
    fn unexpected_kind(name: &str, kind: TypeKind, expected: TypeKind) -> Self {
        Error::new(format!(
            "Type {name} appeared in a position where we expected a {expected:?} but it is a {kind:?}",
        ))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    derivative::Derivative,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
)]
#[repr(transparent)]
pub struct SchemaID(u16);

#[derive(Default)]
pub struct SchemaIDGenerator {
    cur: AtomicU16,
}

impl SchemaIDGenerator {
    /// Generate a new ID for a schema ID.
    pub fn new_id(&self) -> SchemaID {
        let val = self.cur.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        SchemaID(val)
    }
}

#[derive(Default)]
pub struct ConnectorIdGenerator {
    cur: AtomicU16,
}

impl ConnectorIdGenerator {
    /// Generate a new connector ID.
    pub fn new_id(&self) -> u16 {
        self.cur.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Deserialize, Serialize)]
pub struct VersionedRegistry<'a> {
    pub registry: registry_v2::Registry,
    pub deployment_id: Cow<'a, str>,
}

pub use registry_v2::CorsConfig;
pub use registry_v2::TrustedDocuments;

pub trait RegistryV2Ext {
    /// Looks up a particular type in the registry, using the default kind for the given TypeName.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    fn lookup<'a, Name>(&'a self, name: &Name) -> Result<Name::ExpectedType<'a>, Error>
    where
        Name: TypeReference,
        Name::ExpectedType<'a>: TryFrom<registry_v2::MetaType<'a>>,
        <Name::ExpectedType<'a> as TryFrom<registry_v2::MetaType<'a>>>::Error: Into<Error>;

    /// Looks up a particular type in the registry, with the expectation that it is of a particular kind.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    fn lookup_expecting<'a, Expected>(
        &'a self,
        name: &impl TypeReference,
    ) -> Result<Expected, Error>
    where
        Expected: TryFrom<registry_v2::MetaType<'a>> + 'a,
        <Expected as TryFrom<registry_v2::MetaType<'a>>>::Error: Into<Error>;
}

impl RegistryV2Ext for registry_v2::Registry {
    fn lookup<'a, Name>(&'a self, name: &Name) -> Result<Name::ExpectedType<'a>, Error>
    where
        Name: TypeReference,
        Name::ExpectedType<'a>: TryFrom<registry_v2::MetaType<'a>>,
        <Name::ExpectedType<'a> as TryFrom<registry_v2::MetaType<'a>>>::Error: Into<Error>,
    {
        name.lookup_meta(self)
            .ok_or_else(|| Error::new(format!("could not find type: {}", name.named_type())))?
            .try_into()
            .map_err(Into::into)
    }

    fn lookup_expecting<'a, Expected>(
        &'a self,
        name: &impl TypeReference,
    ) -> Result<Expected, Error>
    where
        Expected: TryFrom<registry_v2::MetaType<'a>> + 'a,
        <Expected as TryFrom<registry_v2::MetaType<'a>>>::Error: Into<Error>,
    {
        name.lookup_meta(self)
            .ok_or_else(|| Error::new(format!("could not find type: {}", name.named_type())))?
            .try_into()
            .map_err(Into::into)
    }
}

#[cfg(deleteme)]
impl Registry {
    /// Looks up a particular type in the registry, using the default kind for the given TypeName.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    pub fn lookup<'a, Name>(&'a self, name: &Name) -> Result<Name::ExpectedType<'a>, Error>
    where
        Name: TypeReference,
        Name::ExpectedType<'a>: TryFrom<&'a MetaType>,
        <Name::ExpectedType<'a> as TryFrom<&'a MetaType>>::Error: Into<Error>,
    {
        self.lookup_by_str(name.named_type().as_str())?
            .try_into()
            .map_err(Into::into)
    }

    /// Looks up a particular type in the registry, with the expectation that it is of a particular kind.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    pub fn lookup_expecting<'a, Expected>(
        &'a self,
        name: &impl TypeReference,
    ) -> Result<Expected, Error>
    where
        Expected: TryFrom<&'a MetaType> + 'a,
        <Expected as TryFrom<&'a MetaType>>::Error: Into<Error>,
    {
        self.lookup_by_str(name.named_type().as_str())?
            .try_into()
            .map_err(Into::into)
    }

    fn lookup_by_str<'a>(&'a self, name: &str) -> Result<&'a MetaType, Error> {
        self.types
            .get(name)
            .ok_or_else(|| Error::new(format!("Couldn't find a type named {name}")))
    }

    #[cfg(deleteme)]
    pub fn root_type(&self, operation_type: OperationType) -> SelectionSetTarget<'_> {
        match operation_type {
            OperationType::Query => self.query_root(),
            OperationType::Mutation => self.mutation_root(),
            OperationType::Subscription => {
                // We don't do subscriptions but may as well implement anyway.
                self.concrete_type_by_name(
                    self.subscription_type
                        .as_deref()
                        .expect("we shouldnt get here if theres no subscription type"),
                )
                .expect("the registry to be valid")
            }
        }
        .try_into()
        .expect("root type should always be a composite type")
    }
}

pub mod vectorize {
    use std::iter::FromIterator;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        ser.collect_seq(target)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(container.into_iter().collect::<T>())
    }
}

/// A collection of support functions for the legacy registry
pub trait LegacyRegistryExt {
    fn create_input_type<T: LegacyInputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> InputValueType;
    fn create_output_type<T: LegacyOutputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> MetaFieldType;
    fn create_subscription_type<
        T: SubscriptionType + ?Sized,
        F: FnOnce(&mut Registry) -> MetaType,
    >(
        &mut self,
        f: F,
    ) -> String;
    fn create_fake_output_type<T: LegacyOutputType>(&mut self) -> MetaType;
    fn create_fake_input_type<T: LegacyInputType>(&mut self) -> MetaType;
    fn create_fake_subscription_type<T: SubscriptionType>(&mut self) -> MetaType;
}

impl LegacyRegistryExt for registry_v1::Registry {
    fn create_input_type<T: LegacyInputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> InputValueType {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    fn create_output_type<T: LegacyOutputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> MetaFieldType {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    fn create_subscription_type<
        T: SubscriptionType + ?Sized,
        F: FnOnce(&mut Registry) -> MetaType,
    >(
        &mut self,
        f: F,
    ) -> String {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name().to_string()
    }

    fn create_fake_output_type<T: LegacyOutputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    fn create_fake_input_type<T: LegacyInputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    fn create_fake_subscription_type<T: SubscriptionType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }
}
