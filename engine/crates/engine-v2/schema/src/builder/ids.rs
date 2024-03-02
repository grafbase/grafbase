//! This module is responsible for mapping identifiers between federated schema and the engine v2
//! schema when the mapping is not 1:1.
//!
//! As of 3b56f12c95d334ce6cb46f4f8654ce531a69f975, this only happens when @inaccessible items
//! are removed.

use std::marker::PhantomData;

use id_newtypes::IdRange;

pub(super) struct IdMapper<FgId: Into<usize>, Id: From<usize> + Copy> {
    skipped_ids: Vec<usize>,
    _fgid: PhantomData<FgId>,
    _id: PhantomData<Id>,
}

impl<FgId, Id> Default for IdMapper<FgId, Id>
where
    FgId: Into<usize>,
    Id: From<usize> + Copy,
{
    fn default() -> Self {
        IdMapper {
            skipped_ids: Vec::new(),
            _fgid: PhantomData,
            _id: PhantomData,
        }
    }
}

impl<FgId, Id> IdMapper<FgId, Id>
where
    FgId: Into<usize>,
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

    /// Map a federated_graph id to an engine_schema id taking the skipped IDs into account.
    pub(super) fn map(&self, id: FgId) -> Option<Id> {
        let idx = id.into();
        let skipped = self.skipped_ids.partition_point(|skipped| *skipped <= idx);

        if let Some(last) = self.skipped_ids[..skipped].last().copied() {
            if last == idx {
                return None;
            }
        }

        Some(Id::from(idx - skipped))
    }

    pub(super) fn map_range(&self, (start_id, len): (FgId, usize)) -> crate::IdRange<Id> {
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

    type IdMapper = super::IdMapper<federated_graph::InputValueDefinitionId, InputValueDefinitionId>;

    #[test]
    fn skip_basic() {
        let id = federated_graph::InputValueDefinitionId(2);
        let mut mapper = IdMapper::default();
        assert_eq!(InputValueDefinitionId::from(2), mapper.map(id).unwrap());
        mapper.skip(federated_graph::InputValueDefinitionId(1));
        assert_eq!(InputValueDefinitionId::from(1), mapper.map(id).unwrap());
    }

    #[test]
    fn map_skipped() {
        let id = federated_graph::InputValueDefinitionId(5);
        let mut mapper = IdMapper::default();
        mapper.skip(id);
        assert!(mapper.map(id).is_none());
    }

    #[test]
    fn map_range_basic() {
        let range = (federated_graph::InputValueDefinitionId(6), 10);
        let mut mapper = IdMapper::default();
        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(6),
                end: InputValueDefinitionId::from(16)
            },
            mapper.map_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(2));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(15)
            },
            mapper.map_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(6));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(14)
            },
            mapper.map_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(9));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(13)
            },
            mapper.map_range(range)
        );

        mapper.skip(federated_graph::InputValueDefinitionId(20));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(13)
            },
            mapper.map_range(range)
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
