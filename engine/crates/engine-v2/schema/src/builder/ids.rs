//! This module is responsible for mapping identifiers between federated schema and the engine v2
//! schema when the mapping is not 1:1.
//!
//! As of 3b56f12c95d334ce6cb46f4f8654ce531a69f975, this only happens when @inaccessible items
//! are removed.

use std::marker::PhantomData;

use federated_graph::FederatedGraph;
use id_newtypes::IdRange;

use crate::{EnumDefinitionId, EnumValueId, FieldDefinitionId, InputValueDefinitionId, ScalarDefinitionId};

use super::graph::is_inaccessible;

pub(super) struct IdMap<FgId: Into<usize>, Id: From<usize> + Copy> {
    skipped_ids: Vec<usize>,
    _fgid: PhantomData<FgId>,
    _id: PhantomData<Id>,
}

impl<FgId, Id> Default for IdMap<FgId, Id>
where
    FgId: Into<usize>,
    Id: From<usize> + Copy,
{
    fn default() -> Self {
        IdMap {
            skipped_ids: Vec::new(),
            _fgid: PhantomData,
            _id: PhantomData,
        }
    }
}

pub(crate) struct IdMaps {
    pub(crate) field: IdMap<federated_graph::FieldId, FieldDefinitionId>,
    pub(crate) input_value: IdMap<federated_graph::InputValueDefinitionId, InputValueDefinitionId>,
    pub(crate) enum_values: IdMap<federated_graph::EnumValueId, EnumValueId>,
    /// The index in that vector is the id in the graph being built.
    scalar_ids: Vec<federated_graph::TypeDefinitionId>,
    enum_ids: Vec<federated_graph::TypeDefinitionId>,
}

impl IdMaps {
    pub fn new(graph: &FederatedGraph) -> Self {
        let mut idmaps = IdMaps {
            field: Default::default(),
            input_value: Default::default(),
            enum_values: IdMap::default(),
            scalar_ids: graph.iter_scalars().map(|s| s.id()).collect(),
            enum_ids: graph.iter_enums().map(|e| e.id()).collect(),
        };

        for (i, field) in graph.fields.iter().enumerate() {
            if is_inaccessible(graph, field.composed_directives) {
                idmaps.field.skip(federated_graph::FieldId(i))
            }
        }
        for (i, input_value) in graph.input_value_definitions.iter().enumerate() {
            if is_inaccessible(graph, input_value.directives) {
                idmaps.input_value.skip(federated_graph::InputValueDefinitionId(i))
            }
        }

        idmaps
    }
}

impl IdMaps {
    pub(crate) fn convert_scalar_id(
        &self,
        federated_scalar_id: federated_graph::TypeDefinitionId,
    ) -> ScalarDefinitionId {
        self.scalar_ids
            .binary_search(&federated_scalar_id)
            .expect("Failed to convert scalar id")
            .into()
    }

    pub(crate) fn convert_enum_id(&self, federated_enum_id: federated_graph::TypeDefinitionId) -> EnumDefinitionId {
        self.enum_ids
            .binary_search(&federated_enum_id)
            .expect("Failed to convert scalar id")
            .into()
    }

    pub(crate) fn convert_enum_value_id(
        &self,
        federated_enum_value_id: federated_graph::EnumValueId,
    ) -> Option<EnumValueId> {
        self.enum_values.get(federated_enum_value_id)
    }
}

impl<FgId, Id> IdMap<FgId, Id>
where
    usize: From<FgId>,
    Id: From<usize> + Copy,
{
    /// Mark an id as skipped in the target schema. The element has to be actually filtered out, separately.
    pub(super) fn skip(&mut self, id: FgId) {
        let idx = id.into();
        if let Some(last_entry) = self.skipped_ids.last().copied() {
            assert!(last_entry < idx, "Broken invariant: ids must be skipped in order");
        }

        self.skipped_ids.push(idx);
    }

    pub(super) fn contains(&self, id: impl Into<FgId>) -> bool {
        self.get(id).is_some()
    }

    /// Map a federated_graph id to an engine_schema id taking the skipped IDs into account.
    pub(super) fn get(&self, id: impl Into<FgId>) -> Option<Id> {
        let idx = usize::from(id.into());
        let skipped = self.skipped_ids.partition_point(|skipped| *skipped <= idx);

        if let Some(last) = self.skipped_ids[..skipped].last().copied() {
            if last == idx {
                return None;
            }
        }

        Some(Id::from(idx - skipped))
    }

    pub(super) fn get_range(&self, (start_id, len): (FgId, usize)) -> crate::IdRange<Id> {
        let start_idx = start_id.into();
        // How many ids were skipped before the range.
        let skipped_ids_count_before_start = self.skipped_ids.partition_point(|skipped| *skipped < start_idx);

        // How many ids were skipped inside the range.
        let skipped_ids_count_between_start_and_end = self.skipped_ids[skipped_ids_count_before_start..]
            .iter()
            .take_while(|skipped| **skipped < (start_idx + len))
            .count();

        let start = start_idx - skipped_ids_count_before_start;

        IdRange {
            start: From::from(start),
            end: From::from(start + (len - skipped_ids_count_between_start_and_end)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InputValueDefinitionId;

    use super::*;

    type IdMapper = super::IdMap<federated_graph::InputValueDefinitionId, InputValueDefinitionId>;

    #[test]
    fn skip_basic() {
        let id = federated_graph::InputValueDefinitionId(2);
        let mut mapper = IdMapper::default();
        assert_eq!(InputValueDefinitionId::from(2usize), mapper.get(id).unwrap());
        mapper.skip(federated_graph::InputValueDefinitionId(1));
        assert_eq!(InputValueDefinitionId::from(1usize), mapper.get(id).unwrap());
    }

    #[test]
    fn map_skipped() {
        let id = federated_graph::InputValueDefinitionId(5);
        let mut mapper = IdMapper::default();
        mapper.skip(id);
        assert!(mapper.get(id).is_none());
    }

    #[test]
    fn map_range_basic() {
        let range = (federated_graph::InputValueDefinitionId(6), 10);
        let mut mapper = IdMapper::default();
        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(6usize),
                end: InputValueDefinitionId::from(16usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(2));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(15usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(6));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(14usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(9));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(13usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(20));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(13usize)
            },
            mapper.get_range(range)
        );
    }

    #[test]
    #[should_panic(expected = "Broken invariant: ids must be skipped in order")]
    fn skip_out_of_order() {
        let mut mapper = IdMapper::default();
        mapper.skip(federated_graph::InputValueDefinitionId(5));
        mapper.skip(federated_graph::InputValueDefinitionId(3));
        // boom
    }

    #[test]
    #[should_panic(expected = "Broken invariant: ids must be skipped in order")]
    fn skip_twice() {
        let mut mapper = IdMapper::default();
        mapper.skip(federated_graph::InputValueDefinitionId(5));
        mapper.skip(federated_graph::InputValueDefinitionId(5));
        // boom
    }
}
