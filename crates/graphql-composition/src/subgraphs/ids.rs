use super::*;

macro_rules! make_ids {
    ($($($path:ident).* [ $id_type_name:ident ] -> $out:ident, )*) => {
        $(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $id_type_name(usize);

        impl From<usize> for $id_type_name {
            fn from(value: usize) -> Self {
                $id_type_name(value)
            }
        }

        impl From<$id_type_name> for usize {
            fn from(value: $id_type_name) -> Self {
                value.0
            }
        }

        impl std::ops::Index<$id_type_name> for Subgraphs {
            type Output = $out;

            fn index(&self, index: $id_type_name) -> &$out {
                &self$(.$path)*[index.0]
            }
        }

        impl std::ops::IndexMut<$id_type_name> for Subgraphs {
            fn index_mut(&mut self, index: $id_type_name) -> &mut $out {
                &mut self$(.$path)*[index.0]
            }
        }
        )*
    };
}

make_ids! {
    fields.arguments[ArgumentId] -> ArgumentRecord,
    definitions.definitions[DefinitionId] -> Definition,
    directives.directive_definitions[DirectiveDefinitionId] -> DirectiveDefinition,
    directives.extra_directives[DirectiveId] -> ExtraDirectiveRecord,
    enums.values[EnumValueId] -> EnumValue,
    extensions[ExtensionId] -> ExtensionRecord,
    fields.fields[FieldId] -> FieldTuple,
    keys.keys[KeyId] -> Key,
    linked_schemas.definitions[LinkedDefinitionId] -> LinkedDefinitionRecord,
    linked_schemas.schemas[LinkedSchemaId] -> LinkedSchemaRecord,
    subgraphs[SubgraphId] -> Subgraph,
}
