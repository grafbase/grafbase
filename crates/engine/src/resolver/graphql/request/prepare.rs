use std::fmt::{Error, Write};

use grafbase_telemetry::graphql::OperationType;
use itertools::Itertools;
use operation::{OperationContext, QueryInputValueRecord, QueryOrSchemaInputValueId};
use schema::{CompositeType, EntityDefinition, GraphqlEndpointId, ObjectDefinition, SubgraphId, Type, TypeRecord};
use walker::Walk;

use crate::prepare::{PlanFieldArguments, PlanQueryPartition, PlanValueRecord, SubgraphField, SubgraphSelectionSet};

const VARIABLE_PREFIX: &str = "var";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedGraphqlOperation {
    pub ty: OperationType,
    pub query: String,
    pub variables: Vec<QueryVariable>,
}

impl PreparedGraphqlOperation {
    pub(crate) fn build(
        ctx: OperationContext<'_>,
        endpoint_id: GraphqlEndpointId,
        parent_object: ObjectDefinition<'_>,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> Result<PreparedGraphqlOperation, Error> {
        let mut builder = QueryBuilderContext::new(ctx, endpoint_id.into());
        let parent_object_id = Some(parent_object.id);
        let operation_type = if parent_object_id == Some(ctx.schema.query().id) {
            OperationType::Query
        } else if parent_object_id == ctx.schema.mutation().map(|m| m.id) {
            OperationType::Mutation
        } else if parent_object_id == ctx.schema.subscription().map(|s| s.id) {
            OperationType::Subscription
        } else {
            tracing::error!("Root GraphQL query on a non-root object?");
            return Err(Error);
        };

        // Generating the selection set first as this will define all the operation arguments
        let mut buffer = String::with_capacity(256);
        builder.write_selection_set(
            ParentType::CompositeType(parent_object.into()),
            &mut buffer,
            selection_set,
        )?;

        let mut query = String::with_capacity(buffer.len() + 14 + builder.estimated_variable_definitions_string_len);
        match operation_type {
            OperationType::Query => write!(query, "query")?,
            OperationType::Mutation => write!(query, "mutation")?,
            OperationType::Subscription => write!(query, "subscription")?,
        };

        if !builder.variables.is_empty() {
            query.push('(');
            builder.write_operation_arguments_without_parenthesis(&mut query)?;
            query.push(')');
        }

        query.push_str(&buffer);

        Ok(PreparedGraphqlOperation {
            ty: operation_type,
            query,
            variables: builder.into_query_variables(),
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedFederationEntityOperation {
    pub query: String,
    pub entities_variable_name: String,
    pub variables: Vec<QueryVariable>,
}

impl PreparedFederationEntityOperation {
    pub(crate) fn build(
        ctx: OperationContext<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> Result<Self, Error> {
        let mut builder = QueryBuilderContext::new(ctx, plan_query_partition.resolver_definition().subgraph_id());

        // Generating the selection set first as this will define all the operation arguments
        let mut selection_set = String::with_capacity(256);
        builder.write_selection_set(
            ParentType::Any,
            &mut selection_set,
            plan_query_partition.selection_set(),
        )?;

        let entities_variable_name = format!("{VARIABLE_PREFIX}{}", builder.variables.len());
        let mut query = String::with_capacity(
            // Rough approximation of the final string length counted by hand
            selection_set.len()
                + 60
                + builder.estimated_variable_definitions_string_len
                + 2 * entities_variable_name.len(),
        );
        query.push_str("query");
        query.push('(');
        write!(query, "${entities_variable_name}: [_Any!]!")?;

        if !builder.variables.is_empty() {
            query.push(',');
            builder.write_operation_arguments_without_parenthesis(&mut query)?;
        }
        query.push(')');

        write!(
            query,
            " {{ _entities(representations: ${entities_variable_name}){selection_set} }}"
        )?;

        Ok(PreparedFederationEntityOperation {
            query,
            entities_variable_name,
            variables: builder.into_query_variables(),
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryVariable {
    pub name: String,
    pub ty: TypeRecord,
    pub value: PlanValueRecord,
}

struct QueryBuilderContext<'ctx> {
    ctx: OperationContext<'ctx>,
    subgraph_id: SubgraphId,
    variables: Vec<QueryVariable>,
    estimated_variable_definitions_string_len: usize,
}

enum ParentType<'a> {
    Any,
    CompositeType(CompositeType<'a>),
}

impl<'ctx> QueryBuilderContext<'ctx> {
    fn new(ctx: OperationContext<'ctx>, subgraph_id: SubgraphId) -> Self {
        Self {
            ctx,
            subgraph_id,
            variables: Vec::new(),
            estimated_variable_definitions_string_len: 0,
        }
    }

    fn into_query_variables(self) -> Vec<QueryVariable> {
        self.variables
    }

    fn write_operation_arguments_without_parenthesis(&self, out: &mut String) -> Result<(), Error> {
        write!(
            out,
            "{}",
            self.variables.iter().format_with(", ", |var, f| {
                // no need to add the default value, we'll always provide the variable.
                f(&format_args!("${}: {}", var.name, var.ty.walk(self.ctx)))
            })
        )
    }

    fn write_selection_set(
        &mut self,
        parent_type: ParentType<'_>,
        buffer: &mut String,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> Result<(), Error> {
        buffer.push_str(" {");
        let n = buffer.len();
        if selection_set.requires_typename() {
            // We always need to know the concrete object.
            buffer.push_str(" __typename");
        }
        self.write_selection_set_fields(parent_type, buffer, selection_set)?;
        // If nothing was written it means only meta fields (__typename) are present and during
        // deserialization we'll expect an object. So adding `__typename` to ensure a non empty
        // selection set.
        if buffer.len() == n {
            buffer.push_str(" __typename @skip(if: true)");
        }
        buffer.push_str(" }");
        Ok(())
    }

    fn write_selection_set_fields(
        &mut self,
        parent_type: ParentType<'_>,
        buffer: &mut String,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> Result<(), Error> {
        let subgraph_id = self.subgraph_id;

        let ParentType::CompositeType(parent_type) = parent_type else {
            let entity_fields = selection_set
                .data_fields_ordered_by_parent_entity_then_key()
                .chunk_by(|field| field.definition().parent_entity());

            // Parent is Any or any other type that will accept anything.
            for (entity, fields) in entity_fields.into_iter() {
                self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
            }
            return Ok(());
        };

        if let CompositeType::Object(_) = parent_type {
            // From here one, it doesn't matter from where fields are coming, the
            // subgraph object must expose all the fields so we just request them directly
            // without any type conditions.
            self.write_fields(buffer, selection_set.fields())?;
            return Ok(());
        }

        let entity_fields = selection_set
            .data_fields_ordered_by_parent_entity_then_key()
            .chunk_by(|field| field.definition().parent_entity());

        let maybe_parent_interface_id = parent_type.as_interface().map(|interface| interface.id);
        let parent_is_fully_implemented = parent_type.is_fully_implemented_in_subgraph(subgraph_id);
        for (entity, fields) in entity_fields.into_iter() {
            let interface = match entity {
                EntityDefinition::Object(object) => {
                    match parent_type {
                        CompositeType::Interface(interface)
                            if object.implements_interface_in_subgraph(&subgraph_id, &interface.id) =>
                        {
                            self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                        }
                        CompositeType::Union(union) if union.has_member_in_subgraph(subgraph_id, object.id) => {
                            self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                        }
                        _ => (),
                    }
                    continue;
                }
                EntityDefinition::Interface(interface) => interface,
            };

            // If it's the same interface as the parent, there's nothing to do. It's either fully implemented
            // and we're good. Or it isn't and we can't retrieve missing objects from the parent anyway because
            // it's the same interface from our perspective.
            if Some(interface.id) == maybe_parent_interface_id {
                self.write_fields(buffer, fields)?;
                continue;
            }

            // If fully implemented, it's consistent with the our, the super-graph's, view.
            if interface.is_fully_implemented_in(subgraph_id) {
                self.write_type_condition_and_entity_fields(buffer, entity, fields)?;
                continue;
            }

            // From here on we know that some objects that implement the interface on our side
            // don't in the subgraphs. But we still want to retrieve them if they're part the
            // parent type.

            let fields = fields.collect::<Vec<_>>();
            let possible_subgraph_objects = interface
                .possible_types()
                .filter(|o| o.exists_in_subgraph(&subgraph_id));

            let mut add_interface_fragment = false;
            for object in possible_subgraph_objects {
                // For objects that do implement the interface, we add an interface fragment
                // instead to reduce query size.
                if object.implements_interface_in_subgraph(&subgraph_id, &interface.id) {
                    add_interface_fragment = true;
                } else if parent_is_fully_implemented
                    || parent_type.possible_types_include_in_subgraph(subgraph_id, object.id)
                {
                    // Here we know that the subgraph can provide this object but it doesn't
                    // implement the interface, so we need to add it separately.
                    self.write_type_condition_and_entity_fields(
                        buffer,
                        EntityDefinition::Object(object),
                        fields.iter().copied(),
                    )?;
                }
            }

            if add_interface_fragment {
                self.write_type_condition_and_entity_fields(buffer, entity, fields.iter().copied())?;
            }
        }

        Ok(())
    }

    fn write_type_condition_and_entity_fields<'a>(
        &mut self,
        buffer: &mut String,
        entity: EntityDefinition<'_>,
        fields: impl Iterator<Item = SubgraphField<'a>>,
    ) -> Result<(), Error> {
        write!(buffer, " ... on {} {{", entity.name())?;

        for field in fields {
            self.write_field(buffer, field)?;
        }

        buffer.push_str(" }");

        Ok(())
    }

    fn write_fields<'a>(
        &mut self,
        buffer: &mut String,
        fields: impl Iterator<Item = SubgraphField<'a>>,
    ) -> Result<(), Error> {
        for field in fields {
            self.write_field(buffer, field)?;
        }

        Ok(())
    }

    fn write_field(&mut self, buffer: &mut String, field: SubgraphField<'_>) -> Result<(), Error> {
        let response_key = field.subgraph_response_key_str();
        let name = field.definition().name();
        buffer.push(' ');
        if response_key == name {
            buffer.push_str(name);
        } else {
            write!(buffer, "{response_key}: {name}")?;
        }
        self.write_arguments(buffer, field.arguments())?;
        if let Some(ty) = field
            .definition()
            .subgraph_types()
            .find(|record| record.subgraph_id == self.subgraph_id)
            .and_then(|record| record.ty().definition().as_composite_type())
        {
            self.write_selection_set(ParentType::CompositeType(ty), buffer, field.selection_set())?;
        } else if let Some(ty) = field.definition().ty().definition().as_composite_type() {
            self.write_selection_set(ParentType::CompositeType(ty), buffer, field.selection_set())?;
        }
        Ok(())
    }

    fn write_arguments(&mut self, buffer: &mut String, arguments: PlanFieldArguments<'_>) -> Result<(), Error> {
        if arguments.len() != 0 {
            write!(
                buffer,
                "({})",
                arguments.into_iter().format_with(", ", |arg, f| {
                    // If the argument is a constant value that would still be present after query
                    // normalization we keep it to avoid adding unnecessary variables.
                    if let Some(value) = arg.value_as_sanitized_query_const_value_str() {
                        f(&format_args!("{}: {}", arg.definition().name(), value))
                    } else {
                        let idx = self.get_or_insert_var(arg.definition().ty(), arg.value_record);
                        f(&format_args!(
                            "{}: ${}",
                            arg.definition().name(),
                            self.variables[idx].name
                        ))
                    }
                })
            )?;
        }
        Ok(())
    }

    fn get_or_insert_var(&mut self, ty: Type<'_>, value: PlanValueRecord) -> usize {
        // For variables we re-use its name for clarity and its type. The latter is important to
        // avoid duplicate variables, as the argument may have a different type than the variable
        // and compare the type first to avoid expensive value comparison in the next step.
        let (name, ty) = match value {
            PlanValueRecord::Value(QueryOrSchemaInputValueId::Query(id)) => {
                if let QueryInputValueRecord::Variable(id) = self.ctx.operation[id] {
                    let var = &self.ctx.operation[id];
                    (Some(&var.name), var.ty_record.walk(self.ctx))
                } else {
                    (None, ty)
                }
            }
            _ => (None, ty),
        };

        let pos = self.variables.iter().position(|var| {
            if &var.ty != ty.as_ref() {
                return false;
            }
            let ctx = self.ctx;
            match (var.value, value) {
                (PlanValueRecord::Value(l), PlanValueRecord::Value(r)) => match (l, r) {
                    (QueryOrSchemaInputValueId::Query(lid), QueryOrSchemaInputValueId::Query(rid)) => {
                        operation::are_query_value_equivalent(ctx, &ctx.operation[lid], &ctx.operation[rid])
                    }
                    (QueryOrSchemaInputValueId::Query(qid), QueryOrSchemaInputValueId::Schema(sid))
                    | (QueryOrSchemaInputValueId::Schema(sid), QueryOrSchemaInputValueId::Query(qid)) => {
                        operation::is_query_value_equivalent_schema_value(ctx, &ctx.operation[qid], &ctx.schema[sid])
                    }
                    // Those are default values, so id equality should be enough
                    (QueryOrSchemaInputValueId::Schema(lid), QueryOrSchemaInputValueId::Schema(rid)) => lid == rid,
                },
                (PlanValueRecord::Injection(l), PlanValueRecord::Injection(r)) => l == r,
                _ => false,
            }
        });
        if let Some(pos) = pos {
            pos
        } else {
            let name = name.cloned().unwrap_or_else(|| {
                let mut prefix = String::new();
                loop {
                    let candidate = format!("{}{VARIABLE_PREFIX}{}", prefix, self.variables.len());
                    if self
                        .ctx
                        .operation
                        .variable_definitions
                        .iter()
                        .all(|def| def.name != candidate)
                    {
                        break candidate;
                    }
                    prefix.push('_');
                }
            });
            self.estimated_variable_definitions_string_len +=
                "$".len() + name.len() + ": ".len() + ty.definition().name().len() + "[!]!, ".len();
            self.variables.push(QueryVariable {
                name,
                ty: ty.into(),
                value,
            });
            self.variables.len() - 1
        }
    }
}
