pub(crate) use crate::ids::MAX_ID;
/// A prelude module for all the generated modules
///
/// Anything in here will be pulled into scope for the modules
///
/// This makes the generator simpler as it doesn't need to dynamically
/// figure out how to import everything external it needs - it can just
/// `use prelude::*` and be done with it.
pub(crate) use crate::Schema; // Having the Schema here guarantees the prelude::* is never unused
pub(crate) use id_newtypes::IdRange;
pub(crate) use regex::Regex;
pub(crate) use url::Url;
