/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    CacheConfig, Definition, Directive, Enum, EnumValue, FieldDefinition, Header, InputObject, InputValueDefinition,
    Interface, Object, RequiredFieldArguments, RequiredFieldSet, Resolver, Scalar, Schema, SchemaInputValues, Union,
};
use url::Url;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
pub(crate) const MAX_ID: usize = (1 << 29) - 1;

id_newtypes::NonZeroU32! {
    Schema.definitions[DefinitionId] => Definition | max MAX_ID,
    Schema.directives[DirectiveId] => Directive | max MAX_ID,
    Schema.enum_values[EnumValueId] => EnumValue | max MAX_ID,
    Schema.enums[EnumId] => Enum | max MAX_ID,
    Schema.field_definitions[FieldDefinitionId] => FieldDefinition | max MAX_ID,
    Schema.headers[HeaderId] => Header | max MAX_ID,
    Schema.input_objects[InputObjectId] => InputObject | max MAX_ID,
    Schema.input_value_definitions[InputValueDefinitionId] => InputValueDefinition | max MAX_ID,
    Schema.interfaces[InterfaceId] => Interface | max MAX_ID,
    Schema.objects[ObjectId] => Object | max MAX_ID,
    Schema.resolvers[ResolverId] => Resolver | max MAX_ID,
    Schema.scalars[ScalarId] => Scalar | max MAX_ID,
    Schema.unions[UnionId] => Union | max MAX_ID,
    Schema.urls[UrlId] => Url | max MAX_ID,
    Schema.strings[StringId] => String | max MAX_ID,
    Schema.cache_configs[CacheConfigId] => CacheConfig | max MAX_ID,
    Schema.required_field_sets[RequiredFieldSetId] => RequiredFieldSet | max MAX_ID,
    Schema.required_fields_arguments[RequiredFieldSetArgumentsId] => RequiredFieldArguments | max MAX_ID,
}

impl<T> std::ops::Index<T> for Schema
where
    SchemaInputValues: std::ops::Index<T>,
{
    type Output = <SchemaInputValues as std::ops::Index<T>>::Output;

    fn index(&self, index: T) -> &Self::Output {
        &self.input_values[index]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
