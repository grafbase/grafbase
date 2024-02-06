use std::collections::{HashMap, HashSet};

pub use engine_parser::types::OperationType;
use engine_parser::Positioned;
use engine_value::Name;
use itertools::Itertools;
use schema::{Definition, FieldWalker, Schema};

use crate::response::GraphqlError;

use super::{
    selection_set::BoundField, variable::VariableDefinition, BoundFieldArgument, BoundFieldArguments,
    BoundFieldArgumentsId, BoundFieldId, BoundFragment, BoundFragmentId, BoundFragmentSpread, BoundFragmentSpreadId,
    BoundInlineFragment, BoundInlineFragmentId, BoundSelection, BoundSelectionSet, BoundSelectionSetId, Location,
    Operation, ParsedOperation, ResponseKeys, SelectionSetType, TypeCondition,
};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum OperationLimitExceededError {
    #[error("Query is too complex.")]
    QueryTooComplex,
    #[error("Query is nested too deep.")]
    QueryTooDeep,
    #[error("Query is too high.")]
    QueryTooHigh,
    #[error("Query contains too many root fields.")]
    QueryContainsTooManyRootFields,
    #[error("Query contains too many aliases.")]
    QueryContainsTooManyAliases,
}

#[derive(thiserror::Error, Debug)]
pub enum BindError {
    #[error("Unknown type named '{name}'")]
    UnknownType { name: String, location: Location },
    #[error("{container} does not have a field named '{name}'")]
    UnknownField {
        container: String,
        name: String,
        location: Location,
    },
    #[error("Field '{field}' does not have an argument named '{name}'")]
    UnknownFieldArgument {
        field: String,
        name: String,
        location: Location,
    },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, location: Location },
    #[error("Field '{name}' does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.")]
    UnionHaveNoFields {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Field '{name}' cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Type conditions cannot be declared on '{name}', only on unions, interfaces or objects.")]
    InvalidTypeConditionTargetType { name: String, location: Location },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set")]
    DisjointTypeCondition {
        parent: String,
        name: String,
        location: Location,
    },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
    #[error("Leaf field '{name}' must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum {
        name: String,
        ty: String,
        location: Location,
    },
    #[error(
        "Variable named '${name}' does not have a valid input type. Can only be a scalar, enum or input object. Found: '{ty}'."
    )]
    InvalidVariableType {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Too many fields selection set.")]
    TooManyFields { location: Location },
    #[error("There can only be one variable named '${name}'")]
    DuplicateVariable { name: String, location: Location },
    #[error("Variable '${name}' is not defined{operation}")]
    UndefinedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Location,
    },
    #[error("Variable '${name}' is not used{operation}")]
    UnusedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Location,
    },
    #[error("Fragment cycle detected: {}", .cycle.iter().join(", "))]
    FragmentCycle { cycle: Vec<String>, location: Location },
    #[error("{0}")]
    OperationLimitExceeded(OperationLimitExceededError),
    #[error("Query is too big: {0}")]
    QueryTooBig(String),
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
            | BindError::FragmentCycle { location, .. }
            | BindError::UnusedVariable { location, .. } => vec![location],
            BindError::NoMutationDefined
            | BindError::NoSubscriptionDefined
            | BindError::QueryTooBig { .. }
            | BindError::OperationLimitExceeded { .. } => {
                vec![]
            }
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            ..Default::default()
        }
    }
}

pub type BindResult<T> = Result<T, BindError>;

pub fn bind(schema: &Schema, mut unbound: ParsedOperation) -> BindResult<Operation> {
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
        fragments: HashMap::default(),
        field_arguments: vec![Vec::new()], // first one for all empty arguments.
        location_to_field_arguments: HashMap::default(),
        fields: Vec::new(),
        selection_sets: vec![],
        unbound_fragments: unbound.fragments,
        variable_definitions: vec![],
        variables_used: HashSet::new(),
        next_response_position: 0,
        current_fragments_stack: Vec::new(),
        fragment_spreads: Vec::new(),
        inline_fragments: Vec::new(),
        field_to_parent: Vec::new(),
    };

    binder.variable_definitions = binder.bind_variables(unbound.definition.variable_definitions)?;

    let root_selection_set_id = binder.bind_selection_set(
        SelectionSetType::Object(root_object_id),
        &mut unbound.definition.selection_set,
    )?;

    binder.validate_all_variables_used()?;

    Ok(Operation {
        ty: unbound.definition.ty,
        root_object_id,
        name: unbound.name,
        root_selection_set_id,
        selection_sets: binder.selection_sets,
        fragments: {
            let mut fragment_definitions = binder.fragments.into_values().collect::<Vec<_>>();
            fragment_definitions.sort_unstable_by_key(|(id, _)| *id);
            fragment_definitions.into_iter().map(|(_, def)| def).collect()
        },
        field_arguments: binder.field_arguments,
        response_keys: binder.response_keys,
        fields: binder.fields,
        variable_definitions: binder.variable_definitions,
        cache_config: None,
        fragment_spreads: binder.fragment_spreads,
        inline_fragments: binder.inline_fragments,
        field_to_parent: binder.field_to_parent,
    })
}

pub struct Binder<'a> {
    schema: &'a Schema,
    operation_name: ErrorOperationName,
    response_keys: ResponseKeys,
    unbound_fragments: HashMap<String, Positioned<engine_parser::types::FragmentDefinition>>,
    fragments: HashMap<String, (BoundFragmentId, BoundFragment)>,
    field_arguments: Vec<BoundFieldArguments>,
    location_to_field_arguments: HashMap<Location, BoundFieldArgumentsId>,
    fields: Vec<BoundField>,
    field_to_parent: Vec<BoundSelectionSetId>,
    fragment_spreads: Vec<BoundFragmentSpread>,
    inline_fragments: Vec<BoundInlineFragment>,
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
    current_fragments_stack: Vec<String>,
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
            let name_location = node.name.pos.try_into()?;

            if seen_names.contains(&name) {
                return Err(BindError::DuplicateVariable {
                    name,
                    location: name_location,
                });
            }
            seen_names.insert(name.clone());

            let default_value = node.default_value.map(|Positioned { pos: _, node }| node);
            let r#type = self.convert_type(&name, node.var_type.pos.try_into()?, node.var_type.node)?;

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
        location: Location,
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

    fn bind_selection_set(
        &mut self,
        root: SelectionSetType,
        selection_set: &mut Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<BoundSelectionSetId> {
        let Positioned {
            node: selection_set, ..
        } = selection_set;
        // Keeping the original ordering
        let items = selection_set
            .items
            .iter_mut()
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
        for item in &self.selection_sets[usize::from(id)].items {
            if let BoundSelection::Field(bound_field_id) = item {
                self.field_to_parent[usize::from(*bound_field_id)] = id;
            }
        }
        Ok(id)
    }

    fn bind_field(
        &mut self,
        root: SelectionSetType,
        Positioned { pos, node: field }: &mut Positioned<engine_parser::types::Field>,
    ) -> BindResult<BoundSelection> {
        let name_location: Location = (*pos).try_into()?;
        let walker = self.schema.walker();
        let name = field.name.node.as_str();
        let response_key = self.response_keys.get_or_intern(
            field
                .alias
                .as_ref()
                .map(|Positioned { node, .. }| node.as_str())
                .unwrap_or_else(|| name),
        );
        let bound_response_key =
            response_key
                .with_position(self.next_response_position)
                .ok_or(BindError::TooManyFields {
                    location: name_location,
                })?;
        self.next_response_position += 1;

        let bound_field = match name {
            "__typename" => BoundField::TypeName {
                bound_response_key,
                location: name_location,
            },
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

                let arguments_id = self.bind_field_arguments(schema_field, name_location, &mut field.arguments)?;

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
                    Some(
                        SelectionSetType::maybe_from(schema_field.ty().inner().id())
                            .ok_or_else(|| BindError::CannotHaveSelectionSet {
                                name: name.to_string(),
                                ty: schema_field.ty().to_string(),
                                location: name_location,
                            })
                            .and_then(|ty| self.bind_selection_set(ty, &mut field.selection_set))?,
                    )
                };
                BoundField::Field {
                    bound_response_key,
                    location: name_location,
                    field_id: schema_field.id(),
                    arguments_id,
                    selection_set_id,
                }
            }
        };
        let bound_field_id = BoundFieldId::from(self.fields.len());
        self.fields.push(bound_field);
        // Adding a placeholder.
        self.field_to_parent.push(BoundSelectionSetId::from(0));
        Ok(BoundSelection::Field(bound_field_id))
    }

    fn bind_field_arguments(
        &mut self,
        schema_field: FieldWalker<'_>,
        field_location: Location,
        arguments: &mut Vec<(Positioned<Name>, Positioned<engine_value::Value>)>,
    ) -> BindResult<BoundFieldArgumentsId> {
        if arguments.is_empty() {
            return Ok(BoundFieldArgumentsId::from(0));
        }

        // Avoid binding multiple times the same arguments (same framgnets used at different places)
        if let Some(id) = self.location_to_field_arguments.get(&field_location) {
            return Ok(*id);
        }

        let bound_arguments = std::mem::take(arguments)
            .into_iter()
            .map(|(name, value)| {
                let name_location = name.pos.try_into()?;
                let name = name.node.as_str();
                schema_field
                    .argument_by_name(name)
                    .ok_or_else(|| BindError::UnknownFieldArgument {
                        field: schema_field.name().to_string(),
                        name: name.to_string(),
                        location: name_location,
                    })
                    .and_then(|input_value| {
                        Ok(BoundFieldArgument {
                            name_location,
                            input_value_id: input_value.id(),
                            value_location: value.pos.try_into()?,
                            value: value.node,
                        })
                    })
            })
            .collect::<BindResult<Vec<_>>>()?;

        self.validate_argument_variables(&bound_arguments)?;
        let id = BoundFieldArgumentsId::from(self.field_arguments.len());
        self.field_arguments.push(bound_arguments);
        Ok(id)
    }

    fn bind_fragment_spread(
        &mut self,
        root: SelectionSetType,
        Positioned { pos, node: spread }: &mut Positioned<engine_parser::types::FragmentSpread>,
    ) -> BindResult<BoundSelection> {
        let location = (*pos).try_into()?;
        // We always create a new selection set from a named fragment. It may not be split in the
        // same way and we need to validate the type condition each time.
        let name = spread.fragment_name.node.to_string();
        if self.current_fragments_stack.contains(&name) {
            self.current_fragments_stack.push(name);
            return Err(BindError::FragmentCycle {
                cycle: std::mem::take(&mut self.current_fragments_stack),
                location,
            });
        }

        // To please the borrow checker we remove the fragment temporarily from the hashmap and put
        // it back later. Not super efficient, but well, keeps things simple for now.
        let (fragment_name, mut fragment) =
            self.unbound_fragments
                .remove_entry(&name)
                .ok_or_else(|| BindError::UnknownFragment {
                    name: name.to_string(),
                    location,
                })?;
        self.current_fragments_stack.push(name.clone());

        let type_condition = self.bind_type_condition(root, &fragment.node.type_condition)?;
        let selection_set_id = self.bind_selection_set(type_condition.into(), &mut fragment.node.selection_set)?;
        let fragment_id = {
            let fragment_definition_location = fragment.pos.try_into()?;
            let next_id = BoundFragmentId::from(self.fragments.len());
            self.fragments
                .entry(name)
                .or_insert_with(|| {
                    // A bound fragment definition has no selection set, it was already bound. We
                    // only keep it for errors (name/name_pos) and directives.
                    let fragment_definition = BoundFragment {
                        name: fragment_name.clone(),
                        name_location: fragment_definition_location,
                        type_condition,
                        directives: Vec::new(),
                    };
                    (next_id, fragment_definition)
                })
                .0
        };

        self.current_fragments_stack.pop();
        self.unbound_fragments.insert(fragment_name, fragment);

        let fragment_spread_id = BoundFragmentSpreadId::from(self.fragment_spreads.len());
        self.fragment_spreads.push(BoundFragmentSpread {
            location,
            selection_set_id,
            fragment_id,
        });
        Ok(BoundSelection::FragmentSpread(fragment_spread_id))
    }

    fn bind_inline_fragment(
        &mut self,
        root: SelectionSetType,
        Positioned { pos, node: fragment }: &mut Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<BoundSelection> {
        let type_condition = fragment
            .type_condition
            .as_ref()
            .map(|condition| self.bind_type_condition(root, condition))
            .transpose()?;
        let fragment_root = type_condition.map(Into::into).unwrap_or(root);
        let selection_set_id = self.bind_selection_set(fragment_root, &mut fragment.selection_set)?;

        let inline_fragment_id = BoundInlineFragmentId::from(self.inline_fragments.len());
        self.inline_fragments.push(BoundInlineFragment {
            location: (*pos).try_into()?,
            type_condition,
            selection_set_id,
            directives: Vec::new(),
        });
        Ok(BoundSelection::InlineFragment(inline_fragment_id))
    }

    fn bind_type_condition(
        &self,
        root: SelectionSetType,
        Positioned { pos, node }: &Positioned<engine_parser::types::TypeCondition>,
    ) -> BindResult<TypeCondition> {
        let location = (*pos).try_into()?;
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
