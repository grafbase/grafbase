use std::{borrow::Cow, ops::BitOrAssign};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Partition<Id, FieldsBitSet> {
    One { id: Id, fields: FieldsBitSet },
    Many { ids: Vec<Id>, fields: FieldsBitSet },
    // Should always be last
    Remaining { fields: FieldsBitSet },
}

pub(super) struct Partitioning<Id, FieldsBitSet> {
    pub partition_object_count: usize,
    pub partitions: Vec<Partition<Id, FieldsBitSet>>,
}

impl<Id, FieldsBitSet> Default for Partitioning<Id, FieldsBitSet> {
    fn default() -> Self {
        Self {
            partition_object_count: 0,
            partitions: Vec::new(),
        }
    }
}

/// For selection sets with type conditions, we must know the concrete object id being
/// resolved to apply correctly the field collection as defined in the specification:
/// https://spec.graphql.org/October2021/#CollectFields()
///
/// However, we're building the expect shape of the subgraph response in advance to drive the
/// de-serialization, so we don't have it! So we need to pre-compute all possible shapes and
/// use the right one later.
///
/// This function will create a partition set of the selection set possible types that guarantees
/// the following:
/// - Each partition is disjoint.
/// - All types in a partition have the same fields selected.
///
/// All inputs are expected to be sorted and unique. Generics are only used for easier testing.
///
pub(super) fn partition_object_shapes<Id, FieldsBitSet>(
    // Must be sorted and unique
    output_possible_types: &[Id],
    // Individual possible types must be sorted and unique
    // Arrays may not be unique, but they should as much as possible.
    type_conditions: Vec<(Cow<'_, [Id]>, FieldsBitSet)>,
) -> Partitioning<Id, FieldsBitSet>
where
    Id: Copy + Ord + std::fmt::Debug,
    FieldsBitSet: Default + Clone + BitOrAssign + for<'a> std::ops::BitOrAssign<&'a FieldsBitSet>,
    for<'a> &'a FieldsBitSet: std::ops::BitOr<Output = FieldsBitSet>,
{
    if output_possible_types.len() == 1 {
        return Default::default();
    }

    // Detect supersets of the output, they're all treated the same way.
    let (supersets, mut type_conditions): (Vec<_>, Vec<_>) = type_conditions
        .into_iter()
        .partition(|possible_types| is_superset_of_output_possible_types(&possible_types.0, output_possible_types));

    // If there are only supersets, no need for any partitions.
    if type_conditions.is_empty() {
        return Default::default();
    }

    let has_supersets = !supersets.is_empty();
    let supersets_fields = supersets
        .into_iter()
        .map(|(_, fields_bitset)| fields_bitset)
        .reduce(|mut a, b| {
            a |= b;
            a
        })
        .unwrap_or_default();

    let mut partitioning =
        if type_conditions.iter().all(|(ids, _)| ids.len() == 1) {
            type_conditions.sort_unstable_by(|a, b| a.0[0].cmp(&b.0[0]));
            let mut partitions = Vec::with_capacity(type_conditions.len() + 1);
            partitions.extend(type_conditions.into_iter().chunk_by(|(ids, _)| ids[0]).into_iter().map(
                |(id, chunks)| {
                    let fields =
                        chunks
                            .map(|(_, fields_bitset)| fields_bitset)
                            .fold(supersets_fields.clone(), |mut a, b| {
                                a |= b;
                                a
                            });

                    Partition::One { id, fields }
                },
            ));
            let partition_object_count = partitions.len();
            Partitioning {
                partition_object_count,
                partitions,
            }
        } else {
            let raw_partitions = split_into_partitions(output_possible_types, &type_conditions, &supersets_fields);
            let mut partition_object_count = 0;
            let mut partitions = Vec::with_capacity(raw_partitions.len() + 1);
            partitions.extend(raw_partitions.into_iter().filter_map(|(ids, fields)| match ids[..] {
                [] => None,
                [id] => {
                    partition_object_count += 1;
                    Some(Partition::One { id, fields })
                }
                _ => {
                    partition_object_count += ids.len();
                    Some(Partition::Many { ids, fields })
                }
            }));
            Partitioning {
                partition_object_count,
                partitions,
            }
        };

    if (partitioning.partition_object_count != output_possible_types.len()) && has_supersets {
        partitioning.partitions.push(Partition::Remaining {
            fields: supersets_fields,
        });
    }

    partitioning
}

// Do we have output_possible_types ⊂ type_condition_possible_types ?
fn is_superset_of_output_possible_types<Id: Copy + Ord>(
    type_condition_possible_types: &[Id],
    output_possible_types: &[Id],
) -> bool {
    if output_possible_types.len() > type_condition_possible_types.len() {
        return false;
    }
    let mut l = 0;
    let mut r = 0;

    while let Some((left, right)) = output_possible_types.get(l).zip(type_condition_possible_types.get(r)) {
        match left.cmp(right) {
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Greater => r += 1,
            std::cmp::Ordering::Equal => {
                l += 1;
                r += 1;
            }
        }
    }

    l == output_possible_types.len()
}

// Split selection_set_possible_types into partitions for which a unique
// combination of type conditions applies.
fn split_into_partitions<Id, FieldsBitSet>(
    output_possible_types: &[Id],
    type_conditions: &[(Cow<'_, [Id]>, FieldsBitSet)],
    supersets_fields: &FieldsBitSet,
) -> Vec<(Vec<Id>, FieldsBitSet)>
where
    Id: Copy + Ord + std::fmt::Debug,
    FieldsBitSet: Clone + for<'a> std::ops::BitOrAssign<&'a FieldsBitSet>,
    for<'a> &'a FieldsBitSet: std::ops::BitOr<Output = FieldsBitSet>,
{
    let mut partitions: Vec<(Vec<Id>, FieldsBitSet)> = Vec::new();

    // Re-using those Vec
    let mut new_partition = Vec::new();
    let mut intersection = Vec::new();

    for (possible_types, fields) in type_conditions {
        new_partition.clear();

        // Initialize the new partition to the intersection with the output possible types
        // new_partition = (possible_types ∩ output_possible_types)
        let mut l = 0;
        let mut r = 0;
        while let Some((left, right)) = possible_types.get(l).zip(output_possible_types.get(r)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => l += 1,
                std::cmp::Ordering::Greater => r += 1,
                std::cmp::Ordering::Equal => {
                    new_partition.push(*left);
                    l += 1;
                    r += 1;
                }
            }
        }

        for i in 0..partitions.len() {
            if new_partition.is_empty() {
                break;
            }
            intersection.clear();
            extract_intersection(&mut partitions[i].0, &mut new_partition, &mut intersection);
            // if existing was a subset of candidate, we don't generate a common and just removed elements
            // from the candidate. This avoids the need to check the existing partition size.
            if partitions[i].0.is_empty() {
                std::mem::swap(&mut partitions[i].0, &mut intersection);
                partitions[i].1 |= fields;
            } else if !intersection.is_empty() {
                let mut fields = fields | &partitions[i].1;
                fields |= supersets_fields;
                partitions.push((intersection.clone(), fields));
            }
        }

        if !new_partition.is_empty() {
            let mut fields = fields.clone();
            fields |= supersets_fields;
            partitions.push((new_partition.clone(), fields));
        }
    }

    partitions
}

// Both arrays are sorted an non-empty.
fn extract_intersection<Id: Copy + Ord>(left: &mut Vec<Id>, right: &mut Vec<Id>, intersection: &mut Vec<Id>) {
    let mut l = 0;
    let mut r = 0;
    let mut left_len = 0;
    let mut right_len = 0;

    if left[left.len() - 1] < right[0] || right[right.len() - 1] < left[0] {
        return;
    }

    while let Some((left_id, right_id)) = left.get(l).copied().zip(right.get(r).copied()) {
        match left_id.cmp(&right_id) {
            std::cmp::Ordering::Less => {
                left[left_len] = left_id;
                left_len += 1;
                l += 1;
            }
            std::cmp::Ordering::Greater => {
                right[right_len] = right_id;
                right_len += 1;
                r += 1
            }
            std::cmp::Ordering::Equal => {
                intersection.push(left_id);
                l += 1;
                r += 1;
            }
        }
    }

    if l != left_len {
        while let Some(id) = left.get(l).copied() {
            left[left_len] = id;
            left_len += 1;
            l += 1;
        }
        left.truncate(left_len);
    }

    if r != right_len {
        while let Some(id) = right.get(r).copied() {
            right[right_len] = id;
            right_len += 1;
            r += 1;
        }

        right.truncate(right_len);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_remaining_output_is_subset_of_possible_types() {
        assert!(is_superset_of_output_possible_types::<usize>(&[], &[]));
        assert!(is_superset_of_output_possible_types(&[1], &[1]));
        assert!(is_superset_of_output_possible_types(&[1, 2, 3], &[1, 2]));

        assert!(!is_superset_of_output_possible_types(&[], &[1]));
        assert!(!is_superset_of_output_possible_types(&[2, 3, 4, 5], &[1, 2, 3]));
    }

    #[test]
    fn test_extract_intersection() {
        let mut existing = vec![1, 2, 3, 4, 5];
        let mut candidate = vec![3, 4, 5, 6, 7];
        let mut intersection = Vec::new();
        extract_intersection(&mut existing, &mut candidate, &mut intersection);
        assert_eq!(existing, vec![1, 2]);
        assert_eq!(candidate, vec![6, 7]);
        assert_eq!(intersection, vec![3, 4, 5]);

        let mut existing = vec![1, 2, 3, 4, 5];
        let mut candidate = vec![6, 7];
        let mut intersection = Vec::new();
        extract_intersection(&mut existing, &mut candidate, &mut intersection);
        assert_eq!(existing, vec![1, 2, 3, 4, 5]);
        assert_eq!(candidate, vec![6, 7]);
        assert!(intersection.is_empty());

        let mut existing = vec![1, 2, 3, 4, 5];
        let mut candidate = vec![1, 2, 3, 4, 5];
        let mut intersection = Vec::new();
        extract_intersection(&mut existing, &mut candidate, &mut intersection);
        assert!(existing.is_empty());
        assert!(candidate.is_empty());
        assert_eq!(intersection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_split_into_distinct_ouput_subsets() {
        assert_eq!(
            split_into_partitions(
                &[1, 2, 3, 4, 5, 6],
                &[
                    (vec![1, 2, 3].into(), 0b0010),
                    (vec![3, 4].into(), 0b0100),
                    (vec![5, 6].into(), 0b1000)
                ],
                &0b0001
            ),
            vec![
                (vec![1, 2], 0b0011),
                (vec![3], 0b0111),
                (vec![4], 0b0101),
                (vec![5, 6], 0b1001)
            ]
        );
        assert_eq!(
            split_into_partitions(
                &[3, 4, 5, 6],
                &[
                    (vec![1, 2, 3].into(), 0b0010),
                    (vec![3, 4].into(), 0b0100),
                    (vec![5, 6].into(), 0b1000)
                ],
                &0b0001
            ),
            vec![(vec![3], 0b0111), (vec![4], 0b0101), (vec![5, 6], 0b1001)]
        );
    }

    impl Partition<usize, usize> {
        fn one(id: usize, fields: usize) -> Self {
            Partition::One { id, fields }
        }

        fn many(ids: impl IntoIterator<Item = usize>, fields: usize) -> Self {
            Partition::Many {
                ids: ids.into_iter().collect(),
                fields,
            }
        }

        fn remaining(fields: usize) -> Self {
            Partition::Remaining { fields }
        }
    }

    struct TestCase {
        output: Vec<usize>,
        type_conditions: Vec<(Vec<usize>, usize)>,
    }

    fn given_output(output: impl IntoIterator<Item = usize>) -> TestCase {
        TestCase {
            output: output.into_iter().collect(),
            type_conditions: Vec::new(),
        }
    }

    impl TestCase {
        fn with_object(mut self, id: usize, fields: usize) -> Self {
            self.type_conditions.push((vec![id], fields));
            self
        }

        fn with_objects(mut self, ids: impl IntoIterator<Item = (usize, usize)>) -> Self {
            self.type_conditions
                .extend(ids.into_iter().map(|(id, fields)| (vec![id], fields)));
            self
        }

        fn with_type_condition(mut self, type_condition: impl IntoIterator<Item = usize>, fields: usize) -> Self {
            self.type_conditions
                .push((type_condition.into_iter().collect(), fields));
            self
        }

        #[track_caller]
        fn expect_none(self) {
            assert!(!self.type_conditions.is_empty());
            let actual = partition_object_shapes(
                &self.output,
                self.type_conditions
                    .iter()
                    .map(|(ids, fields)| (ids.as_slice().into(), *fields))
                    .collect::<Vec<_>>(),
            );
            let actual_partitions = actual
                .partitions
                .into_iter()
                .map(format_partition)
                .collect::<BTreeSet<_>>();
            assert!(actual_partitions.is_empty(), "{actual_partitions:#?}");
            assert_eq!(0, actual.partition_object_count);
        }

        #[track_caller]
        fn expect_partitions(self, expected: impl IntoIterator<Item = Partition<usize, usize>>) {
            assert!(!self.type_conditions.is_empty());
            let expected = expected.into_iter().map(format_partition).collect::<BTreeSet<_>>();
            let expected_partition_object_count: usize = expected
                .iter()
                .map(|partition| match partition {
                    Partition::One { .. } => 1,
                    Partition::Many { ids, .. } => ids.len(),
                    _ => 0,
                })
                .sum();
            let actual = partition_object_shapes(
                &self.output,
                self.type_conditions
                    .iter()
                    .map(|(ids, fields)| (ids.as_slice().into(), *fields))
                    .collect::<Vec<_>>(),
            );
            let actual_partitions = actual
                .partitions
                .into_iter()
                .map(format_partition)
                .collect::<BTreeSet<_>>();
            assert_eq!(expected, actual_partitions);
            assert_eq!(expected_partition_object_count, actual.partition_object_count);
        }
    }

    fn format_partition(p: Partition<usize, usize>) -> Partition<usize, String> {
        match p {
            Partition::One { id, fields } => Partition::One {
                id,
                fields: format!("{fields:b}"),
            },
            Partition::Many { ids, fields } => Partition::Many {
                ids,
                fields: format!("{fields:b}"),
            },
            Partition::Remaining { fields } => Partition::Remaining {
                fields: format!("{fields:b}"),
            },
        }
    }

    #[test]
    fn only_objects() {
        given_output([1]).with_objects([(1, 0b0)]).expect_none();

        given_output([1, 2])
            .with_objects([(1, 0b01), (2, 0b10)])
            .expect_partitions([Partition::one(1, 0b01), Partition::one(2, 0b10)]);

        given_output([1, 2])
            .with_object(1, 0b0)
            .expect_partitions([Partition::one(1, 0b0)]);
    }

    #[test]
    fn single_interface() {
        given_output([1]).with_type_condition([1, 2], 0b0).expect_none();
        given_output([1, 2]).with_type_condition([1, 2], 0b0).expect_none();

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b01)
            .expect_partitions([Partition::many([1, 2], 0b01)]);

        given_output([1, 47])
            .with_type_condition([1, 2], 0b01)
            .expect_partitions([Partition::one(1, 0b01)]);
    }

    #[test]
    fn single_interface_with_objects() {
        given_output([1])
            .with_type_condition([1, 2], 0b01)
            .with_object(1, 0b10)
            .expect_none();

        given_output([1, 2])
            .with_type_condition([1, 2], 0b01)
            .with_object(2, 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::remaining(0b01)]);

        given_output([1, 2])
            .with_type_condition([1, 2], 0b001)
            .with_object(1, 0b010)
            .with_object(2, 0b100)
            .expect_partitions([Partition::one(1, 0b011), Partition::one(2, 0b101)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b01)
            .with_object(2, 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::one(1, 0b01)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b001)
            .with_object(2, 0b010)
            .with_object(3, 0b100)
            .expect_partitions([
                Partition::one(1, 0b001),
                Partition::one(2, 0b011),
                Partition::one(3, 0b100),
            ]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b01)
            .with_object(3, 0b10)
            .expect_partitions([Partition::one(3, 0b10), Partition::many([1, 2], 0b01)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b01)
            .with_object(3, 0b10)
            .expect_partitions([Partition::one(3, 0b10), Partition::many([1, 2], 0b01)]);
    }

    #[test]
    fn two_disjoint_interfaces() {
        given_output([2, 3])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([3, 4], 0b10)
            .expect_partitions([Partition::one(2, 0b01), Partition::one(3, 0b10)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([3, 4], 0b10)
            .expect_partitions([Partition::many([1, 2], 0b01), Partition::many([3, 4], 0b10)]);

        given_output([1, 2, 3, 4, 5])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([3, 4], 0b10)
            .expect_partitions([Partition::many([1, 2], 0b01), Partition::many([3, 4], 0b10)]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([3, 4], 0b10)
            .expect_partitions([Partition::one(2, 0b01), Partition::one(3, 0b10)]);
    }

    #[test]
    fn two_disjoint_interfaces_with_object() {
        given_output([2, 3])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([3, 4], 0b010)
            .with_object(2, 0b100)
            .expect_partitions([Partition::one(2, 0b101), Partition::one(3, 0b010)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([3, 4], 0b010)
            .with_object(4, 0b100)
            .expect_partitions([
                Partition::one(4, 0b110),
                Partition::many([1, 2], 0b001),
                Partition::one(3, 0b010),
            ]);

        given_output([1, 2, 3, 4, 5, 6])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([3, 4], 0b010)
            .with_object(5, 0b100)
            .expect_partitions([
                Partition::one(5, 0b100),
                Partition::many([1, 2], 0b001),
                Partition::many([3, 4], 0b010),
            ]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([3, 4], 0b10)
            .expect_partitions([Partition::one(2, 0b01), Partition::one(3, 0b10)]);
    }

    #[test]
    fn two_interfaces_having_a_common_object() {
        given_output([2, 3])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([2, 3], 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::remaining(0b10)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([2, 3], 0b10)
            .expect_partitions([
                Partition::one(2, 0b11),
                Partition::one(1, 0b01),
                Partition::one(3, 0b10),
            ]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([2, 3], 0b10)
            .expect_partitions([
                Partition::one(2, 0b11),
                Partition::one(1, 0b01),
                Partition::one(3, 0b10),
            ]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([2, 3], 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::one(3, 0b10)]);
    }

    #[test]
    fn two_interfaces_having_a_common_object_with_object() {
        given_output([2, 3])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([2, 3], 0b010)
            .with_object(2, 0b100)
            .expect_partitions([Partition::one(2, 0b111), Partition::remaining(0b010)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([2, 3], 0b010)
            .with_object(3, 0b100)
            .expect_partitions([
                Partition::one(3, 0b110),
                Partition::one(2, 0b011),
                Partition::one(1, 0b001),
            ]);

        given_output([1, 2, 3, 4, 5])
            .with_type_condition([1, 2], 0b001)
            .with_type_condition([2, 3], 0b010)
            .with_object(4, 0b100)
            .expect_partitions([
                Partition::one(4, 0b100),
                Partition::one(2, 0b011),
                Partition::one(1, 0b001),
                Partition::one(3, 0b010),
            ]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([2, 3], 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::one(3, 0b10)]);
    }

    #[test]
    fn small_and_big_interface() {
        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([1, 2, 3, 4, 5, 6], 0b10)
            .expect_partitions([Partition::many([1, 2], 0b11), Partition::remaining(0b10)]);

        given_output([2, 3, 4, 57])
            .with_type_condition([1, 2], 0b01)
            .with_type_condition([1, 2, 3, 4, 5, 6], 0b10)
            .expect_partitions([Partition::one(2, 0b11), Partition::many([3, 4], 0b10)]);
    }

    #[test]
    fn all_cases_specified_by_an_objects() {
        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2], 0b00_0001)
            .with_type_condition([1, 2, 3, 4, 5, 6], 0b00_0010)
            .with_object(1, 0b00_0100)
            .with_object(2, 0b00_1000)
            .with_object(3, 0b01_0000)
            .with_object(4, 0b10_0000)
            .expect_partitions([
                Partition::one(1, 0b00_0111),
                Partition::one(2, 0b00_1011),
                Partition::one(3, 0b01_0010),
                Partition::one(4, 0b10_0010),
            ]);
    }

    #[test]
    fn only_interfaces_matching_all_possible_types() {
        given_output([2, 3, 4])
            .with_type_condition([1, 2, 3, 4], 0b01)
            .with_type_condition([1, 2, 3, 4, 5, 6], 0b10)
            .expect_none();
    }

    #[test]
    fn multiple_complex_interfaces() {
        given_output([1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12])
            .with_type_condition([1, 2, 3, 4], 0b0_0001)
            .with_type_condition([5, 6, 7, 8], 0b0_0010)
            .with_type_condition([9, 10, 11, 12], 0b0_0100)
            .with_type_condition([3, 7, 11, 17], 0b0_1000)
            .with_type_condition([3, 10, 12, 18], 0b1_0000)
            .expect_partitions([
                Partition::one(3, 0b1_1001),
                Partition::one(7, 0b0_1010),
                Partition::one(11, 0b0_1100),
                Partition::many([10, 12], 0b1_0100),
                Partition::many([1, 2, 4], 0b0_0001),
                Partition::many([5, 6], 0b0_0010),
                Partition::one(9, 0b0_0100),
            ])
    }

    #[test]
    fn interfaces_with_single_object() {
        given_output([1, 2])
            .with_object(1, 0b0_0001)
            .with_object(2, 0b0_0010)
            .with_type_condition([1], 0b0_0100)
            .with_type_condition([2], 0b0_1000)
            .with_type_condition([3], 0b1_0000)
            .expect_partitions([
                Partition::one(1, 0b0_0101),
                Partition::one(2, 0b0_1010),
                Partition::one(3, 0b1_0000),
            ]);

        given_output([1, 2, 3])
            .with_object(1, 0b0_0001)
            .with_object(2, 0b0_0010)
            .with_type_condition([1], 0b0_0100)
            .with_type_condition([2], 0b0_1000)
            .with_type_condition([3], 0b1_0000)
            .expect_partitions([
                Partition::one(1, 0b0_0101),
                Partition::one(2, 0b0_1010),
                Partition::one(3, 0b1_0000),
            ]);

        given_output([1, 2, 3, 4])
            .with_object(1, 0b0_0001)
            .with_object(2, 0b0_0010)
            .with_type_condition([1], 0b0_0100)
            .with_type_condition([2], 0b0_1000)
            .with_type_condition([3], 0b1_0000)
            .expect_partitions([
                Partition::one(1, 0b0_0101),
                Partition::one(2, 0b0_1010),
                Partition::one(3, 0b1_0000),
            ]);
    }
}
