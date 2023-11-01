use crate::{subgraphs, VecExt};
use grafbase_federated_graph as federated;
use std::collections::HashMap;

/// Responsible for mapping field types between the subgraphs and the federated graph.
pub(super) struct FieldTypesMap {
    pub(super) field_types: Vec<federated::FieldType>,
    pub(super) definitions: HashMap<subgraphs::StringId, federated::Definition>,

    map: HashMap<subgraphs::FieldTypeId, federated::FieldTypeId>,
}

impl FieldTypesMap {
    pub(super) fn new(definitions: HashMap<subgraphs::StringId, federated::Definition>) -> Self {
        Self {
            definitions,
            map: HashMap::default(),
            field_types: Vec::new(),
        }
    }

    pub(super) fn insert(
        &mut self,
        field_type: subgraphs::FieldTypeWalker<'_>,
    ) -> federated::FieldTypeId {
        *self.map.entry(field_type.id).or_insert_with(|| {
            let Some(kind) = self.definitions.get(&field_type.type_name().id).copied() else {
                panic!(
                    "Invariant violation: definition {:?} from field type not registered.",
                    field_type.type_name().as_str()
                )
            };

            federated::FieldTypeId(
                self.field_types.push_return_idx(federated::FieldType {
                    kind,
                    inner_is_required: field_type.inner_is_required(),
                    list_wrappers: field_type
                        .iter_wrappers()
                        .map(|wrapper| match wrapper {
                            subgraphs::WrapperTypeKind::List => {
                                federated::ListWrapper::NullableList
                            }
                            subgraphs::WrapperTypeKind::NonNullList => {
                                federated::ListWrapper::RequiredList
                            }
                        })
                        .collect(),
                }),
            )
        })
    }
}
