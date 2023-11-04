mod back_relation;
mod r#enum;
mod enum_variant;
mod foreign_key;
mod foreign_key_column;
mod forward_relation;
mod relation;
mod table;
mod table_column;
mod unique_constraint;
mod unique_constraint_column;

pub use back_relation::BackRelationWalker;
pub use enum_variant::EnumVariantWalker;
pub use forward_relation::ForwardRelationWalker;
pub use r#enum::EnumWalker;
pub use relation::RelationWalker;
pub use table::TableWalker;
pub use table_column::TableColumnWalker;
pub use unique_constraint::UniqueConstraintWalker;
pub use unique_constraint_column::UniqueConstraintColumnWalker;

pub(crate) use foreign_key::ForeignKeyWalker;
pub(crate) use foreign_key_column::ForeignKeyColumnWalker;

use crate::database_definition::DatabaseDefinition;
use std::ops::Range;

use super::names::StringId;

/// An abstraction to iterate over an introspected PostgreSQL database.
///
/// The `Id` must be something that points to an object in the database.
#[derive(Clone, Copy)]
pub struct Walker<'a, Id> {
    pub(super) id: Id,
    pub(super) database_definition: &'a DatabaseDefinition,
}

impl<'a, Id> PartialEq for Walker<'a, Id>
where
    Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, Id> Walker<'a, Id>
where
    Id: Copy,
{
    pub fn new(id: Id, database_definition: &'a DatabaseDefinition) -> Self {
        Self {
            id,
            database_definition,
        }
    }

    pub fn id(self) -> Id {
        self.id
    }

    fn walk<OtherId>(self, id: OtherId) -> Walker<'a, OtherId> {
        self.database_definition.walk(id)
    }

    fn get_name(self, id: StringId) -> &'a str {
        self.database_definition.names.get_name(id)
    }
}

/// For a slice sorted by a key K, return the contiguous range of items matching the key.
fn range_for_key<I, K>(slice: &[I], key: K, extract: fn(&I) -> K) -> Range<usize>
where
    K: Copy + Ord + PartialOrd + PartialEq,
{
    let seed = slice.binary_search_by_key(&key, extract).unwrap_or(0);
    let mut iter = slice[..seed].iter();
    let start = match iter.rposition(|i| extract(i) != key) {
        None => 0,
        Some(other) => other + 1,
    };
    let mut iter = slice[seed..].iter();
    let end = seed + iter.position(|i| extract(i) != key).unwrap_or(slice.len() - seed);
    start..end
}
