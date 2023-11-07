use super::FieldEdgeId;

mod input;
mod output;

pub use input::{InputNodeSelection, InputNodeSelectionSet};
pub use output::{OutputNodeSelection, OutputNodeSelectionSet};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeSelectionSet {
    // sorted by field
    items: Vec<NodeSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSelection {
    pub field: FieldEdgeId,
    pub subselection: NodeSelectionSet,
}

impl Extend<NodeSelection> for NodeSelectionSet {
    fn extend<T: IntoIterator<Item = NodeSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<NodeSelection> for NodeSelectionSet {
    fn from_iter<T: IntoIterator<Item = NodeSelection>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field);
        Self { items }
    }
}

impl IntoIterator for NodeSelectionSet {
    type Item = NodeSelection;

    type IntoIter = <Vec<NodeSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a NodeSelectionSet {
    type Item = &'a NodeSelection;

    type IntoIter = <&'a Vec<NodeSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl From<NodeSelection> for NodeSelectionSet {
    fn from(selection: NodeSelection) -> Self {
        Self { items: vec![selection] }
    }
}

impl NodeSelectionSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &NodeSelection> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    // pub fn difference(&self, other: &NodeSelectionSet) -> Self {
    //     let mut diff = Self::empty();
    //     let mut l = 0;
    //     let mut r = 0;
    //     while l < self.items.len() && r < other.items.len() {
    //         let left = &self.items[l];
    //         let right = &other.items[r];
    //         if left.field < right.field {
    //             diff.items.push(left.clone());
    //             l += 1;
    //         } else if left.field > right.field {
    //             r += 1;
    //         } else {
    //             let subdiff = left.subselection.difference(&right.subselection);
    //             if !subdiff.is_empty() {
    //                 diff.items.push(NodeSelection {
    //                     field: left.field,
    //                     subselection: subdiff,
    //                 });
    //             }
    //             l += 1;
    //             r += 1;
    //         }
    //     }
    //     while l < self.items.len() {
    //         diff.items.push(self.items[l].clone());
    //     }
    //     diff
    // }
    //
    // pub fn extend(&mut self, other: &NodeSelectionSet) {
    //     // Yes those std::mem::replace are premature optimizaitons, had fun doing those though. :)
    //     let mut current = {
    //         let capacity = self.items.len() + other.items.len();
    //         std::mem::replace(&mut self.items, Vec::with_capacity(capacity))
    //     };
    //     let placeholder = NodeSelection {
    //         field: FieldEdgeId(0),
    //         subselection: NodeSelectionSet::default(),
    //     };
    //     let mut l = 0;
    //     let mut r = 0;
    //     while l < current.len() && r < other.items.len() {
    //         let mut left = &mut current[l];
    //         let right = &other.items[r];
    //         if left.field < right.field {
    //             self.items.push(std::mem::replace(&mut left, placeholder.clone()));
    //             l += 1;
    //         } else if left.field > right.field {
    //             self.items.push(right.clone());
    //             r += 1;
    //         } else {
    //             let mut selection: NodeSelection = std::mem::replace(&mut left, placeholder.clone());
    //             selection.subselection.extend(&right.subselection);
    //             self.items.push(selection);
    //             l += 1;
    //             r += 1;
    //         }
    //     }
    //
    //     while l < current.len() {
    //         self.items.push(std::mem::replace(&mut current[l], placeholder.clone()));
    //         l += 1;
    //     }
    //
    //     while r < other.items.len() {
    //         self.items.push(other.items[r].clone());
    //         r += 1;
    //     }
    // }
    //
    // pub fn is_disjoint(&self, other: &NodeSelectionSet) -> bool {
    //     let mut l = 0;
    //     let mut r = 0;
    //     while l < self.items.len() && r < other.items.len() {
    //         let left = &self.items[l];
    //         let right = &other.items[r];
    //         if left.field < right.field {
    //             l += 1;
    //         } else if left.field > right.field {
    //             r += 1;
    //         } else {
    //             if left.subselection.is_empty() || right.subselection.is_empty() {
    //                 return false;
    //             }
    //             if !left.subselection.is_disjoint(&right.subselection) {
    //                 return false;
    //             }
    //             l += 1;
    //             r += 1;
    //         }
    //     }
    //     true
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     macro_rules! selection_set {
//         () => {
//             NodeSelectionSet::empty()
//         };
//         ($($selection: tt)+) => {{
//             let mut set = NodeSelectionSet::empty();
//             selection_set_add!(set => $($selection)+);
//             set
//         }};
//     }
//
//     macro_rules! selection_set_add {
//         ($set:ident => $field:ident { $($subselection: tt)+ } $($rest: tt)*) => {
//             $set.items.push(NodeSelection {
//                 field: $field,
//                 subselection: selection_set!($($subselection)+)
//             });
//             selection_set_add!($set => $($rest)*);
//         };
//         ($set:ident => $field:ident $($rest: tt)*) => {
//             $set.items.push(NodeSelection {
//                 field: $field,
//                 subselection: NodeSelectionSet::empty()
//             });
//             selection_set_add!($set => $($rest)*)
//         };
//         ($set:ident => ) => {};
//     }
//
//     fn field(id: u32) -> FieldEdgeId {
//         FieldEdgeId(id)
//     }
//
//     #[test]
//     fn test_node_selection_set_extend() {
//         let a = field(0);
//         let b = field(1);
//         let c = field(2);
//         let d = field(3);
//         let e = field(4);
//         let f = field(5);
//         let g = field(6);
//
//         // disjoint
//         let mut set1 = selection_set! {
//             d
//             e { f }
//         };
//         set1.extend(&selection_set! {
//             a
//             b { c }
//         });
//         assert_eq!(
//             set1,
//             selection_set! {
//                 a
//                 b { c }
//                 d
//                 e { f }
//             }
//         );
//
//         // common field
//         let mut set2 = selection_set! { a b };
//         set2.extend(&selection_set! { b c });
//         assert_eq!(set2, selection_set! { a b c });
//
//         // nested field
//         let mut set3 = selection_set! {
//             b { d e }
//             f
//         };
//         set3.extend(&selection_set! {
//             a
//             b { c }
//         });
//         assert_eq!(
//             set3,
//             selection_set! {
//                 a
//                 b { c d e }
//                 f
//             }
//         );
//
//         // deeply nested
//         let mut set4 = selection_set! {
//             a {
//                 b {
//                     c { d e }
//                 }
//             }
//         };
//         set4.extend(&selection_set! {
//             a {
//                 b {
//                     c { f g }
//                 }
//             }
//         });
//         assert_eq!(
//             set4,
//             selection_set! {
//                 a {
//                     b {
//                         c { d e f g }
//                     }
//                 }
//             }
//         );
//
//         // one nested other not
//         let mut set5 = selection_set! {
//             a { b }
//             c
//         };
//         set5.extend(&selection_set! {
//             a
//             c { d }
//         });
//         assert_eq!(
//             set5,
//             selection_set! {
//                 a { b }
//                 c { d }
//             }
//         );
//     }
//
//     #[test]
//     fn test_node_selection_is_disjoint() {
//         let a = field(0);
//         let b = field(1);
//         let c = field(2);
//         let d = field(3);
//         let e = field(4);
//
//         assert!(selection_set! {}.is_disjoint(&selection_set! { a }));
//         assert!(selection_set! { a }.is_disjoint(&selection_set! {}));
//
//         assert!(selection_set! { a b }.is_disjoint(&selection_set! { c d }));
//         assert!(!selection_set! { a b }.is_disjoint(&selection_set! { a c }));
//         assert!(!selection_set! { a b }.is_disjoint(&selection_set! { b c }));
//
//         assert!(!selection_set! { a { b } }.is_disjoint(&selection_set! { a { b } }));
//         assert!(selection_set! { a { b } }.is_disjoint(&selection_set! { a { c } }));
//         assert!(!selection_set! { a { b e } }.is_disjoint(&selection_set! { a { c e } }));
//         assert!(selection_set! { a { b { c } } }.is_disjoint(&selection_set! { a { b { d } } }));
//     }
// }
