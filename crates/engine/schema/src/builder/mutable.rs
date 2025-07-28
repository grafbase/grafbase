use id_newtypes::BitSet;
use url::Url;

use crate::{
    DirectiveSiteId, EnumDefinitionId, Inaccessible, InputObjectDefinitionId, InterfaceDefinitionId,
    ObjectDefinitionId, ScalarDefinitionId, Schema, TypeDefinitionId, UnionDefinitionId, UrlId, builder,
};

pub struct MutableSchema(Schema);

impl std::ops::Deref for MutableSchema {
    type Target = Schema;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Schema {
    pub fn into_mutable(self) -> MutableSchema {
        MutableSchema(self)
    }
}

impl MutableSchema {
    pub fn new_inaccessible(&self) -> Inaccessible {
        Inaccessible {
            object_definitions: BitSet::with_capacity(self.graph.object_definitions.len()),
            interface_definitions: BitSet::with_capacity(self.graph.interface_definitions.len()),
            field_definitions: BitSet::with_capacity(self.graph.field_definitions.len()),
            enum_definitions: BitSet::with_capacity(self.graph.enum_definitions.len()),
            enum_values: BitSet::with_capacity(self.graph.enum_values.len()),
            input_object_definitions: BitSet::with_capacity(self.graph.input_object_definitions.len()),
            input_value_definitions: BitSet::with_capacity(self.graph.input_value_definitions.len()),
            scalar_definitions: BitSet::with_capacity(self.graph.scalar_definitions.len()),
            union_definitions: BitSet::with_capacity(self.graph.union_definitions.len()),
        }
    }

    pub fn update_inaccessible(&mut self, contract: Inaccessible) {
        let current = &mut self.0.graph.inaccessible;
        current.object_definitions |= contract.object_definitions;
        current.interface_definitions |= contract.interface_definitions;
        current.field_definitions |= contract.field_definitions;
        current.enum_definitions |= contract.enum_definitions;
        current.enum_values |= contract.enum_values;
        current.input_object_definitions |= contract.input_object_definitions;
        current.input_value_definitions |= contract.input_value_definitions;
        current.scalar_definitions |= contract.scalar_definitions;
        current.union_definitions |= contract.union_definitions;
    }

    pub fn update_graphql_endpoint(&mut self, name: &str, url: Url) {
        let Some(id) = self
            .0
            .subgraphs()
            .filter_map(|sg| sg.as_graphql_endpoint())
            .find(|gql| gql.subgraph_name() == name)
            .filter(|gql| gql.url() == &url)
            .map(|gql| gql.id)
        else {
            return;
        };
        let url_id = self.get_or_insert_url(url);
        self.0[id].url_id = url_id;
    }

    fn get_or_insert_url(&mut self, url: Url) -> UrlId {
        if let Some(pos) = self.0.urls.iter().position(|candidate| candidate == &url) {
            return pos.into();
        }
        self.0.urls.push(url);
        UrlId::from(self.0.urls.len() - 1)
    }

    pub fn finalize(mut self, hide_unreachable_types: bool) -> Schema {
        mark_builtins_and_introspection_as_accessible(&mut self.0);
        if hide_unreachable_types {
            self::hide_unreachable_types(&mut self.0);
        }
        builder::finalize_inaccessible(&mut self.0.graph);
        self.0
    }
}

fn hide_unreachable_types(schema: &mut Schema) {
    let mut reached_object_definitions =
        BitSet::<ObjectDefinitionId>::with_capacity(schema.graph.object_definitions.len());
    let mut reached_union_definitions =
        BitSet::<UnionDefinitionId>::with_capacity(schema.graph.union_definitions.len());
    let mut reached_interface_definitions =
        BitSet::<InterfaceDefinitionId>::with_capacity(schema.graph.interface_definitions.len());
    let mut reached_scalar_definitions =
        BitSet::<ScalarDefinitionId>::with_capacity(schema.graph.scalar_definitions.len());
    let mut reached_enum_definitions = BitSet::<EnumDefinitionId>::with_capacity(schema.graph.enum_definitions.len());
    let mut reached_input_object_definitions =
        BitSet::<InputObjectDefinitionId>::with_capacity(schema.graph.input_object_definitions.len());

    let mut stack = vec![TypeDefinitionId::from(
        schema.graph.root_operation_types_record.query_id,
    )];
    if let Some(mutation_id) = schema.graph.root_operation_types_record.mutation_id {
        stack.push(TypeDefinitionId::from(mutation_id));
    }
    if let Some(subscription_id) = schema.graph.root_operation_types_record.subscription_id {
        stack.push(TypeDefinitionId::from(subscription_id));
    }

    while let Some(type_id) = stack.pop() {
        match type_id {
            TypeDefinitionId::Object(id) => {
                let object = schema.walk(id);
                if object.is_inaccessible() {
                    continue;
                }
                if reached_object_definitions.put(id) {
                    continue;
                }

                for field in object.fields() {
                    if field.is_inaccessible() {
                        continue;
                    }
                    stack.push(field.ty_record.definition_id);

                    for arg in field.arguments() {
                        if arg.is_inaccessible() {
                            continue;
                        }
                        stack.push(arg.ty_record.definition_id);
                    }
                }

                // Process interfaces implemented
                for interface_id in &object.interface_ids {
                    stack.push(TypeDefinitionId::Interface(*interface_id));
                }
            }
            TypeDefinitionId::Interface(id) => {
                let interface = schema.walk(id);
                if interface.is_inaccessible() {
                    continue;
                }
                if reached_interface_definitions.put(id) {
                    continue;
                }

                for field in interface.fields() {
                    if field.is_inaccessible() {
                        continue;
                    }
                    stack.push(field.ty_record.definition_id);
                    for arg in field.arguments() {
                        if arg.is_inaccessible() {
                            continue;
                        }
                        stack.push(arg.ty_record.definition_id);
                    }
                }

                // Process implementing objects
                for object_id in &interface.possible_type_ids {
                    stack.push(TypeDefinitionId::Object(*object_id));
                }
            }
            TypeDefinitionId::Union(id) => {
                let union = schema.walk(id);
                if union.is_inaccessible() {
                    continue;
                }
                if reached_union_definitions.put(id) {
                    continue;
                }

                for object_id in &union.possible_type_ids {
                    stack.push(TypeDefinitionId::Object(*object_id));
                }
            }
            TypeDefinitionId::Enum(id) => {
                if schema.graph.inaccessible.enum_definitions[id] {
                    continue;
                }
                if reached_enum_definitions.put(id) {
                    continue;
                }
            }
            TypeDefinitionId::Scalar(id) => {
                if schema.graph.inaccessible.scalar_definitions[id] {
                    continue;
                }
                if reached_scalar_definitions.put(id) {
                    continue;
                }
            }
            TypeDefinitionId::InputObject(id) => {
                let input_object = schema.walk(id);
                if input_object.is_inaccessible() {
                    continue;
                }
                if reached_input_object_definitions.put(id) {
                    continue;
                }
                for input_field in input_object.input_fields() {
                    if input_field.is_inaccessible() {
                        continue;
                    }
                    stack.push(input_field.ty_record.definition_id);
                }
            }
        }
    }

    // Mark unreachable types as inaccessible
    schema.graph.inaccessible.object_definitions |= !reached_object_definitions;
    schema.graph.inaccessible.union_definitions |= !reached_union_definitions;
    schema.graph.inaccessible.interface_definitions |= !reached_interface_definitions;
    schema.graph.inaccessible.scalar_definitions |= !reached_scalar_definitions;
    schema.graph.inaccessible.enum_definitions |= !reached_enum_definitions;
    schema.graph.inaccessible.input_object_definitions |= !reached_input_object_definitions;
}

pub(super) fn mark_builtins_and_introspection_as_accessible(schema: &mut Schema) {
    // Built-in scalars are never accessible
    for (ix, scalar) in schema.graph.scalar_definitions.iter().enumerate() {
        let id = ScalarDefinitionId::from(ix);
        if ["Boolean", "String", "Int", "Float", "ID"].contains(&schema[scalar.name_id].as_str()) {
            schema.graph.inaccessible.scalar_definitions.set(id, false);
        }
    }

    // Root types are never inaccessible
    schema
        .graph
        .inaccessible
        .object_definitions
        .set(schema.graph.root_operation_types_record.query_id, false);
    if let Some(mutation_id) = schema.graph.root_operation_types_record.mutation_id {
        schema.graph.inaccessible.object_definitions.set(mutation_id, false);
    }
    if let Some(subscription_id) = schema.graph.root_operation_types_record.subscription_id {
        schema.graph.inaccessible.object_definitions.set(subscription_id, false);
    }

    // Introspection types are never inaccessible
    let mut reached_object_definitions =
        BitSet::<ObjectDefinitionId>::with_capacity(schema.graph.object_definitions.len());
    let mut reached_union_definitions =
        BitSet::<UnionDefinitionId>::with_capacity(schema.graph.union_definitions.len());
    let mut reached_interface_definitions =
        BitSet::<InterfaceDefinitionId>::with_capacity(schema.graph.interface_definitions.len());
    let mut reached_scalar_definitions =
        BitSet::<ScalarDefinitionId>::with_capacity(schema.graph.scalar_definitions.len());
    let mut reached_enum_definitions = BitSet::<EnumDefinitionId>::with_capacity(schema.graph.enum_definitions.len());
    let mut reached_input_object_definitions =
        BitSet::<InputObjectDefinitionId>::with_capacity(schema.graph.input_object_definitions.len());

    let mut stack = schema
        .subgraphs
        .introspection
        .meta_fields
        .iter()
        .map(|id| DirectiveSiteId::from(*id))
        .collect::<Vec<_>>();

    while let Some(id) = stack.pop() {
        match id {
            DirectiveSiteId::Object(id) => {
                if reached_object_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.object_definitions.set(id, false);
                let object = &schema[id];

                stack.extend(object.field_ids.into_iter().map(DirectiveSiteId::from));
                stack.extend(
                    object
                        .interface_ids
                        .iter()
                        .map(|interface_id| DirectiveSiteId::Interface(*interface_id)),
                );
            }
            DirectiveSiteId::Interface(id) => {
                if reached_interface_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.interface_definitions.set(id, false);
                let interface = &schema[id];

                stack.extend(interface.field_ids.into_iter().map(DirectiveSiteId::from));
                stack.extend(
                    interface
                        .possible_type_ids
                        .iter()
                        .map(|object_id| DirectiveSiteId::Object(*object_id)),
                );
            }
            DirectiveSiteId::Union(id) => {
                if reached_union_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.union_definitions.set(id, false);
                let union = &schema[id];

                stack.extend(
                    union
                        .possible_type_ids
                        .iter()
                        .map(|object_id| DirectiveSiteId::Object(*object_id)),
                );
            }
            DirectiveSiteId::Enum(id) => {
                if reached_enum_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.enum_definitions.set(id, false);
                stack.extend(schema[id].value_ids.into_iter().map(DirectiveSiteId::from));
            }
            DirectiveSiteId::Scalar(id) => {
                if reached_scalar_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.scalar_definitions.set(id, false);
            }
            DirectiveSiteId::InputObject(id) => {
                if reached_input_object_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible.input_object_definitions.set(id, false);
                let input_object = &schema[id];
                stack.extend(input_object.input_field_ids.into_iter().map(DirectiveSiteId::from));
            }
            DirectiveSiteId::EnumValue(enum_value_id) => {
                schema.graph.inaccessible.enum_values.set(enum_value_id, false);
            }
            DirectiveSiteId::Field(field_definition_id) => {
                schema
                    .graph
                    .inaccessible
                    .field_definitions
                    .set(field_definition_id, false);
                stack.extend(
                    schema[field_definition_id]
                        .argument_ids
                        .into_iter()
                        .map(DirectiveSiteId::from),
                );
                stack.push(schema[field_definition_id].ty_record.definition_id.into());
            }
            DirectiveSiteId::InputValue(input_value_definition_id) => {
                schema
                    .graph
                    .inaccessible
                    .input_value_definitions
                    .set(input_value_definition_id, false);
                stack.push(schema[input_value_definition_id].ty_record.definition_id.into());
            }
        }
    }
}
