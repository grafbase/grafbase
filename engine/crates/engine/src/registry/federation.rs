//! Registry structs for using the engine as an [apollo federation subgraph][1]
//!
//! [1]: https://www.apollographql.com/docs/federation/subgraph-spec

use serde_json::Value;

use super::field_set::all_fieldset_fields_are_present;

/// Takes an `_Any` representation from the federation `_entities` field and determines
/// which `FederationKey` the representation matches.
pub(crate) fn find_key_for_entity<'a>(
    entity: &'a registry_v2::FederationEntity,
    data: &Value,
) -> Option<&'a registry_v2::FederationKey> {
    let object = data.as_object()?;
    entity
        .keys
        .iter()
        .find(|key| all_fieldset_fields_are_present(&key.selections, object))
}
