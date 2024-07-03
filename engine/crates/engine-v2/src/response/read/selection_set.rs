use crate::response::ResponseEdge;

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    items: Vec<ReadField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadField {
    pub edge: ResponseEdge,
    pub name: String,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn extend_disjoint(&mut self, other: Self) {
        self.items.extend(other.items);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn union(self, other: ReadSelectionSet) -> ReadSelectionSet {
        let mut left = self.items;
        let mut right = other.items;

        // We're reading fields from a single entity, so field names will unique.
        left.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        right.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        let mut items = Vec::with_capacity(left.len() + right.len());
        let mut left = left.into_iter().peekable();
        let mut right = right.into_iter().peekable();

        while let (Some(l), Some(r)) = (left.peek(), right.peek()) {
            match l.name.cmp(&r.name) {
                std::cmp::Ordering::Less => {
                    items.push(left.next().unwrap());
                }
                std::cmp::Ordering::Equal => {
                    let left_field = left.next().unwrap();
                    let right_field = right.next().unwrap();
                    items.push(ReadField {
                        edge: left_field.edge,
                        name: left_field.name,
                        subselection: left_field.subselection.union(right_field.subselection),
                    });
                }
                std::cmp::Ordering::Greater => {
                    items.push(right.next().unwrap());
                }
            }
        }

        ReadSelectionSet { items }
    }
}

impl<'a> IntoIterator for &'a ReadSelectionSet {
    type Item = &'a ReadField;

    type IntoIter = <&'a Vec<ReadField> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl FromIterator<ReadField> for ReadSelectionSet {
    fn from_iter<T: IntoIterator<Item = ReadField>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}
