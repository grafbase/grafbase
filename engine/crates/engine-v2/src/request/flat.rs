use std::cmp::Ordering;

use schema::{Definition, InterfaceId, ObjectId, Schema};

use crate::request::{BoundFieldId, BoundSelectionSetId, SelectionSetType, TypeCondition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatSelectionSetId(BoundSelectionSetId);

impl From<BoundSelectionSetId> for FlatSelectionSetId {
    fn from(value: BoundSelectionSetId) -> Self {
        Self(value)
    }
}

impl From<FlatSelectionSetId> for BoundSelectionSetId {
    fn from(value: FlatSelectionSetId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
pub struct FlatSelectionSet<Ty = SelectionSetType> {
    pub ty: Ty,
    pub id: FlatSelectionSetId,
    pub fields: Vec<FlatField>,
}

#[derive(Debug, Clone)]
pub struct FlatField {
    pub type_condition: Option<FlatTypeCondition>,
    // There is always at least one element.
    pub selection_set_path: Vec<BoundSelectionSetId>,
    pub bound_field_id: BoundFieldId,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityType {
    Interface(InterfaceId),
    Object(ObjectId),
}

impl From<EntityType> for Definition {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => Definition::Interface(id),
            EntityType::Object(id) => Definition::Object(id),
        }
    }
}

impl From<EntityType> for SelectionSetType {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => SelectionSetType::Interface(id),
            EntityType::Object(id) => SelectionSetType::Object(id),
        }
    }
}

impl From<EntityType> for TypeCondition {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => TypeCondition::Interface(id),
            EntityType::Object(id) => TypeCondition::Object(id),
        }
    }
}

impl EntityType {
    pub fn maybe_from(definition: Definition) -> Option<EntityType> {
        match definition {
            Definition::Object(id) => Some(EntityType::Object(id)),
            Definition::Interface(id) => Some(EntityType::Interface(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatTypeCondition {
    Interface(InterfaceId),
    // sorted by ObjectId
    Objects(Box<[ObjectId]>),
}

impl FlatTypeCondition {
    pub fn is_possible(condition: &Option<Self>) -> bool {
        if let Some(condition) = condition {
            match condition {
                FlatTypeCondition::Interface(_) => true,
                FlatTypeCondition::Objects(ids) => !ids.is_empty(),
            }
        } else {
            true
        }
    }

    pub fn matches(&self, schema: &Schema, object_id: ObjectId) -> bool {
        match self {
            FlatTypeCondition::Interface(id) => schema[object_id].interfaces.contains(id),
            FlatTypeCondition::Objects(ids) => ids.binary_search(&object_id).is_ok(),
        }
    }

    pub fn flatten(schema: &Schema, ty: SelectionSetType, type_condition_chain: Vec<TypeCondition>) -> Option<Self> {
        let mut type_condition_chain = type_condition_chain.into_iter().peekable();
        let mut candidate = match ty {
            SelectionSetType::Object(object_id) => {
                // Checking that all type conditions apply.
                for type_condition in type_condition_chain {
                    match type_condition {
                        TypeCondition::Interface(id) if schema[object_id].interfaces.contains(&id) => (),
                        TypeCondition::Object(id) if object_id == id => (),
                        TypeCondition::Union(id) if schema[id].possible_types.contains(&object_id) => (),
                        _ => return Some(FlatTypeCondition::Objects(Box::new([]))),
                    }
                }
                return None;
            }
            SelectionSetType::Interface(id) => {
                // Any type condition that just applies on the root interface is ignored.
                while let Some(next) = type_condition_chain.peek().copied() {
                    if let TypeCondition::Interface(interface_id) = next {
                        if interface_id == id {
                            type_condition_chain.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // If there no type conditions anymore it means it just applies to the root
                // directly.
                type_condition_chain.peek()?;
                FlatTypeCondition::Interface(id)
            }
            SelectionSetType::Union(union_id) => {
                // Any type condition that just applies on the root union is ignored.
                while let Some(next) = type_condition_chain.peek().copied() {
                    if let TypeCondition::Union(id) = next {
                        if union_id == id {
                            type_condition_chain.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let first = type_condition_chain.next()?;
                match first {
                    TypeCondition::Interface(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                    TypeCondition::Object(id) => {
                        if schema[union_id].possible_types.contains(&id) {
                            FlatTypeCondition::Objects(Box::new([id]))
                        } else {
                            FlatTypeCondition::Objects(Box::new([]))
                        }
                    }
                    TypeCondition::Union(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                }
            }
        };

        for type_condition in type_condition_chain {
            candidate = match type_condition {
                TypeCondition::Interface(interface_id) => match candidate {
                    FlatTypeCondition::Interface(id) => {
                        if schema[interface_id].interfaces.contains(&id) {
                            FlatTypeCondition::Interface(id)
                        } else {
                            FlatTypeCondition::Objects(sorted_intersection(
                                &schema[interface_id].possible_types,
                                &schema[id].possible_types,
                            ))
                        }
                    }
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &schema[interface_id].possible_types))
                    }
                },
                TypeCondition::Object(object_id) => match candidate {
                    FlatTypeCondition::Interface(id) => {
                        if schema[object_id].interfaces.contains(&id) {
                            FlatTypeCondition::Objects(Box::new([object_id]))
                        } else {
                            FlatTypeCondition::Objects(Box::new([]))
                        }
                    }
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &[object_id]))
                    }
                },
                TypeCondition::Union(union_id) => match candidate {
                    FlatTypeCondition::Interface(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &schema[union_id].possible_types))
                    }
                },
            };
        }

        Some(candidate)
    }
}

fn sorted_intersection(left: &[ObjectId], right: &[ObjectId]) -> Box<[ObjectId]> {
    let mut l = 0;
    let mut r = 0;
    let mut result = vec![];
    while l < left.len() && r < right.len() {
        match left[l].cmp(&right[r]) {
            Ordering::Less => l += 1,
            Ordering::Equal => {
                result.push(left[l]);
                l += 1;
                r += 1;
            }
            Ordering::Greater => r += 1,
        }
    }
    result.into_boxed_slice()
}
