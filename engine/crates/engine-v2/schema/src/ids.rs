/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    sources::federation::{DataSource as FederationDataSource, Subgraph},
    CacheConfig, Definition, Directive, Enum, EnumValue, Field, Header, InputObject, InputValueDefinition, Interface,
    Object, Resolver, Scalar, Schema, Type, Union,
};
use url::Url;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
pub(crate) const MAX_ID: usize = (1 << 29) - 1;

id_newtypes::U32! {
    Schema.definitions[DefinitionId] => Definition | unless "Too many definitions" max MAX_ID,
    Schema.directives[DirectiveId] => Directive | unless "Too many directives" max MAX_ID,
    Schema.enum_values[EnumValueId] => EnumValue | unless "Too many enum values" max MAX_ID,
    Schema.enums[EnumId] => Enum | unless "Too many enums" max MAX_ID,
    Schema.fields[FieldId] => Field | unless "Too many fields" max MAX_ID,
    Schema.headers[HeaderId] => Header | unless "Too many headers" max MAX_ID,
    Schema.input_objects[InputObjectId] => InputObject | unless "Too many input objects" max MAX_ID,
    Schema.input_value_definitions[InputValueDefinitionId] => InputValueDefinition | unless "Too many input value definitions" max MAX_ID,
    Schema.interfaces[InterfaceId] => Interface | unless "Too many interfaces" max MAX_ID,
    Schema.objects[ObjectId] => Object | unless "Too many objects" max MAX_ID,
    Schema.resolvers[ResolverId] => Resolver | unless "Too many resolvers" max MAX_ID,
    Schema.scalars[ScalarId] => Scalar | unless "Too many scalars" max MAX_ID,
    Schema.types[TypeId] => Type | unless "Too many types" max MAX_ID,
    Schema.unions[UnionId] => Union | unless "Too many unions" max MAX_ID,
    Schema.urls[UrlId] => Url | unless "Too many urls" max MAX_ID,
    Schema.strings[StringId] => String | unless "Too many strings" max MAX_ID,
    FederationDataSource.subgraphs[SubgraphId] => Subgraph | unless "Too many subgraphs" max MAX_ID,
    Schema.cache_configs[CacheConfigId] => CacheConfig | unless "Too many cache configs" max MAX_ID,
}
