use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Partition<Id> {
    One(Id),
    Many(Vec<Id>),
}

impl<Id> From<Id> for Partition<Id> {
    fn from(id: Id) -> Self {
        Partition::One(id)
    }
}

impl<Id: Copy> From<Vec<Id>> for Partition<Id> {
    fn from(ids: Vec<Id>) -> Self {
        match ids[..] {
            [] => unreachable!(),
            [id] => Partition::One(id),
            _ => Partition::Many(ids),
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
#[allow(dead_code)]
pub(super) fn partition_shapes<Id: Copy + Ord + std::fmt::Debug>(
    // Must be sorted and unique
    selection_set_possible_types: &[Id],
    // Must be sorted and unique
    mut objects: Vec<Id>,
    // Individual possible types must be sorted and unique
    // Arrays may not be unique, but they should as much as possible.
    mut type_conditions_possible_types: Vec<&[Id]>,
) -> Option<Vec<Partition<Id>>> {
    if selection_set_possible_types.len() == 1 {
        return None;
    }

    // Treat any array with a single element into single_object_ids
    let mut i = 0;
    while i < type_conditions_possible_types.len() {
        if type_conditions_possible_types[i].len() == 1 {
            let id = type_conditions_possible_types.swap_remove(i)[0];
            if let Err(i) = objects.binary_search(&id) {
                objects.insert(i, id);
            }
        } else {
            i += 1;
        }
    }

    // All possible cases are defined as objects
    if objects.len() == selection_set_possible_types.len() || type_conditions_possible_types.is_empty() {
        return Some(objects.into_iter().map(Into::into).collect());
    }

    let selection_set_possible_types = &difference(selection_set_possible_types, &objects);

    // Detect supersets of the output, they're all treated the same way.
    let (supersets, type_conditions_possible_types): (Vec<_>, Vec<_>) = type_conditions_possible_types
        .into_iter()
        .partition(|possible_types| is_subset_of(selection_set_possible_types, possible_types));

    // If there are only supersets, no need for any partitions.
    if objects.is_empty() && type_conditions_possible_types.is_empty() {
        return None;
    }

    // From here we exhausted all "simple" cases, we need to create partitions. Any type condition
    // left only applies to some of the output.
    let mut output = objects.iter().copied().map(Into::into).collect::<Vec<_>>();

    if !type_conditions_possible_types.is_empty() {
        let partitions = split_into_partitions(selection_set_possible_types, &type_conditions_possible_types);
        if !supersets.is_empty() {
            output.push(compute_remaining_partition(
                selection_set_possible_types,
                // using subsets as excluded list here as it's equivalent to the type deduplicated
                // type conditions.
                partitions.iter().map(Vec::as_slice),
            ));
        }

        output.extend(partitions.into_iter().map(Into::into));
    } else if !supersets.is_empty() {
        output.push(compute_remaining_partition(
            selection_set_possible_types,
            type_conditions_possible_types,
        ));
    }

    Some(output)
}

// a \ b
// b is assumed to be included in a.
fn difference<'a, Id: Copy + Ord>(a: &'a [Id], b: &[Id]) -> Cow<'a, [Id]> {
    if b.is_empty() {
        return Cow::Borrowed(a);
    }
    let Ok(mut i) = a.binary_search(&b[0]) else {
        return Cow::Borrowed(a);
    };
    let mut diff = Vec::with_capacity(a.len());
    diff.extend_from_slice(&a[..i]);
    i += 1;
    let mut j = 1;
    while let Some((left, right)) = a.get(i).copied().zip(b.get(j)) {
        match left.cmp(right) {
            std::cmp::Ordering::Less => {
                diff.push(left);
                i += 1;
            }
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }
    diff.extend_from_slice(&a[i..]);
    Cow::Owned(diff)
}

// Do we have a ⊂ b ?
fn is_subset_of<Id: Copy + Ord>(a: &[Id], b: &[Id]) -> bool {
    if a.len() > b.len() {
        return false;
    }
    let mut i = 0;
    let mut j = 0;

    while let Some((left, right)) = a.get(i).zip(b.get(j)) {
        match left.cmp(right) {
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    i == a.len()
}

// Split selection_set_possible_types into partitions for which a unique
// combination of type conditions applies.
fn split_into_partitions<Id: Copy + Ord + std::fmt::Debug>(
    selection_set_possible_types: &[Id],
    type_conditions_possible_types: &[&[Id]],
) -> Vec<Vec<Id>> {
    let mut partitions: Vec<Vec<Id>> = Vec::new();

    // Re-using those Vec
    let mut remaining = Vec::new();
    let mut intersection = Vec::new();

    for possible_types in type_conditions_possible_types {
        remaining.clear();

        // remaining = (possible_types ∩ selection_set_possible_types)
        let mut i = 0;
        let mut j = 0;
        while let Some((left, right)) = possible_types.get(i).zip(selection_set_possible_types.get(j)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    remaining.push(*left);
                    i += 1;
                    j += 1;
                }
            }
        }

        for i in 0..partitions.len() {
            if remaining.is_empty() {
                break;
            }
            intersection.clear();
            extract_intersection(&mut partitions[i], &mut remaining, &mut intersection);
            // if existing was a subset of candidate, we don't generate a common and just removed elements
            // from the candidate. This avoids the need to check the existing partition size.
            if partitions[i].is_empty() {
                std::mem::swap(&mut partitions[i], &mut intersection);
            } else if !intersection.is_empty() {
                partitions.push(intersection.clone());
            }
        }

        if !remaining.is_empty() {
            partitions.push(remaining.clone());
        }
    }

    partitions
}

// Both arrays are sorted an non-empty.
fn extract_intersection<Id: Copy + Ord>(existing: &mut Vec<Id>, candidate: &mut Vec<Id>, intersection: &mut Vec<Id>) {
    let mut i = 0;
    let mut j = 0;
    let mut new_existing_len = 0;
    let mut new_candidate_len = 0;

    if existing[existing.len() - 1] < candidate[0] || candidate[candidate.len() - 1] < existing[0] {
        return;
    }

    while let Some((left, right)) = existing.get(i).copied().zip(candidate.get(j).copied()) {
        match left.cmp(&right) {
            std::cmp::Ordering::Less => {
                existing[new_existing_len] = left;
                new_existing_len += 1;
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                candidate[new_candidate_len] = right;
                new_candidate_len += 1;
                j += 1
            }
            std::cmp::Ordering::Equal => {
                intersection.push(left);
                i += 1;
                j += 1;
            }
        }
    }

    if i != new_existing_len {
        while let Some(left) = existing.get(i).copied() {
            existing[new_existing_len] = left;
            new_existing_len += 1;
            i += 1;
        }
        existing.truncate(new_existing_len);
    }

    if j != new_candidate_len {
        while let Some(right) = candidate.get(j).copied() {
            candidate[new_candidate_len] = right;
            new_candidate_len += 1;
            j += 1;
        }

        candidate.truncate(new_candidate_len);
    }
}

// output_ids \ single_object_ids \ excluded_list
fn compute_remaining_partition<'a, Id: Copy + Ord>(
    selection_set_possible_types: &'a [Id],
    subsets: impl IntoIterator<Item = &'a [Id]>,
) -> Partition<Id> {
    let mut remaining = Cow::Borrowed(selection_set_possible_types);
    let mut tmp = Vec::new();

    for subset in subsets {
        let mut i = 0;
        let mut j = 0;

        // Detect the first common element if there is any before copying anything.
        while let Some((left, right)) = remaining.get(i).zip(subset.get(j)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => {
                    i += 1;
                }
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    break;
                }
            }
        }

        // No common element
        if i == remaining.len() || j == subset.len() {
            continue;
        }

        tmp.clear();
        tmp.extend_from_slice(&remaining[..i]);

        while let Some((left, right)) = remaining.get(i).zip(subset.get(j)) {
            match left.cmp(right) {
                std::cmp::Ordering::Less => {
                    tmp.push(*left);
                    i += 1;
                }
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            }
        }

        tmp.extend_from_slice(&remaining[i..]);
        if let Cow::Owned(ref mut owned) = remaining {
            std::mem::swap(owned, &mut tmp);
        } else {
            remaining = Cow::Owned(std::mem::take(&mut tmp));
        }
    }

    remaining.into_owned().into()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_difference() {
        assert_eq!(difference(&[1, 2, 3], &[]).into_owned(), vec![1, 2, 3]);
        assert_eq!(difference(&[1, 2, 3], &[2]).into_owned(), vec![1, 3]);
    }

    #[test]
    fn test_remaining_output_is_subset_of_possible_types() {
        assert!(is_subset_of::<usize>(&[], &[]));
        assert!(is_subset_of(&[1], &[1]));
        assert!(is_subset_of(&[1, 2], &[1, 2, 3]));

        assert!(!is_subset_of(&[1], &[]));
        assert!(!is_subset_of(&[1, 2], &[2, 3, 4]));
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
            split_into_partitions(&[1, 2, 3, 4, 5, 6], &[&[1, 2, 3], &[3, 4], &[5, 6]]),
            vec![vec![1, 2], vec![3], vec![4], vec![5, 6]]
        );
        assert_eq!(
            split_into_partitions(&[3, 4, 5, 6], &[&[1, 2, 3], &[3, 4], &[5, 6]]),
            vec![vec![3], vec![4], vec![5, 6]]
        );
    }

    #[test]
    fn test_compute_remaining_subset() {
        assert_eq!(
            compute_remaining_partition(&[1, 2, 3, 4, 5, 6], [vec![3, 4], vec![5, 6]].iter().map(Vec::as_slice)),
            Partition::many(vec![1, 2])
        );
        assert_eq!(
            compute_remaining_partition(&[2, 3, 4, 5, 6], [vec![3, 4], vec![5, 6]].iter().map(Vec::as_slice)),
            Partition::one(2)
        );
    }

    impl Partition<usize> {
        fn one(id: usize) -> Self {
            Partition::One(id)
        }

        fn many(ids: impl IntoIterator<Item = usize>) -> Self {
            Partition::Many(ids.into_iter().collect())
        }
    }

    struct TestCase {
        output: Vec<usize>,
        single_object_ids: Vec<usize>,
        type_conditions: Vec<Vec<usize>>,
    }

    fn given_output(output: impl IntoIterator<Item = usize>) -> TestCase {
        TestCase {
            output: output.into_iter().map(Into::into).collect(),
            single_object_ids: Vec::new(),
            type_conditions: Vec::new(),
        }
    }

    impl TestCase {
        fn with_objects(mut self, ids: impl IntoIterator<Item = usize>) -> Self {
            self.single_object_ids.extend(ids);
            self
        }

        fn with_type_condition(mut self, type_condition: impl IntoIterator<Item = usize>) -> Self {
            self.type_conditions.push(type_condition.into_iter().collect());
            self
        }

        #[track_caller]
        fn expect_none(self) {
            assert!(!self.type_conditions.is_empty() || !self.single_object_ids.is_empty());
            assert_eq!(
                None,
                partition_shapes(
                    &self.output,
                    self.single_object_ids,
                    self.type_conditions.iter().map(Vec::as_slice).collect()
                ),
            );
        }

        #[track_caller]
        fn expect_partitions(self, expected: impl IntoIterator<Item = Partition<usize>>) {
            assert!(!self.type_conditions.is_empty() || !self.single_object_ids.is_empty());
            assert_eq!(
                Some(expected.into_iter().collect::<BTreeSet<_>>()),
                partition_shapes(
                    &self.output,
                    self.single_object_ids,
                    self.type_conditions.iter().map(Vec::as_slice).collect()
                )
                .map(|res| res.into_iter().collect::<BTreeSet<_>>())
            );
        }
    }

    #[test]
    fn only_objects() {
        given_output([1]).with_objects([1]).expect_none();

        given_output([1, 2])
            .with_objects([1, 2])
            .expect_partitions([Partition::one(1), Partition::one(2)]);

        given_output([1, 2])
            .with_objects([1])
            .expect_partitions([Partition::one(1)]);
    }

    #[test]
    fn single_interface() {
        given_output([1]).with_type_condition([1, 2]).expect_none();
        given_output([1, 2]).with_type_condition([1, 2]).expect_none();

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .expect_partitions([Partition::many([1, 2])]);

        given_output([1, 47])
            .with_type_condition([1, 2])
            .expect_partitions([Partition::one(1)]);
    }

    #[test]
    fn single_interface_with_objects() {
        given_output([1])
            .with_type_condition([1, 2])
            .with_objects([1])
            .expect_none();

        given_output([1, 2])
            .with_type_condition([1, 2])
            .with_objects([2])
            .expect_partitions([Partition::one(2), Partition::one(1)]);

        given_output([1, 2])
            .with_type_condition([1, 2])
            .with_objects([1, 2])
            .expect_partitions([Partition::one(1), Partition::one(2)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .with_objects([2])
            .expect_partitions([Partition::one(2), Partition::one(1)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .with_objects([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(3), Partition::one(1)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .with_objects([3])
            .expect_partitions([Partition::one(3), Partition::many([1, 2])]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_objects([3])
            .expect_partitions([Partition::one(3), Partition::many([1, 2])]);
    }

    #[test]
    fn two_disjoint_interfaces() {
        given_output([2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .expect_partitions([Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .expect_partitions([Partition::many([1, 2]), Partition::many([3, 4])]);

        given_output([1, 2, 3, 4, 5])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .expect_partitions([Partition::many([1, 2]), Partition::many([3, 4])]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .expect_partitions([Partition::one(2), Partition::one(3)]);
    }

    #[test]
    fn two_disjoint_interfaces_with_object() {
        given_output([2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .with_objects([2])
            .expect_partitions([Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .with_objects([4])
            .expect_partitions([Partition::one(4), Partition::many([1, 2]), Partition::one(3)]);

        given_output([1, 2, 3, 4, 5, 6])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .with_objects([5])
            .expect_partitions([Partition::one(5), Partition::many([1, 2]), Partition::many([3, 4])]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2])
            .with_type_condition([3, 4])
            .expect_partitions([Partition::one(2), Partition::one(3)]);
    }

    #[test]
    fn two_interfaces_having_a_common_object() {
        given_output([2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(1), Partition::one(3)]);

        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(1), Partition::one(3)]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(3)]);
    }

    #[test]
    fn two_interfaces_having_a_common_object_with_object() {
        given_output([2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .with_objects([2])
            .expect_partitions([Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .with_objects([3])
            .expect_partitions([Partition::one(3), Partition::one(2), Partition::one(1)]);

        given_output([1, 2, 3, 4, 5])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .with_objects([4])
            .expect_partitions([
                Partition::one(4),
                Partition::one(2),
                Partition::one(1),
                Partition::one(3),
            ]);

        given_output([2, 3, 47])
            .with_type_condition([1, 2])
            .with_type_condition([2, 3])
            .expect_partitions([Partition::one(2), Partition::one(3)]);
    }

    #[test]
    fn small_and_big_interface() {
        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_type_condition([1, 2, 3, 4, 5, 6])
            .expect_partitions([Partition::many([1, 2]), Partition::many([3, 4])]);

        given_output([2, 3, 4, 57])
            .with_type_condition([1, 2])
            .with_type_condition([1, 2, 3, 4, 5, 6])
            .expect_partitions([Partition::one(2), Partition::many([3, 4])]);
    }

    #[test]
    fn all_cases_specified_by_an_objects() {
        given_output([1, 2, 3, 4])
            .with_type_condition([1, 2])
            .with_type_condition([1, 2, 3, 4, 5, 6])
            .with_objects([1, 2, 3, 4])
            .expect_partitions([
                Partition::one(1),
                Partition::one(2),
                Partition::one(3),
                Partition::one(4),
            ]);
    }

    #[test]
    fn only_interfaces_matching_all_possible_types() {
        given_output([2, 3, 4])
            .with_type_condition([1, 2, 3, 4])
            .with_type_condition([1, 2, 3, 4, 5, 6])
            .expect_none();
    }

    #[test]
    fn multiple_complex_interfaces() {
        given_output([1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12])
            .with_type_condition([1, 2, 3, 4])
            .with_type_condition([5, 6, 7, 8])
            .with_type_condition([9, 10, 11, 12])
            .with_type_condition([3, 7, 11, 17])
            .with_type_condition([3, 10, 12, 18])
            .expect_partitions([
                Partition::one(3),
                Partition::one(7),
                Partition::one(11),
                Partition::many([10, 12]),
                Partition::many([1, 2, 4]),
                Partition::many([5, 6]),
                Partition::one(9),
            ])
    }

    #[test]
    fn interfaces_with_single_object() {
        given_output([1, 2])
            .with_objects([1, 2])
            .with_type_condition([1])
            .with_type_condition([2])
            .with_type_condition([3])
            .expect_partitions([Partition::one(1), Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3])
            .with_objects([1, 2])
            .with_type_condition([1])
            .with_type_condition([2])
            .with_type_condition([3])
            .expect_partitions([Partition::one(1), Partition::one(2), Partition::one(3)]);

        given_output([1, 2, 3, 4])
            .with_objects([1, 2])
            .with_type_condition([1])
            .with_type_condition([2])
            .with_type_condition([3])
            .expect_partitions([Partition::one(1), Partition::one(2), Partition::one(3)])
    }
}
