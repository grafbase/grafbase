use schema::{ObjectId, Schema};

use super::TypeCondition;
use crate::execution::StrId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryPath(im::Vector<QueryPathSegment>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPathSegment {
    pub resolved_type_condition: Option<ResolvedTypeCondition>,
    pub name: StrId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedTypeCondition {
    // sorted to guarantee deterministic order
    possible_types: Vec<ObjectId>,
}

impl ResolvedTypeCondition {
    pub fn new(mut possible_types: Vec<ObjectId>) -> Self {
        possible_types.sort_unstable();
        Self { possible_types }
    }

    pub fn matches(&self, object_id: ObjectId) -> bool {
        self.possible_types.contains(&object_id)
    }

    pub fn possible_types(&self) -> &[ObjectId] {
        self.possible_types.as_slice()
    }

    pub fn merge(
        parent: Option<ResolvedTypeCondition>,
        nested: Option<ResolvedTypeCondition>,
    ) -> Option<ResolvedTypeCondition> {
        match (parent, nested) {
            (None, None) => None,
            (None, cond) | (cond, None) => cond,
            (Some(parent), Some(nested)) => Some(Self::new(
                parent
                    .possible_types
                    .into_iter()
                    .filter(|object_id| nested.matches(*object_id))
                    .collect(),
            )),
        }
    }
}

impl TypeCondition {
    pub fn resolve(self, schema: &Schema) -> ResolvedTypeCondition {
        ResolvedTypeCondition {
            possible_types: match self {
                TypeCondition::Interface(interface_id) => schema[interface_id].possible_types.clone(),
                TypeCondition::Object(object_id) => vec![object_id],
                TypeCondition::Union(union_id) => schema[union_id].possible_types.clone(),
            },
        }
    }
}

impl<'a> IntoIterator for &'a QueryPath {
    type Item = &'a QueryPathSegment;

    type IntoIter = <&'a im::Vector<QueryPathSegment> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl QueryPath {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(&self, segment: QueryPathSegment) -> Self {
        let mut child = self.clone();
        child.0.push_back(segment);
        child
    }
}
