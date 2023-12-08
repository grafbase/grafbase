use std::collections::{HashMap, HashSet};

pub use engine_parser::types::OperationType;
use engine_parser::Positioned;
use schema::{Definition, FieldWalker, Schema};

use super::{
    selection_set::BoundField, variable::VariableDefinition, BoundAnyFieldDefinition, BoundAnyFieldDefinitionId,
    BoundFieldArgument, BoundFieldDefinition, BoundFieldId, BoundFragmentDefinition, BoundFragmentDefinitionId,
    BoundFragmentSpread, BoundInlineFragment, BoundSelection, BoundSelectionSet, BoundSelectionSetId,
    BoundTypeNameFieldDefinition, Operation, Pos, ResponseKeys, SelectionSetType, TypeCondition, UnboundOperation,
};
use crate::response::GraphqlError;

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
    #[error("Leaf field '{name}' must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum { name: String, ty: String, location: Pos },
    #[error(
        "Variable named '${name}' does not have a valid input type. Can only be a scalar, enum or input object. Found: '{ty}'."
    )]
    InvalidVariableType { name: String, ty: String, location: Pos },
    #[error("Too many fields selection set.")]
    TooManyFields { location: Pos },
    #[error("There can only be one variable named '${name}'")]
    DuplicateVariable { name: String, location: Pos },
    #[error("Variable '${name}' is not defined{operation}")]
    UndefinedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Pos,
    },
    #[error("Variable '${name}' is not used{operation}")]
    UnusedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Pos,
    },
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
            | BindError::DisjointTypeCondition { location, .. }
            | BindError::InvalidVariableType { location, .. }
            | BindError::TooManyFields { location }
            | BindError::LeafMustBeAScalarOrEnum { location, .. }
            | BindError::DuplicateVariable { location, .. }
            | BindError::UndefinedVariable { location, .. }
            | BindError::UnusedVariable { location, .. } => vec![location],
            BindError::NoMutationDefined | BindError::NoSubscriptionDefined => vec![],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            ..Default::default()
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
        operation_name: ErrorOperationName(unbound.name.clone()),
        response_keys: ResponseKeys::default(),
        fragment_definitions: HashMap::new(),
        field_definitions: Vec::new(),
        fields: Vec::new(),
        selection_sets: vec![],
        unbound_fragments: unbound.fragments,
        variable_definitions: vec![],
        variables_used: HashSet::new(),
        next_response_position: 0,
    };

    binder.variable_definitions = binder.bind_variables(unbound.definition.variable_definitions)?;

    let root_selection_set_id = binder.bind_field_selection_set(
        SelectionSetType::Object(root_object_id),
        unbound.definition.selection_set,
    )?;

    binder.validate_all_variables_used()?;

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
        response_keys: binder.response_keys,
        field_definitions: binder.field_definitions,
        fields: binder.fields,
        variable_definitions: binder.variable_definitions,
    })
}

pub struct Binder<'a> {
    schema: &'a Schema,
    operation_name: ErrorOperationName,
    response_keys: ResponseKeys,
    unbound_fragments: HashMap<String, Positioned<engine_parser::types::FragmentDefinition>>,
    fragment_definitions: HashMap<String, (BoundFragmentDefinitionId, BoundFragmentDefinition)>,
    field_definitions: Vec<BoundAnyFieldDefinition>,
    fields: Vec<BoundField>,
    selection_sets: Vec<BoundSelectionSet>,
    variable_definitions: Vec<VariableDefinition>,
    variables_used: HashSet<String>,
    // We keep track of the position of fields within the response object that will be
    // returned. With type conditions it's not obvious to know which field will be present or
    // not, but we can order all bound fields. This needs to be done at the request binding
    // level to ensure that fields stay in the right order even if split across different
    // execution plans.
    // We also need to use the global request position so that merged selection sets are still
    // in the right order.
    next_response_position: usize,
}

impl<'a> Binder<'a> {
    fn bind_variables(
        &self,
        variables: Vec<Positioned<engine_parser::types::VariableDefinition>>,
    ) -> BindResult<Vec<VariableDefinition>> {
        let mut seen_names = HashSet::new();
        let mut bound_variables = vec![];

        for Positioned { node, .. } in variables {
            let name = node.name.node.to_string();
            let name_location = node.name.pos;

            if seen_names.contains(&name) {
                return Err(BindError::DuplicateVariable {
                    name,
                    location: name_location,
                });
            }
            seen_names.insert(name.clone());

            let default_value = node.default_value.map(|Positioned { pos: _, node }| node);
            let r#type = self.convert_type(&name, node.var_type.pos, node.var_type.node)?;

            bound_variables.push(VariableDefinition {
                name,
                name_location,
                directives: vec![],
                default_value,
                r#type,
            });
        }

        Ok(bound_variables)
    }

    fn convert_type(
        &self,
        variable_name: &str,
        location: Pos,
        ty: engine_parser::types::Type,
    ) -> BindResult<schema::Type> {
        match ty.base {
            engine_parser::types::BaseType::Named(type_name) => {
                let definition =
                    self.schema
                        .definition_by_name(type_name.as_str())
                        .ok_or_else(|| BindError::UnknownType {
                            name: type_name.to_string(),
                            location,
                        })?;
                if !matches!(
                    definition,
                    Definition::Enum(_) | Definition::Scalar(_) | Definition::InputObject(_)
                ) {
                    return Err(BindError::InvalidVariableType {
                        name: variable_name.to_string(),
                        ty: self.schema.walker().walk(definition).name().to_string(),
                        location,
                    });
                }
                Ok(schema::Type {
                    inner: definition,
                    wrapping: schema::Wrapping {
                        inner_is_required: !ty.nullable,
                        list_wrapping: vec![],
                    },
                })
            }
            engine_parser::types::BaseType::List(nested) => {
                self.convert_type(variable_name, location, *nested).map(|mut r#type| {
                    r#type.wrapping.list_wrapping.push(if ty.nullable {
                        schema::ListWrapping::NullableList
                    } else {
                        schema::ListWrapping::RequiredList
                    });
                    r#type
                })
            }
        }
    }

    fn bind_field_selection_set(
        &mut self,
        root: SelectionSetType,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<BoundSelectionSetId> {
        self.bind_selection_set(root, selection_set)
    }

    fn bind_selection_set(
        &mut self,
        root: SelectionSetType,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<BoundSelectionSetId> {
        let Positioned {
            pos: _,
            node: selection_set,
        } = selection_set;
        // Keeping the original ordering
        let items = selection_set
            .items
            .into_iter()
            .map(|Positioned { node: selection, .. }| match selection {
                engine_parser::types::Selection::Field(selection) => self.bind_field(root, selection),
                engine_parser::types::Selection::FragmentSpread(selection) => {
                    self.bind_fragment_spread(root, selection)
                }
                engine_parser::types::Selection::InlineFragment(selection) => {
                    self.bind_inline_fragment(root, selection)
                }
            })
            .collect::<BindResult<Vec<_>>>()?;
        let id = BoundSelectionSetId::from(self.selection_sets.len());
        let selection_set = BoundSelectionSet { ty: root, items };
        self.selection_sets.push(selection_set);
        Ok(id)
    }

    fn bind_field(
        &mut self,
        root: SelectionSetType,
        Positioned {
            pos: name_location,
            node: field,
        }: Positioned<engine_parser::types::Field>,
    ) -> BindResult<BoundSelection> {
        let walker = self.schema.walker();
        let name = field.name.node.as_str();
        let response_key = self.response_keys.get_or_intern(
            &field
                .alias
                .map(|Positioned { node, .. }| node.to_string())
                .unwrap_or_else(|| name.to_string()),
        );
        let bound_response_key =
            response_key
                .with_position(self.next_response_position)
                .ok_or(BindError::TooManyFields {
                    location: name_location,
                })?;
        self.next_response_position += 1;

        let (bound_field_definition, selection_set_id) = match name {
            "__typename" => (
                BoundAnyFieldDefinition::TypeName(BoundTypeNameFieldDefinition {
                    name_location,
                    response_key,
                }),
                None,
            ),

            name => {
                let schema_field: FieldWalker<'_> = match root {
                    SelectionSetType::Object(object_id) => self.schema.object_field_by_name(object_id, name),
                    SelectionSetType::Interface(interface_id) => {
                        self.schema.interface_field_by_name(interface_id, name)
                    }
                    SelectionSetType::Union(union_id) => {
                        return Err(BindError::UnionHaveNoFields {
                            name: name.to_string(),
                            ty: walker.walk(union_id).name().to_string(),
                            location: name_location,
                        });
                    }
                }
                .map(|field_id| walker.walk(field_id))
                .ok_or_else(|| BindError::UnknownField {
                    container: walker.walk(Definition::from(root)).name().to_string(),
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
                                    input_value_id: input_value.id(),
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
                    .collect::<BindResult<Vec<_>>>()?;

                self.validate_argument_variables(&arguments)?;

                let selection_set_id = if field.selection_set.node.items.is_empty() {
                    if !matches!(
                        schema_field.ty().inner().id(),
                        Definition::Scalar(_) | Definition::Enum(_)
                    ) {
                        return Err(BindError::LeafMustBeAScalarOrEnum {
                            name: name.to_string(),
                            ty: schema_field.ty().inner().name().to_string(),
                            location: name_location,
                        });
                    }
                    None
                } else {
                    Some(match schema_field.ty().inner().id() {
                        Definition::Object(object_id) => {
                            self.bind_field_selection_set(SelectionSetType::Object(object_id), field.selection_set)
                        }
                        Definition::Interface(interface_id) => self
                            .bind_field_selection_set(SelectionSetType::Interface(interface_id), field.selection_set),
                        Definition::Union(union_id) => {
                            self.bind_field_selection_set(SelectionSetType::Union(union_id), field.selection_set)
                        }
                        _ => Err(BindError::CannotHaveSelectionSet {
                            name: name.to_string(),
                            ty: schema_field.ty().to_string(),
                            location: name_location,
                        }),
                    }?)
                };
                (
                    BoundAnyFieldDefinition::Field(BoundFieldDefinition {
                        response_key,
                        name_location,
                        field_id: schema_field.id(),
                        arguments,
                    }),
                    selection_set_id,
                )
            }
        };
        let definition_id = BoundAnyFieldDefinitionId::from(self.field_definitions.len());
        self.field_definitions.push(bound_field_definition);
        let bound_field_id = BoundFieldId::from(self.fields.len());
        self.fields.push(BoundField {
            // Adding the position ensures fields are always returned in the right order in the
            // final response. Currently, we support up to 4095 bound fields in a selection set,
            // which should be more than enough for anything remotely sane.
            bound_response_key,
            definition_id,
            selection_set_id,
        });
        Ok(BoundSelection::Field(bound_field_id))
    }

    fn bind_fragment_spread(
        &mut self,
        root: SelectionSetType,
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
        let type_condition = self.bind_type_condition(root, &fragment_definition.type_condition)?;
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
        root: SelectionSetType,
        Positioned {
            pos: location,
            node: fragment,
        }: Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<BoundSelection> {
        let type_condition = fragment
            .type_condition
            .map(|condition| self.bind_type_condition(root, &condition))
            .transpose()?;
        let fragment_root = type_condition.map(Into::into).unwrap_or(root);
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
        root: SelectionSetType,
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
        let possible_types = TypeCondition::from(root)
            .resolve(self.schema)
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let frament_possible_types = type_condition
            .resolve(self.schema)
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        if possible_types.is_disjoint(&frament_possible_types) {
            let walker = self.schema.walker();
            return Err(BindError::DisjointTypeCondition {
                parent: walker.walk(Definition::from(root)).name().to_string(),
                name: name.to_string(),
                location,
            });
        }
        Ok(type_condition)
    }

    fn validate_argument_variables(&mut self, arguments: &[BoundFieldArgument]) -> BindResult<()> {
        for argument in arguments {
            for variable in argument.value.variables_used() {
                if !self
                    .variable_definitions
                    .iter()
                    .any(|definition| definition.name == *variable)
                {
                    return Err(BindError::UndefinedVariable {
                        name: variable.to_string(),
                        operation: self.operation_name.clone(),
                        location: argument.value_location,
                    });
                }
                self.variables_used.insert(variable.to_string());
            }
        }

        Ok(())
    }

    fn validate_all_variables_used(&self) -> BindResult<()> {
        for variable in &self.variable_definitions {
            if !self.variables_used.contains(&variable.name) {
                return Err(BindError::UnusedVariable {
                    name: variable.name.clone(),
                    location: variable.name_location,
                    operation: self.operation_name.clone(),
                });
            }
        }

        Ok(())
    }
}

impl From<SelectionSetType> for TypeCondition {
    fn from(parent: SelectionSetType) -> Self {
        match parent {
            SelectionSetType::Interface(id) => Self::Interface(id),
            SelectionSetType::Object(id) => Self::Object(id),
            SelectionSetType::Union(id) => Self::Union(id),
        }
    }
}

impl From<TypeCondition> for SelectionSetType {
    fn from(cond: TypeCondition) -> Self {
        match cond {
            TypeCondition::Interface(id) => Self::Interface(id),
            TypeCondition::Object(id) => Self::Object(id),
            TypeCondition::Union(id) => Self::Union(id),
        }
    }
}

impl From<SelectionSetType> for Definition {
    fn from(parent: SelectionSetType) -> Self {
        match parent {
            SelectionSetType::Interface(id) => Self::Interface(id),
            SelectionSetType::Object(id) => Self::Object(id),
            SelectionSetType::Union(id) => Self::Union(id),
        }
    }
}

/// A helper struct for optionally including operation names in error messages
#[derive(Debug, Clone)]
pub struct ErrorOperationName(Option<String>);

impl std::fmt::Display for ErrorOperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.0 {
            write!(f, " by operation '{name}'")?;
        }
        Ok(())
    }
}
