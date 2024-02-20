//! This module is responsible for mapping identifiers between federated schema and the engine v2
//! schema when the mapping is not 1:1.
//!
//! As of 3b56f12c95d334ce6cb46f4f8654ce531a69f975, this only happens when @inaccessible items
//! are removed.

use crate::ids::*;

macro_rules! mapped_ids {
    ($($name:ident : $from:ty => $to:ty)*) => {

        #[derive(Default, Debug)]
        pub(super) struct IdMapper {
            $(
                /// The indexes of the skipped ids.
                $name: Vec<usize>,
            )*
        }

        $(
            #[allow(unused)]
            pub(super) mod $name {
                use super::*;

                /// Mark an id as skipped in the target schema. The element has to be actually filtered out, separately.
                pub(crate) fn skip(mapper: &mut IdMapper, id: $from) {
                    super::skip_impl(&mut mapper.$name, id.0)
                }

                /// Map a federated_graph id to an engine_schema id taking the skipped IDs into account.
                pub(crate) fn map(mapper: &IdMapper, id: $from) -> Option<$to> {
                    super::map_impl(&mapper.$name, id.0)
                }

                pub(crate) fn map_range(mapper: &IdMapper, range: ($from, usize)) -> IdRange<$to> {
                    super::map_range_impl(&mapper.$name, (range.0.0, range.1))
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

fn skip_impl(skipped_ids: &mut Vec<usize>, idx: usize) {
    if let Some(last_entry) = skipped_ids.last().copied() {
        assert!(last_entry < idx, "Broken invariant: ids must be skipped in order");
    }

    skipped_ids.push(idx);
}

fn map_impl<T: From<usize>>(skipped_ids: &[usize], idx: usize) -> Option<T> {
    let skipped = skipped_ids.partition_point(|skipped| *skipped <= idx);

    if let Some(last) = skipped_ids[..skipped].last().copied() {
        if last == idx {
            return None;
        }
    }

    Some(T::from(idx - skipped))
}

fn map_range_impl<T: From<usize> + Copy>(skipped_ids: &[usize], (start_idx, len): (usize, usize)) -> crate::IdRange<T> {
    // How many ids were skipped before the range.
    let skipped_ids_count_before_start = skipped_ids.partition_point(|skipped| *skipped < start_idx);

    // How many ids were skipped inside the range.
    let skipped_ids_count_between_start_and_end = skipped_ids[skipped_ids_count_before_start..]
        .iter()
        .take_while(|skipped| **skipped < (start_idx + len))
        .count();

    let start = start_idx - skipped_ids_count_before_start;

    IdRange {
        start: From::from(start),
        end: From::from(start + (len - skipped_ids_count_between_start_and_end)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_basic() {
        let id = federated_graph::InputValueDefinitionId(2);
        let mut mapper = IdMapper::default();
        assert_eq!(InputValueDefinitionId::from(2), input_values::map(&mapper, id).unwrap());
        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(1));
        assert_eq!(InputValueDefinitionId::from(1), input_values::map(&mapper, id).unwrap());
    }

    #[test]
    fn map_skipped() {
        let id = federated_graph::InputValueDefinitionId(5);
        let mut mapper = IdMapper::default();
        input_values::skip(&mut mapper, id);
        assert!(input_values::map(&mapper, id).is_none());
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
            input_values::map_range(&mapper, range)
        );

        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(2));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(15)
            },
            input_values::map_range(&mapper, range)
        );

        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(6));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(14)
            },
            input_values::map_range(&mapper, range)
        );

        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(9));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(13)
            },
            input_values::map_range(&mapper, range)
        );

        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(20));

        assert_eq!(
            IdRange {
                start: InputValueDefinitionId::from(5),
                end: InputValueDefinitionId::from(13)
            },
            input_values::map_range(&mapper, range)
        );
    }

    #[test]
    #[should_panic(expected = "Broken invariant: ids must be skipped in order")]
    fn skip_out_of_order() {
        let mut mapper = IdMapper::default();
        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(5));
        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(3));
        // boom
    }

    #[test]
    #[should_panic(expected = "Broken invariant: ids must be skipped in order")]
    fn skip_twice() {
        let mut mapper = IdMapper::default();
        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(5));
        input_values::skip(&mut mapper, federated_graph::InputValueDefinitionId(5));
        // boom
    }
}
