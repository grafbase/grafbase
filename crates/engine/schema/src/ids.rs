use std::num::NonZero;

use regex::Regex;
use url::Url;
use walker::Walk;

use crate::Schema;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct UrlId(NonZero<u32>);

impl<'a> Walk<&'a Schema> for UrlId {
    type Walker<'w>
        = &'w Url
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        &schema.into()[self]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct StringId(NonZero<u32>);

impl<'a> Walk<&'a Schema> for StringId {
    type Walker<'w>
        = &'w str
    where
        'a: 'w;

    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        &schema.into()[self]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct RegexId(NonZero<u32>);

impl<'a> Walk<&'a Schema> for RegexId {
    type Walker<'w>
        = &'w Regex
    where
        'a: 'w;

    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        &schema.into()[self]
    }
}
