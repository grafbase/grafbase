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
        field_types::MetaFieldTypeRecord,
        ids::{self, StringId},
        IdReader, Iter, ReadContext, RecordLookup, RegistryId,
    };
    pub(super) use engine_id_newtypes::IdRange;
    pub(super) use registry_v2::CacheControl;
}

// The actual generated modules.
//
// If you add a new one you'll need to import it here.
pub mod field;
pub mod interface;
pub mod metatype;
pub mod objects;
