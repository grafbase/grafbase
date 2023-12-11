/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    sources::federation::{DataSource as FederationMetadata, Subgraph},
    Definition, Enum, Field, Header, InputObject, InputValue, Interface, Object, Resolver, Scalar, Schema, Type, Union,
};

macro_rules! id_newtypes {
    ($($ty:ident.$field:ident[$name:ident] => $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $name(std::num::NonZeroU32);

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
                    Self(std::num::NonZeroU32::new((index + 1) as u32).unwrap())
                }
            }

            impl From<$name> for usize {
                fn from(id: $name) -> Self {
                    (id.0.get() - 1) as usize
                }
            }
        )*
    }
}

id_newtypes! {
    Schema.enums[EnumId] => Enum,
    Schema.fields[FieldId] => Field,
    Schema.types[TypeId] => Type,
    Schema.input_objects[InputObjectId] => InputObject,
    Schema.interfaces[InterfaceId] => Interface,
    Schema.objects[ObjectId] => Object,
    Schema.scalars[ScalarId] => Scalar,
    Schema.strings[StringId] => String,
    Schema.unions[UnionId] => Union,
    Schema.resolvers[ResolverId] => Resolver,
    Schema.definitions[DefinitionId] => Definition,
    Schema.input_values[InputValueId] => InputValue,
    Schema.headers[HeaderId] => Header,
    FederationMetadata.subgraphs[SubgraphId] => Subgraph,
}
