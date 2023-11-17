/// Isolating ids from the rest to prevent misuse of the NonZeroU32.
/// They can only be created by From<usize>
use crate::{
    DataSource, Enum, Field, FieldType, InputObject, Interface, Object, Resolver, Scalar, Schema, Subgraph, Union,
};

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $name(std::num::NonZeroU32);

            impl std::ops::Index<$name> for Schema {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[(index.0.get() - 1) as usize]
                }
            }

            impl std::ops::IndexMut<$name> for Schema {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[(index.0.get() - 1) as usize]
                }
            }


            impl From<usize> for $name {
                fn from(index: usize) -> Self {
                    Self(std::num::NonZeroU32::new((index + 1) as u32).unwrap())
                }
            }
        )*
    }
}

// TODO: won't work with multiple sources.
impl From<SubgraphId> for DataSourceId {
    fn from(subgraph_id: SubgraphId) -> Self {
        DataSourceId(subgraph_id.0)
    }
}

id_newtypes! {
    DataSourceId + data_sources + DataSource,
    EnumId + enums + Enum,
    FieldId + fields + Field,
    FieldTypeId + field_types + FieldType,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
    ResolverId + resolvers + Resolver,
}
