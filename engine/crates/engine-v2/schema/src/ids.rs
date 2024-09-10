use std::num::NonZero;

use regex::Regex;
use url::Url;
use walker::Walk;

use crate::Schema;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
pub(crate) const MAX_ID: u32 = (1 << 29) - 1;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct UrlId(NonZero<u32>);

impl Walk<Schema> for UrlId {
    type Walker<'a> = &'a Url;
    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        &schema[self]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct StringId(NonZero<u32>);

impl Walk<Schema> for StringId {
    type Walker<'a> = &'a str;

    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        &schema[self]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct RegexId(NonZero<u32>);

impl Walk<Schema> for RegexId {
    type Walker<'a> = &'a Regex;

    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        &schema[self]
    }
}
