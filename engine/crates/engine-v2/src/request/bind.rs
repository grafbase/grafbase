use std::collections::{HashMap, HashSet};

pub use engine_parser::types::OperationType;
use engine_parser::Positioned;
use schema::{Definition, FieldWalker, InterfaceId, ObjectId, Schema, UnionId};

use super::{
    selection_set::BoundField, BoundFieldArgument, BoundFieldDefinition, BoundFieldDefinitionId, BoundFieldId,
    BoundFragmentDefinition, BoundFragmentDefinitionId, BoundFragmentSpread, BoundInlineFragment, BoundSelection,
    BoundSelectionSet, BoundSelectionSetId, Operation, Pos, TypeCondition, UnboundOperation,
};
use crate::{execution::Strings, response::GraphqlError};

#[derive(thiserror::Error, Debug)]
pub enum BindError {
    #[error("Unknown type named '{name}'")]
    UnknownType { name: String, location: Pos },
    #[error("{container} does not have a field named '{name}'")]
    UnknownField {
        container: String,
        name: String,
        location: Pos,
    },
    #[error("Field '{field}' does not have an argument named '{name}'")]
    UnknownFieldArgument { field: String, name: String, location: Pos },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, location: Pos },
    #[error("Field '{name}' does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.")]
    UnionHaveNoFields { name: String, ty: String, location: Pos },
    #[error("Field '{name}' cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet { name: String, ty: String, location: Pos },
    #[error("Type conditions cannot be declared on '{name}', only on unions, interfaces or objects.")]
    InvalidTypeConditionTargetType { name: String, location: Pos },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set")]
    DisjointTypeCondition {
        parent: String,
        name: String,
        location: Pos,
    },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
}

impl From<BindError> for GraphqlError {
    fn from(err: BindError) -> Self {
        let locations = match err {
            BindError::UnknownField { location, .. }
            | BindError::UnknownFieldArgument { location, .. }
            | BindError::UnknownType { location, .. }
            | BindError::UnknownFragment { location, .. }
            | BindError::UnionHaveNoFields { location, .. }
            | BindError::InvalidTypeConditionTargetType { location, .. }
            | BindError::CannotHaveSelectionSet { location, .. }
            | BindError::DisjointTypeCondition { location, .. } => vec![location],
            BindError::NoMutationDefined | BindError::NoSubscriptionDefined => vec![],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            path: vec![],
        }
    }
}

pub type BindResult<T> = Result<T, BindError>;

pub fn bind(schema: &Schema, unbound: UnboundOperation) -> BindResult<Operation> {
    let root_object_id = match unbound.definition.ty {
        OperationType::Query => schema.root_operation_types.query,
        OperationType::Mutation => schema
            .root_operation_types
            .mutation
            .ok_or(BindError::NoMutationDefined)?,
        OperationType::Subscription => schema
            .root_operation_types
            .subscription
            .ok_or(BindError::NoSubscriptionDefined)?,
    };
    let mut binder = Binder {
        schema,
        strings: Strings::new(),
        fragment_definitions: HashMap::new(),
        field_definitions: Vec::new(),
        fields: Vec::new(),
        selection_sets: vec![BoundSelectionSet { items: vec![] }],
        emtpy_selection_set_id: BoundSelectionSetId::from(0),
        unbound_fragments: unbound.fragments,
    };
    let root_selection_set_id =
        binder.bind_selection_set(ParentType::Object(root_object_id), unbound.definition.selection_set)?;
    Ok(Operation {
        ty: unbound.definition.ty,
        root_object_id,
        name: unbound.name,
        root_selection_set_id,
        selection_sets: binder.selection_sets,
        fragment_definitions: {
            let mut fragment_definitions = binder.fragment_definitions.into_values().collect::<Vec<_>>();
            fragment_definitions.sort_unstable_by_key(|(id, _)| *id);
            fragment_definitions.into_iter().map(|(_, def)| def).collect()
        },
        strings: binder.strings,
        field_definitions: binder.field_definitions,
        fields: binder.fields,
    })
}

pub struct Binder<'a> {
    schema: &'a Schema,
    strings: Strings,
    unbound_fragments: HashMap<String, Positioned<engine_parser::types::FragmentDefinition>>,
    fragment_definitions: HashMap<String, (BoundFragmentDefinitionId, BoundFragmentDefinition)>,
    field_definitions: Vec<BoundFieldDefinition>,
    fields: Vec<BoundField>,
    selection_sets: Vec<BoundSelectionSet>,
    emtpy_selection_set_id: BoundSelectionSetId,
}

impl<'a> Binder<'a> {
    fn bind_selection_set(
        &mut self,
        parent: ParentType,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<BoundSelectionSetId> {
        let Positioned {
            pos: _,
            node: selection_set,
        } = selection_set;
        // Keeping the original ordering
        let bound = selection_set
            .items
            .into_iter()
            .map(|Positioned { node: selection, .. }| match selection {
                engine_parser::types::Selection::Field(selection) => self.bind_field(parent, selection),
                engine_parser::types::Selection::FragmentSpread(selection) => {
                    self.bind_fragment_spread(parent, selection)
                }
                engine_parser::types::Selection::InlineFragment(selection) => {
                    self.bind_inline_fragment(parent, selection)
                }
            })
            .collect::<BindResult<BoundSelectionSet>>()?;
        let id = BoundSelectionSetId::from(self.selection_sets.len());
        self.selection_sets.push(bound);
        Ok(id)
    }

    fn bind_field(
        &mut self,
        parent: ParentType,
        Positioned {
            pos: name_location,
            node: field,
        }: Positioned<engine_parser::types::Field>,
    ) -> BindResult<BoundSelection> {
        let walker = self.schema.default_walker();
        let name = field.name.node.as_str();
        let schema_field: FieldWalker<'_> = match parent {
            ParentType::Object(object_id) => self.schema.object_field_by_name(object_id, name),
            ParentType::Interface(interface_id) => self.schema.interface_field_by_name(interface_id, name),
            ParentType::Union(union_id) => {
                return Err(BindError::UnionHaveNoFields {
                    name: name.to_string(),
                    ty: walker.walk(union_id).name().to_string(),
                    location: name_location,
                });
            }
        }
        .map(|field_id| walker.walk(field_id))
        .ok_or_else(|| BindError::UnknownField {
            container: walker.walk(Definition::from(parent)).name().to_string(),
            name: name.to_string(),
            location: name_location,
        })?;

        let arguments = field
            .arguments
            .into_iter()
            .map(
                |(
                    Positioned {
                        pos: name_location,
                        node: name,
                    },
                    Positioned {
                        pos: value_location,
                        node: value,
                    },
                )| {
                    let name = name.to_string();
                    schema_field
                        .argument_by_name(&name)
                        .map(|input_value| BoundFieldArgument {
                            name_location,
                            input_value_id: input_value.id,
                            value_location,
                            value,
                        })
                        .ok_or_else(|| BindError::UnknownFieldArgument {
                            field: schema_field.name().to_string(),
                            name,
                            location: name_location,
                        })
                },
            )
            .collect::<BindResult<_>>()?;

        let selection_set_id = if field.selection_set.node.items.is_empty() {
            self.emtpy_selection_set_id
        } else {
            match schema_field.ty().inner().id {
                Definition::Object(object_id) => {
                    self.bind_selection_set(ParentType::Object(object_id), field.selection_set)
                }
                Definition::Interface(interface_id) => {
                    self.bind_selection_set(ParentType::Interface(interface_id), field.selection_set)
                }
                Definition::Union(union_id) => {
                    self.bind_selection_set(ParentType::Union(union_id), field.selection_set)
                }
                _ => Err(BindError::CannotHaveSelectionSet {
                    name: name.to_string(),
                    ty: schema_field.ty().name().to_string(),
                    location: name_location,
                }),
            }?
        };
        let name = &field
            .alias
            .map(|Positioned { node, .. }| node.to_string())
            .unwrap_or_else(|| name.to_string());
        let operation_field_definition_id = BoundFieldDefinitionId::from(self.field_definitions.len());
        self.field_definitions.push(BoundFieldDefinition {
            name: self.strings.get_or_intern(name),
            name_location,
            field_id: schema_field.id,
            arguments,
        });
        let bound_field_id = BoundFieldId::from(self.fields.len());
        self.fields.push(BoundField {
            definition_id: operation_field_definition_id,
            selection_set_id,
        });
        Ok(BoundSelection::Field(bound_field_id))
    }

    fn bind_fragment_spread(
        &mut self,
        parent: ParentType,
        Positioned {
            pos: location,
            node: spread,
        }: Positioned<engine_parser::types::FragmentSpread>,
    ) -> BindResult<BoundSelection> {
        // We always create a new selection set from a named fragment. It may not be split in the
        // same way and we need to validate the type condition each time.
        let name = spread.fragment_name.node.as_str();
        let Positioned {
            pos: fragment_definition_pos,
            node: fragment_definition,
        } = self
            .unbound_fragments
            .get(name)
            .cloned()
            .ok_or_else(|| BindError::UnknownFragment {
                name: name.to_string(),
                location,
            })?;
        let type_condition = self.bind_type_condition(parent, &fragment_definition.type_condition)?;
        let selection_set_id = self.bind_selection_set(type_condition.into(), fragment_definition.selection_set)?;

        Ok(BoundSelection::FragmentSpread(BoundFragmentSpread {
            location,
            selection_set_id,
            fragment_id: match self.fragment_definitions.get(name) {
                Some((id, _)) => *id,
                None => {
                    // A bound fragment definition has no selection set, it was already bound. We
                    // only keep it for errors (name/name_pos) and directives.
                    let fragment_definition = BoundFragmentDefinition {
                        name: name.to_string(),
                        name_location: fragment_definition_pos,
                        type_condition,
                        directives: vec![],
                    };
                    let id = BoundFragmentDefinitionId::from(self.fragment_definitions.len());
                    self.fragment_definitions
                        .insert(name.to_string(), (id, fragment_definition));

                    id
                }
            },
        }))
    }

    fn bind_inline_fragment(
        &mut self,
        parent: ParentType,
        Positioned {
            pos: location,
            node: fragment,
        }: Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<BoundSelection> {
        let type_condition = fragment
            .type_condition
            .map(|condition| self.bind_type_condition(parent, &condition))
            .transpose()?;
        let fragment_root = type_condition.map(Into::into).unwrap_or(parent);
        let selection_set_id = self.bind_selection_set(fragment_root, fragment.selection_set)?;
        Ok(BoundSelection::InlineFragment(BoundInlineFragment {
            location,
            type_condition,
            selection_set_id,
            directives: vec![],
        }))
    }

    fn bind_type_condition(
        &self,
        parent: ParentType,
        Positioned { pos: location, node }: &Positioned<engine_parser::types::TypeCondition>,
    ) -> BindResult<TypeCondition> {
        let location = *location;
        let name = node.on.node.as_str();
        let definition = self
            .schema
            .definition_by_name(name)
            .ok_or_else(|| BindError::UnknownType {
                name: name.to_string(),
                location,
            })?;
        let type_condition = match definition {
            Definition::Object(object_id) => TypeCondition::Object(object_id),
            Definition::Interface(interface_id) => TypeCondition::Interface(interface_id),
            Definition::Union(union_id) => TypeCondition::Union(union_id),
            _ => {
                return Err(BindError::InvalidTypeConditionTargetType {
                    name: name.to_string(),
                    location,
                });
            }
        };
        let possible_types = TypeCondition::from(parent)
            .resolve(self.schema)
            .possible_types()
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let frament_possible_types = type_condition
            .resolve(self.schema)
            .possible_types()
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        if possible_types.is_disjoint(&frament_possible_types) {
            let walker = self.schema.default_walker();
            return Err(BindError::DisjointTypeCondition {
                parent: walker.walk(Definition::from(parent)).name().to_string(),
                name: name.to_string(),
                location,
            });
        }
        Ok(type_condition)
    }
}

#[derive(Clone, Copy)]
pub enum ParentType {
    Union(UnionId),
    Interface(InterfaceId),
    Object(ObjectId),
}

impl From<ParentType> for TypeCondition {
    fn from(parent: ParentType) -> Self {
        match parent {
            ParentType::Interface(id) => Self::Interface(id),
            ParentType::Object(id) => Self::Object(id),
            ParentType::Union(id) => Self::Union(id),
        }
    }
}

impl From<TypeCondition> for ParentType {
    fn from(cond: TypeCondition) -> Self {
        match cond {
            TypeCondition::Interface(id) => Self::Interface(id),
            TypeCondition::Object(id) => Self::Object(id),
            TypeCondition::Union(id) => Self::Union(id),
        }
    }
}

impl From<ParentType> for Definition {
    fn from(parent: ParentType) -> Self {
        match parent {
            ParentType::Interface(id) => Self::Interface(id),
            ParentType::Object(id) => Self::Object(id),
            ParentType::Union(id) => Self::Union(id),
        }
    }
}
