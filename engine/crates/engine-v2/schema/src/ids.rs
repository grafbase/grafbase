use std::num::NonZero;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
pub(crate) const MAX_ID: usize = (1 << 29) - 1;

#[id_derives::id(max(MAX_ID))]
pub struct DefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct TypeSystemDirectiveId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct EnumValueId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct EnumDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct FieldDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct InputObjectDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct InputValueDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct InterfaceDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct ObjectDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct ScalarDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct UnionDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct ResolverDefinitionId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct RequiredFieldSetId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct RequiredFieldId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct CacheControlId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct RequiredScopesId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct AuthorizedDirectiveId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct UrlId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct StringId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct RegexId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct HeaderRuleId(NonZero<u32>);

#[id_derives::id(max(MAX_ID))]
pub struct SubgraphId(NonZero<u16>);
