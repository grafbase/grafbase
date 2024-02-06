/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    sources::federation::{DataSource as FederationDataSource, Subgraph},
    CacheConfig, Definition, Directive, Enum, EnumValue, Field, Header, InputObject, InputValue, Interface, Object,
    Resolver, Scalar, Schema, Type, Union,
};
use url::Url;

/// Reserving the 4 upper bits for some fun with bit packing. It still leaves 268 million possible values.
/// And it's way easier to increase that limit if needed that to reserve some bits later!
/// Currently, we use the two upper bits of the FieldId for the ResponseEdge in the engine.
const MAX_ID: usize = (1 << 29) - 1;

macro_rules! id_newtypes {
    ($($ty:ident.$field:ident[$name:ident] => $out:ident unless $msg:literal,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $name(std::num::NonZeroU32);

            impl $name {
                pub const fn const_from_usize(index: usize, msg: &str) -> Self {
                    assert!(index <= MAX_ID, "{}", msg);
                    let Some(inner) = std::num::NonZeroU32::new((index + 1) as u32) else {
                        panic!("{}", msg)
                    };
                    Self(inner)

                }
            }

            impl std::ops::Index<$name> for $ty {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$field[(index.0.get() - 1) as usize]
                }
            }

            impl std::ops::IndexMut<$name> for $ty {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$field[(index.0.get() - 1) as usize]
                }
            }


            impl From<usize> for $name {
                fn from(index: usize) -> Self {
                    Self::const_from_usize(index, $msg)
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as usize
                }
            }

            impl From<$name> for u32 {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1)
                }
            }
        )*
    }
}

id_newtypes! {
    Schema.definitions[DefinitionId] => Definition unless "Too many definitions",
    Schema.directives[DirectiveId] => Directive unless "Too many directives",
    Schema.enum_values[EnumValueId] => EnumValue unless "Too many enum values",
    Schema.enums[EnumId] => Enum unless "Too many enums",
    Schema.fields[FieldId] => Field unless "Too many fields",
    Schema.headers[HeaderId] => Header unless "Too many headers",
    Schema.input_objects[InputObjectId] => InputObject unless "Too many input objects",
    Schema.input_values[InputValueId] => InputValue unless "Too many input values",
    Schema.interfaces[InterfaceId] => Interface unless "Too many interfaces",
    Schema.objects[ObjectId] => Object unless "Too many objects",
    Schema.resolvers[ResolverId] => Resolver unless "Too many resolvers",
    Schema.scalars[ScalarId] => Scalar unless "Too many scalars",
    Schema.types[TypeId] => Type unless "Too many types",
    Schema.unions[UnionId] => Union unless "Too many unions",
    Schema.urls[UrlId] => Url unless "Too many urls",
    Schema.strings[StringId] => String unless "Too many strings",
    FederationDataSource.subgraphs[SubgraphId] => Subgraph unless "Too many subgraphs",
    Schema.cache_configs[CacheConfigId] => CacheConfig unless "Too many cache configs",
}
