//! This file isn't generated, but every other file in this module is

/// A prelude module for all the generated modules
///
/// Anything in here will be pulled into scope for the modules
///
/// This makes the generator simpler as it doesn't need to dynamically
/// figure out how to import everything external it needs - it can just
/// `use prelude::*` and be done with it.
mod prelude {
    pub(super) use crate::{
        cache_control::CacheControl,
        common::{DirectiveLocation, IdRange, IdReader, Iter},
        field_types::{MetaFieldTypeRecord, MetaInputValueTypeRecord},
        ids::{self, StringId},
        misc_types::*,
        resolvers::Resolver,
        validators::DynValidator,
        ReadContext, RecordLookup, RegistryId,
    };
    pub(super) use common_types::auth::Operations;
    pub(super) use engine_value::ConstValue;
    pub(super) use gateway_v2_auth_config::v1::AuthConfig;
}

// The actual generated modules.
//
// If you add a new one you'll need to import it here.
pub mod directives;
pub mod enums;
pub mod field;
pub mod inputs;
pub mod interface;
pub mod metatype;
pub mod objects;
pub mod scalar;
pub mod union;
