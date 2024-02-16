//! This module is responsible for mapping identifiers between federated schema and the engine v2
//! schema when the mapping is not 1:1.
//!
//! As of 3b56f12c95d334ce6cb46f4f8654ce531a69f975, this only happens when @inaccessible items
//! are removed.

use crate::ids::*;

pub(super) trait MappableId: Copy {
    type Counterpart: Copy + From<usize>;

    fn get_idx(self) -> usize;
    fn get_map(mapper: &IdMapper) -> &[usize];
    fn get_map_mut(mapper: &mut IdMapper) -> &mut Vec<usize>;
}

macro_rules! mapped_ids {
    ($($name:ident : $from:ty => $to:ty)*) => {

        #[derive(Default, Debug)]
        pub(super) struct IdMapper {
            $(
                /// The index of skipped ids.
                $name: Vec<usize>,
            )*
        }

        $(
            impl MappableId for $from {
                type Counterpart = $to;

                fn get_idx(self) -> usize {
                    self.0
                }

                fn get_map(mapper: &IdMapper) -> &[usize] {
                    &mapper.$name
                }

                fn get_map_mut(mapper: &mut IdMapper) -> &mut Vec<usize> {
                    &mut mapper.$name
                }
            }
        )*
    };
}

mapped_ids!(
    fields: federated_graph::FieldId => FieldId
    input_values: federated_graph::InputValueDefinitionId => InputValueDefinitionId
    enum_values: federated_graph::EnumValueId => EnumValueId
);

impl IdMapper {
    pub(super) fn skip<Id: MappableId>(&mut self, id: Id) {
        let idx = id.get_idx();
        let entries = Id::get_map_mut(self);

        if let Some(last_entry) = entries.last().copied() {
            assert!(last_entry < idx);
        }

        entries.push(idx);
    }

    pub(super) fn map<Id: MappableId>(&self, id: Id) -> Option<Id::Counterpart> {
        let idx = id.get_idx();
        let map = Id::get_map(self);
        let skipped = map.partition_point(|skipped| *skipped <= idx);

        if let Some(last) = map[..skipped].last().copied() {
            if last == idx {
                return None;
            }
        }

        Some(Id::Counterpart::from(idx - skipped))
    }

    pub(super) fn map_range<Id: MappableId>(&self, (start_id, len): (Id, usize)) -> crate::IdRange<Id::Counterpart>
    where
        usize: From<Id::Counterpart>,
    {
        let start_idx = start_id.get_idx();
        let skipped = Id::get_map(self);

        // How many ids were skipped before the range.
        let skipped_before = skipped.partition_point(|skipped| *skipped < start_idx);

        // How many ids were skipped inside the range.
        let skipped_inside = skipped[skipped_before..]
            .iter()
            .take_while(|skipped| **skipped < (start_idx + len))
            .count();

        let start = start_idx - skipped_before;

        IdRange {
            start: From::from(start),
            end: From::from(start + (len - skipped_inside)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic(expected = "assertion failed: last_entry < idx")]
    fn skip_out_of_order() {
        let mut mapper = IdMapper::default();
        mapper.skip(federated_graph::InputValueDefinitionId(5));
        mapper.skip(federated_graph::InputValueDefinitionId(5)); // boom
    }
}
