#![allow(clippy::panic)]

use std::collections::HashMap;

use grafbase_federated_graph as federated;

use super::Context;
use crate::{subgraphs, VecExt};

/// Responsible for mapping field types between the subgraphs and the federated graph. See
/// [Context::insert_field_type()].
#[derive(Default)]
pub(super) struct FieldTypesMap {
    map: HashMap<subgraphs::FieldTypeId, federated::FieldTypeId>,
}

impl Context<'_> {
    /// Subgraphs field type -> federated graph field type.
    pub(super) fn insert_field_type(&mut self, field_type: subgraphs::FieldTypeWalker<'_>) -> federated::FieldTypeId {
        *self.field_types_map.map.entry(field_type.id).or_insert_with(|| {
            let Some(kind) = self.definitions.get(&field_type.type_name().id).copied() else {
                panic!(
                    "Invariant violation: definition {:?} from field type not registered.",
                    field_type.type_name().as_str()
                )
            };

            federated::FieldTypeId(
                self.out.field_types.push_return_idx(federated::FieldType {
                    kind,
                    inner_is_required: field_type.inner_is_required(),
                    list_wrappers: field_type
                        .iter_wrappers()
                        .map(|wrapper| match wrapper {
                            subgraphs::WrapperTypeKind::List => federated::ListWrapper::NullableList,
                            subgraphs::WrapperTypeKind::NonNullList => federated::ListWrapper::RequiredList,
                        })
                        .collect(),
                }),
            )
        })
    }
}
