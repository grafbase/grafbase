use super::{enum_variant::EnumVariantWalker, Walker};
use crate::database_definition::{names::StringId, Enum, EnumId, EnumVariantId};

/// An enum definition in the database.
pub type EnumWalker<'a> = Walker<'a, EnumId>;

impl<'a> EnumWalker<'a> {
    /// The schema this enum belongs to.
    pub fn schema(self) -> &'a str {
        &self.database_definition.schemas[self.get().schema_id().0 as usize]
    }

    /// The name of the enum in the database.
    pub fn database_name(self) -> &'a str {
        self.get_name(self.get().database_name())
    }

    /// The name of the enum in the GraphQL APIs.
    pub fn client_name(self) -> &'a str {
        self.get_name(self.get().client_name())
    }

    /// The variants that are part of the enum.
    pub fn variants(self) -> impl ExactSizeIterator<Item = EnumVariantWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.enum_variants, self.id, |variant| {
            variant.enum_id()
        });

        range.map(move |id| self.walk(EnumVariantId(id as u32)))
    }

    fn get(self) -> &'a Enum<StringId> {
        &self.database_definition.enums[self.id.0 as usize]
    }
}
