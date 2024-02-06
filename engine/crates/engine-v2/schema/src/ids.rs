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

            impl std::ops::Index<crate::ids::IdRange<$name>> for $ty {
                type Output = [$out];

                fn index(&self, range: crate::ids::IdRange<$name>) -> &Self::Output {
                    let crate::ids::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &self.$field[start..end]
                }
            }


            impl From<usize> for $name {
                fn from(index: usize) -> Self {
                    assert!(index <= MAX_ID, "{}", $msg);
                    Self(std::num::NonZeroU32::new((index + 1) as u32).unwrap())
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

// Not necessary anymore when Rust stabilize std::iter::Step
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IdRange<Id: Copy> {
    pub start: Id,
    pub end: Id,
}

impl<Id> IdRange<Id>
where
    Id: From<usize> + Copy,
    usize: From<Id>,
{
    pub fn empty() -> Self {
        Self {
            start: Id::from(0),
            end: Id::from(0),
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Id> {
        (usize::from(self.start)..usize::from(self.end)).map(Id::from)
    }

    pub fn get(&self, i: usize) -> Option<Id> {
        let i = i + usize::from(self.start);
        if i < usize::from(self.end) {
            Some(Id::from(i))
        } else {
            None
        }
    }
}

impl<Src, Target> From<(Src, usize)> for IdRange<Target>
where
    Src: Copy,
    usize: From<Src>,
    Target: From<usize> + From<Src> + Copy,
{
    fn from((start, len): (Src, usize)) -> Self {
        IdRange {
            start: start.into(),
            end: Target::from(usize::from(start) + len),
        }
    }
}
