#![allow(clippy::panic)]

use super::Context;
use crate::{federated_graph as federated, subgraphs};
use std::collections::HashMap;

/// Responsible for mapping field types between the subgraphs and the federated graph. See
/// [Context::insert_field_type()].
#[derive(Default)]
pub(super) struct FieldTypesMap {
    map: HashMap<subgraphs::FieldType, federated::Type>,
}

impl Context<'_> {
    /// Subgraphs field type -> federated graph field type.
    pub(super) fn insert_field_type(&mut self, field_type: subgraphs::FieldType) -> federated::Type {
        let type_name = self.insert_str(&self.subgraphs[field_type.definition_name_id]);
        *self.field_types_map.map.entry(field_type).or_insert_with(|| {
            let Some(definition) = self.definitions.get(&type_name).copied() else {
                panic!(
                    "Invariant violation: definition {:?} from field type not registered.",
                    self.subgraphs[field_type.definition_name_id],
                )
            };

            federated::Type {
                definition,
                wrapping: field_type.wrapping,
            }
        })
    }
}
