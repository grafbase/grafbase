use std::{
    collections::HashMap,
    fmt::{Error, Write},
};

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::{
    EntityDefinition, EntityDefinitionId, InterfaceDefinition, InterfaceDefinitionId, SubgraphId, UnionDefinition,
};

use crate::operation::{
    FieldArgumentsWalker, PlanField, PlanSelectionSet, PlanWalker, QueryInputValueId, SelectionSetType,
};

const VARIABLE_PREFIX: &str = "var";

macro_rules! indent_write {
    ($dst:ident, $($arg:tt)*) => {{
        $dst.write_indent();
        write!($dst, $($arg)*)
    }};
}

pub(crate) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: QueryVariables,
}

impl PreparedGraphqlOperation {
    pub(crate) fn build(
        operation_type: OperationType,
        plan: PlanWalker<'_>,
        subgraph_id: SubgraphId,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut ctx = QueryBuilderContext::new(subgraph_id);

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::with_capacity(256);
            let entity_id = EntityDefinitionId::Object(match operation_type {
                OperationType::Query => plan.schema().query().id(),
                OperationType::Mutation => plan.schema().mutation().unwrap().id(),
                OperationType::Subscription => plan.schema().subscription().unwrap().id(),
            });

            ctx.write_selection_set(None, Some(entity_id), &mut buffer, plan.selection_set())?;

            buffer.into_string()
        };

        let mut query = String::with_capacity(selection_set.len() + 14 + ctx.estimated_variable_definitions_string_len);
        match operation_type {
            OperationType::Query => write!(query, "query")?,
            OperationType::Mutation => write!(query, "mutation")?,
            OperationType::Subscription => write!(query, "subscription")?,
        };

        if !ctx.variables.is_empty() {
            query.push('(');
            ctx.write_operation_arguments_without_parenthesis(&mut query)?;
            query.push(')');
        }

        query.push_str(&selection_set);

        Ok(PreparedGraphqlOperation {
            ty: operation_type,
            query,
            variables: ctx.into_query_variables(),
        })
    }
}

pub(crate) struct PreparedFederationEntityOperation {
    pub query: String,
    pub entities_variable_name: String,
    pub variables: QueryVariables,
}

impl PreparedFederationEntityOperation {
    pub(crate) fn build(plan: PlanWalker<'_>, subgraph_id: SubgraphId) -> Result<Self, Error> {
        let mut ctx = QueryBuilderContext::new(subgraph_id);

        // Generating the selection set first as this will define all the operation arguments
        let selection_set = {
            let mut buffer = Buffer::with_capacity(256);
            buffer.indent += 1;
            ctx.write_selection_set(None, None, &mut buffer, plan.selection_set())?;
            buffer.into_string()
        };

        let entities_variable_name = format!("{VARIABLE_PREFIX}{}", ctx.variables.len());
        let mut query = String::with_capacity(
            // Rough approximation of the final string length counted by hand
            selection_set.len() + 60 + ctx.estimated_variable_definitions_string_len + 2 * entities_variable_name.len(),
        );
        query.push_str("query");
        query.push('(');
        write!(query, "${entities_variable_name}: [_Any!]!")?;

        if !ctx.variables.is_empty() {
            query.push(',');
            ctx.write_operation_arguments_without_parenthesis(&mut query)?;
        }
        query.push(')');

        write!(
            query,
            " {{\n  _entities(representations: ${entities_variable_name}){selection_set}}}"
        )?;

        Ok(PreparedFederationEntityOperation {
            query,
            entities_variable_name,
            variables: ctx.into_query_variables(),
        })
    }
}

/// All variables associated with a subgraph query. Each one is associated with the variable name
/// "{$VARIABLE_PREFIX}{idx}" with `idx` being the position of the input value in the inner vec.
pub(crate) struct QueryVariables(Vec<QueryInputValueId>);

impl QueryVariables {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, QueryInputValueId)> + '_ {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, &id)| (format!("{VARIABLE_PREFIX}{}", idx), id))
    }
}

struct QueryVariable {
    idx: usize,
    ty: String,
}

#[derive(Debug, Clone, Copy)]
enum SelectionSetRendering<'a> {
    // Return type is an interface and we use an interface fragment to a subgraph that does
    // not implement the interface to all types
    InterfaceWithPartialFragment(InterfaceDefinition<'a>),
    // Return type is a union and we use an interface fragment to a subgraph that does
    // not implement the interface to all types
    UnionWithPartialFragment(UnionDefinition<'a>, InterfaceDefinition<'a>),
    // Return type is an interface, we have to filter out all entities not implementing the interface.
    InterfaceWithObjects(InterfaceDefinitionId),
    // No extra rendering measures needed.
    Other,
}

impl<'a> SelectionSetRendering<'a> {
    fn new(
        subgraph_id: SubgraphId,
        parent_entity_id: Option<EntityDefinitionId>,
        selection_set_type: Option<SelectionSetType>,
        entity: schema::EntityDefinition<'a>,
        selection_set: PlanSelectionSet<'a>,
    ) -> Self {
        let in_same_entity = parent_entity_id == Some(entity.id());

        match (selection_set_type, entity) {
            (Some(SelectionSetType::Interface(_)), schema::EntityDefinition::Interface(interface))
                if interface.is_not_fully_implemented_in(subgraph_id) && !in_same_entity =>
            {
                Self::InterfaceWithPartialFragment(interface)
            }
            (Some(SelectionSetType::Union(union_id)), schema::EntityDefinition::Interface(interface))
                if interface.is_not_fully_implemented_in(subgraph_id) && !in_same_entity =>
            {
                Self::UnionWithPartialFragment(selection_set.walker().schema().walk(union_id), interface)
            }
            (Some(SelectionSetType::Interface(interface_id)), _) => Self::InterfaceWithObjects(interface_id),
            _ => Self::Other,
        }
    }
}

struct QueryBuilderContext {
    subgraph_id: SubgraphId,
    variables: HashMap<QueryInputValueId, QueryVariable>,
    estimated_variable_definitions_string_len: usize,
}

impl QueryBuilderContext {
    fn new(subgraph_id: SubgraphId) -> Self {
        Self {
            subgraph_id,
            variables: HashMap::new(),
            estimated_variable_definitions_string_len: 0,
        }
    }

    fn into_query_variables(self) -> QueryVariables {
        let mut vars = vec![None; self.variables.len()];
        for (input_value_id, var) in self.variables {
            vars[var.idx] = Some(input_value_id);
        }

        QueryVariables(vars.into_iter().map(Option::unwrap).collect())
    }

    fn write_operation_arguments_without_parenthesis(&self, out: &mut String) -> Result<(), Error> {
        write!(
            out,
            "{}",
            self.variables.values().format_with(", ", |var, f| {
                // no need to add the default value, we'll always provide the variable.
                f(&format_args!("${VARIABLE_PREFIX}{}: {}", var.idx, var.ty))
            })
        )
    }

    fn write_selection_set(
        &mut self,
        maybe_selection_set_type: Option<SelectionSetType>,
        maybe_entity_id: Option<EntityDefinitionId>,
        buffer: &mut Buffer,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        buffer.write_str(" {\n")?;
        buffer.indent += 1;
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            indent_write!(buffer, "__typename\n")?;
        }
        self.write_selection_set_fields(maybe_selection_set_type, maybe_entity_id, buffer, selection_set)?;
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            indent_write!(buffer, "__typename\n")?;
        }
        buffer.indent -= 1;
        indent_write!(buffer, "}}\n")
    }

    fn write_selection_set_fields(
        &mut self,
        maybe_selection_set_type: Option<SelectionSetType>,
        maybe_entity_id: Option<EntityDefinitionId>,
        buffer: &mut Buffer,
        selection_set: PlanSelectionSet<'_>,
    ) -> Result<(), Error> {
        let subgraph_id = self.subgraph_id;
        let entity_to_fields = selection_set
            .fields_ordered_by_parent_entity_id_then_position()
            .into_iter()
            .chunk_by(|field| field.definition().parent_entity_id);

        for (entity_id, fields) in entity_to_fields.into_iter() {
            let fields = fields.collect_vec();
            let entity = selection_set.walker().schema().walk(entity_id);
            let in_same_entity = maybe_entity_id == Some(entity_id);

            let rendering = SelectionSetRendering::new(
                self.subgraph_id,
                maybe_entity_id,
                maybe_selection_set_type,
                entity,
                selection_set,
            );

            let mut add_interface_fragment = false;

            match rendering {
                SelectionSetRendering::InterfaceWithPartialFragment(interface) => {
                    let objects = interface
                        .possible_types_ordered_by_typename()
                        .filter(|o| o.is_resolvable_in(&subgraph_id));

                    for object in objects {
                        if object.subgraph_implements_interface(&subgraph_id, &interface.id()) {
                            add_interface_fragment = true;
                        } else {
                            self.write_type_fields(buffer, object.name(), &fields)?;
                        }
                    }
                }
                SelectionSetRendering::UnionWithPartialFragment(union, interface) => {
                    let objects = union
                        .possible_types_ordered_by_typename()
                        .filter(|o| o.is_resolvable_in(&subgraph_id));

                    for object in objects {
                        if object.subgraph_implements_interface(&subgraph_id, &interface.id()) {
                            add_interface_fragment = true;
                        } else {
                            self.write_type_fields(buffer, object.name(), &fields)?;
                        }
                    }
                }
                SelectionSetRendering::InterfaceWithObjects(interface_id) => {
                    if let EntityDefinition::Object(ref object) = entity {
                        if !object.subgraph_implements_interface(&subgraph_id, &interface_id) {
                            continue;
                        }
                    }

                    add_interface_fragment = true;
                }
                SelectionSetRendering::Other => {
                    add_interface_fragment = true;
                }
            }

            if add_interface_fragment {
                self.write_entity_fields(in_same_entity, buffer, entity, &fields)?;
            }
        }

        Ok(())
    }

    fn write_entity_fields(
        &mut self,
        in_same_entity: bool,
        buffer: &mut Buffer,
        entity: EntityDefinition<'_>,
        fields: &[PlanWalker<'_, crate::operation::FieldId>],
    ) -> Result<(), Error> {
        if !in_same_entity {
            indent_write!(buffer, "... on {} {{\n", entity.name())?;
            buffer.indent += 1;
        }

        for field in fields {
            self.write_field(buffer, *field)?;
        }

        if !in_same_entity {
            buffer.indent -= 1;
            indent_write!(buffer, "}}\n")?;
        }

        Ok(())
    }

    fn write_type_fields(
        &mut self,
        buffer: &mut Buffer,
        type_name: &str,
        fields: &[PlanWalker<'_, crate::operation::FieldId>],
    ) -> Result<(), Error> {
        indent_write!(buffer, "... on {} {{\n", type_name)?;
        buffer.indent += 1;

        for field in fields {
            self.write_field(buffer, *field)?;
        }

        buffer.indent -= 1;
        indent_write!(buffer, "}}\n")?;

        Ok(())
    }

    fn write_field(&mut self, buffer: &mut Buffer, field: PlanField<'_>) -> Result<(), Error> {
        let response_key = field.response_key_str();
        let name = field.definition().name();
        if response_key == name {
            indent_write!(buffer, "{name}")?;
        } else {
            indent_write!(buffer, "{response_key}: {name}")?;
        }
        self.write_arguments(buffer, field.arguments())?;
        if let Some(selection_set) = field.selection_set() {
            self.write_selection_set(
                SelectionSetType::maybe_from(field.definition().ty().definition().id()),
                EntityDefinitionId::maybe_from(field.definition().ty().definition().id()),
                buffer,
                selection_set,
            )?;
        } else {
            buffer.push('\n');
        }
        Ok(())
    }

    fn write_arguments(&mut self, buffer: &mut Buffer, arguments: FieldArgumentsWalker<'_>) -> Result<(), Error> {
        if !arguments.is_empty() {
            write!(
                buffer,
                "({})",
                arguments.into_iter().format_with(", ", |arg, f| {
                    // If the argument is a constant value that would still be present after query
                    // normalization we keep it to avoid adding unnecessary variables.
                    if let Some(value) = arg
                        .value()
                        .and_then(|value| value.to_normalized_query_const_value_str())
                    {
                        f(&format_args!("{}: {}", arg.definition().name(), value))
                    } else {
                        let idx = self.variables.len();
                        let var = self.variables.entry(arg.as_ref().input_value_id).or_insert_with(|| {
                            let ty = arg.definition().ty().to_string();
                            // prefix + ': ' + index (2) + ',' + ty.len()
                            self.estimated_variable_definitions_string_len += VARIABLE_PREFIX.len() + 5 + ty.len();
                            QueryVariable { idx, ty }
                        });
                        f(&format_args!(
                            "{}: ${VARIABLE_PREFIX}{}",
                            arg.definition().name(),
                            var.idx
                        ))
                    }
                })
            )?;
        }
        Ok(())
    }
}

#[derive(Hash, PartialEq, Eq)]
struct Buffer {
    inner: String,
    indent: usize,
}

impl std::ops::Deref for Buffer {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Buffer {
    fn with_capacity(capacity: usize) -> Self {
        Buffer {
            inner: String::with_capacity(capacity),
            indent: 0,
        }
    }

    fn into_string(self) -> String {
        self.inner
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.inner.push(' ');
            self.inner.push(' ');
        }
    }
}
