use super::FederatedGraph;

macro_rules! id_newtypes {
    ($($storage:ident [ $name:ident ] -> $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(usize);

            impl From<$name> for usize {
              fn from(value: $name) -> usize {
                value.0
              }
            }

            impl From<usize> for $name {
              fn from(value: usize) -> $name {
                $name(value)
              }
            }

            impl std::ops::Index<$name> for FederatedGraph {
                type Output = super::$out;

                fn index(&self, index: $name) -> &super::$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraph {
                fn index_mut(&mut self, index: $name) -> &mut super::$out {
                    &mut self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    type_definitions[TypeDefinitionId] -> TypeDefinitionRecord,
    enum_values[EnumValueId] -> EnumValueRecord,
}
