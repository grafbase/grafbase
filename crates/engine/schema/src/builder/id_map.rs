//! This module is responsible for mapping identifiers between federated schema and the engine v2
//! schema when the mapping is not 1:1.
//!
//! As of 3b56f12c95d334ce6cb46f4f8654ce531a69f975, this only happens when @inaccessible items
//! are removed.

use std::marker::PhantomData;

use federated_graph::FederatedGraph;
use id_newtypes::IdRange;

use crate::{EnumDefinitionId, EnumValueId, FieldDefinitionId, InputValueDefinitionId, ScalarDefinitionId};

pub(crate) struct IdMap<FgId: Into<usize>, Id: From<usize> + Copy> {
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
    pub(crate) input_value: IdMap<federated_graph::InputValueDefinitionId, InputValueDefinitionId>,
}

impl IdMaps {
    pub fn new(graph: &FederatedGraph) -> Self {
        let mut idmaps = IdMaps {
            input_value: Default::default(),
        };

        for (i, input_value) in graph.input_value_definitions.iter().enumerate() {
            if is_inaccessible(graph, input_value.directives) {
                idmaps
                    .input_value
                    .skip(federated_graph::InputValueDefinitionId::from(i))
            }
        }

        idmaps
    }
}

impl IdMaps {
    pub(crate) fn convert_input_value_definition_id(
        &self,
        id: federated_graph::InputValueDefinitionId,
    ) -> InputValueDefinitionId {
        self.input_value.get(id).expect("failed to map an id")
    }

    pub(crate) fn convert_input_value_definitions_range(
        &self,
        values: federated_graph::InputValueDefinitions,
    ) -> IdRange<InputValueDefinitionId> {
        self.input_value.get_range(values)
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
        let id = federated_graph::InputValueDefinitionId::from(2);
        let mut mapper = IdMapper::default();
        assert_eq!(InputValueDefinitionId::from(2usize), mapper.get(id).unwrap());
        mapper.skip(federated_graph::InputValueDefinitionId::from(1));
        assert_eq!(InputValueDefinitionId::from(1usize), mapper.get(id).unwrap());
    }

    #[test]
    fn map_skipped() {
        let id = federated_graph::InputValueDefinitionId::from(5);
        let mut mapper = IdMapper::default();
        mapper.skip(id);
        assert!(mapper.get(id).is_none());
    }

    #[test]
    fn map_range_basic() {
        let range = (federated_graph::InputValueDefinitionId::from(6), 10);
        let mut mapper = IdMapper::default();
        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(6usize),
                end: InputValueDefinitionId::from(16usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId::from(2));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(15usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId::from(6));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(14usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId::from(9));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5usize),
                end: InputValueDefinitionId::from(13usize)
            },
            mapper.get_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId::from(20));

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
        mapper.skip(federated_graph::InputValueDefinitionId::from(5));
        mapper.skip(federated_graph::InputValueDefinitionId::from(3));
        // boom
    }

    #[test]
    #[should_panic(expected = "Broken invariant: ids must be skipped in order")]
    fn skip_twice() {
        let mut mapper = IdMapper::default();
        mapper.skip(federated_graph::InputValueDefinitionId::from(5));
        mapper.skip(federated_graph::InputValueDefinitionId::from(5));
        // boom
    }
}
