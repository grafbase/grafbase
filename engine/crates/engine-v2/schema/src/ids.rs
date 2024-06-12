/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    CacheControl, Definition, Enum, EnumValue, FieldDefinition, Graph, Header, InputObject, InputValueDefinition,
    Interface, Object, RequiredFieldArguments, RequiredFieldSet, RequiredScopes, Resolver, Scalar, Schema,
    TypeSystemDirective, Union,
};
use url::Url;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
pub(crate) const MAX_ID: usize = (1 << 29) - 1;

id_newtypes::NonZeroU32! {
    Graph.type_definitions[DefinitionId] => Definition | max(MAX_ID) | index(Schema.graph),
    Graph.type_system_directives[TypeSystemDirectiveId] => TypeSystemDirective | max(MAX_ID) | index(Schema.graph),
    Graph.enum_value_definitions[EnumValueId] => EnumValue | max(MAX_ID) | index(Schema.graph),
    Graph.enum_definitions[EnumId] => Enum | max(MAX_ID) | index(Schema.graph),
    Graph.field_definitions[FieldDefinitionId] => FieldDefinition | max(MAX_ID) | index(Schema.graph),
    Graph.input_object_definitions[InputObjectId] => InputObject | max(MAX_ID) | index(Schema.graph),
    Graph.input_value_definitions[InputValueDefinitionId] => InputValueDefinition | max(MAX_ID) | index(Schema.graph),
    Graph.interface_definitions[InterfaceId] => Interface | max(MAX_ID) | index(Schema.graph),
    Graph.object_definitions[ObjectId] => Object | max(MAX_ID) | index(Schema.graph),
    Graph.scalar_definitions[ScalarId] => Scalar | max(MAX_ID) | index(Schema.graph),
    Graph.union_definitions[UnionId] => Union | max(MAX_ID) | index(Schema.graph),
    Graph.resolvers[ResolverId] => Resolver | max(MAX_ID) | index(Schema.graph),
    Graph.required_field_sets[RequiredFieldSetId] => RequiredFieldSet | max(MAX_ID) | index(Schema.graph),
    Graph.required_fields_arguments[RequiredFieldSetArgumentsId] => RequiredFieldArguments | max(MAX_ID) | index(Schema.graph),
    Graph.cache_control[CacheControlId] => CacheControl | max(MAX_ID) | index(Schema.graph),
    Graph.required_scopes[RequiredScopesId] => RequiredScopes | max(MAX_ID) | index(Schema.graph),
    Schema.headers[HeaderId] => Header | max(MAX_ID),
    Schema.urls[UrlId] => Url | max(MAX_ID),
    Schema.strings[StringId] => String | max(MAX_ID),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubgraphId(u8);

impl std::fmt::Debug for SubgraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subgraph#{}", self.0)
    }
}

impl From<usize> for SubgraphId {
    fn from(id: usize) -> Self {
        Self(u8::try_from(id).expect("Too many subgraphs"))
    }
}
