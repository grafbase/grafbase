use id_newtypes::BitSet;
use url::Url;

use crate::{
    DirectiveSiteId, EntityDefinitionId, EnumDefinitionId, InputObjectDefinitionId, InputValueParentDefinitionId,
    InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId, Schema, TypeDefinitionId, UnionDefinitionId, UrlId,
    builder,
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
    pub fn mark_all_as_inaccessible(&mut self) {
        self.0.graph.inaccessible_object_definitions.set_all(true);
        self.0.graph.inaccessible_interface_definitions.set_all(true);
        self.0.graph.inaccessible_field_definitions.set_all(true);
        self.0.graph.inaccessible_enum_definitions.set_all(true);
        self.0.graph.inaccessible_enum_values.set_all(true);
        self.0.graph.inaccessible_input_object_definitions.set_all(true);
        self.0.graph.inaccessible_input_value_definitions.set_all(true);
        self.0.graph.inaccessible_scalar_definitions.set_all(true);
        self.0.graph.inaccessible_union_definitions.set_all(true);
    }

    pub fn mark_as_accessible(&mut self, site_id: DirectiveSiteId, accessible: bool) {
        match site_id {
            DirectiveSiteId::Enum(id) => {
                self.0.graph.inaccessible_enum_definitions.set(id, !accessible);
            }
            DirectiveSiteId::EnumValue(id) => {
                self.0.graph.inaccessible_enum_values.set(id, !accessible);
                if accessible {
                    let id = self.walk(id).parent_enum_id;
                    self.0.graph.inaccessible_enum_definitions.set(id, false);
                }
            }
            DirectiveSiteId::Field(id) => {
                self.0.graph.inaccessible_field_definitions.set(id, !accessible);
                if accessible {
                    match self.walk(id).parent_entity_id {
                        EntityDefinitionId::Interface(id) => {
                            self.0.graph.inaccessible_interface_definitions.set(id, false);
                        }
                        EntityDefinitionId::Object(id) => {
                            self.0.graph.inaccessible_object_definitions.set(id, false);
                        }
                    }
                }
            }
            DirectiveSiteId::InputObject(id) => {
                self.0.graph.inaccessible_input_object_definitions.set(id, !accessible);
            }
            DirectiveSiteId::InputValue(id) => {
                self.0.graph.inaccessible_input_value_definitions.set(id, !accessible);
                if accessible {
                    if let InputValueParentDefinitionId::InputObject(id) = self.walk(id).parent_id {
                        self.0.graph.inaccessible_input_object_definitions.set(id, false);
                    }
                }
            }
            DirectiveSiteId::Interface(id) => {
                self.0.graph.inaccessible_interface_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Object(id) => {
                self.0.graph.inaccessible_object_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Scalar(id) => {
                self.0.graph.inaccessible_scalar_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Union(id) => {
                self.0.graph.inaccessible_union_definitions.set(id, !accessible);
            }
        }
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
        // reset
        self.0.graph.union_has_inaccessible_member.set_all(false);
        self.0.graph.interface_has_inaccessible_implementor.set_all(false);
        if hide_unreachable_types {
            self::hide_unreachable_types(&mut self.0);
        }
        mark_builtins_and_introspection_as_accessible(&mut self.0);
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
                if schema.graph.inaccessible_object_definitions[id] {
                    continue;
                }
                if reached_object_definitions.put(id) {
                    continue;
                }
                let object = &schema[id];

                for field in &schema[object.field_ids] {
                    stack.push(field.ty_record.definition_id);

                    for arg in &schema[field.argument_ids] {
                        stack.push(arg.ty_record.definition_id);
                    }
                }

                // Process interfaces implemented
                for interface_id in &object.interface_ids {
                    stack.push(TypeDefinitionId::Interface(*interface_id));
                }
            }
            TypeDefinitionId::Interface(id) => {
                if schema.graph.inaccessible_interface_definitions[id] {
                    continue;
                }
                if reached_interface_definitions.put(id) {
                    continue;
                }
                let interface = &schema[id];

                for field in &schema[interface.field_ids] {
                    stack.push(field.ty_record.definition_id);

                    for arg in &schema[field.argument_ids] {
                        stack.push(arg.ty_record.definition_id);
                    }
                }

                // Process implementing objects
                for object_id in &interface.possible_type_ids {
                    stack.push(TypeDefinitionId::Object(*object_id));
                }
            }
            TypeDefinitionId::Union(id) => {
                if schema.graph.inaccessible_union_definitions[id] {
                    continue;
                }
                if reached_union_definitions.put(id) {
                    continue;
                }
                let union = &schema[id];

                for object_id in &union.possible_type_ids {
                    stack.push(TypeDefinitionId::Object(*object_id));
                }
            }
            TypeDefinitionId::Enum(id) => {
                if schema.graph.inaccessible_enum_definitions[id] {
                    continue;
                }
                if reached_enum_definitions.put(id) {
                    continue;
                }
            }
            TypeDefinitionId::Scalar(id) => {
                if schema.graph.inaccessible_scalar_definitions[id] {
                    continue;
                }
                if reached_scalar_definitions.put(id) {
                    continue;
                }
            }
            TypeDefinitionId::InputObject(id) => {
                if schema.graph.inaccessible_input_object_definitions[id] {
                    continue;
                }
                if reached_input_object_definitions.put(id) {
                    continue;
                }
                let input_object = &schema[id];

                for field in &schema[input_object.input_field_ids] {
                    stack.push(field.ty_record.definition_id);
                }
            }
        }
    }

    // Mark unreachable types as inaccessible
    schema.graph.inaccessible_object_definitions |= !reached_object_definitions;
    schema.graph.inaccessible_union_definitions |= !reached_union_definitions;
    schema.graph.inaccessible_interface_definitions |= !reached_interface_definitions;
    schema.graph.inaccessible_scalar_definitions |= !reached_scalar_definitions;
    schema.graph.inaccessible_enum_definitions |= !reached_enum_definitions;
    schema.graph.inaccessible_input_object_definitions |= !reached_input_object_definitions;
}

fn mark_builtins_and_introspection_as_accessible(schema: &mut Schema) {
    for (ix, scalar) in schema.graph.scalar_definitions.iter().enumerate() {
        let id = ScalarDefinitionId::from(ix);
        if ["Boolean", "String", "Int", "Float"].contains(&schema[scalar.name_id].as_str()) {
            schema.graph.inaccessible_scalar_definitions.set(id, false);
        }
    }

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
                schema.graph.inaccessible_object_definitions.set(id, false);
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
                schema.graph.inaccessible_interface_definitions.set(id, false);
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
                schema.graph.inaccessible_union_definitions.set(id, false);
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
                schema.graph.inaccessible_enum_definitions.set(id, false);
                stack.extend(schema[id].value_ids.into_iter().map(DirectiveSiteId::from));
            }
            DirectiveSiteId::Scalar(id) => {
                if reached_scalar_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible_scalar_definitions.set(id, false);
            }
            DirectiveSiteId::InputObject(id) => {
                if reached_input_object_definitions.put(id) {
                    continue;
                }
                schema.graph.inaccessible_input_object_definitions.set(id, false);
                let input_object = &schema[id];
                stack.extend(input_object.input_field_ids.into_iter().map(DirectiveSiteId::from));
            }
            DirectiveSiteId::EnumValue(enum_value_id) => {
                schema.graph.inaccessible_enum_values.set(enum_value_id, false);
            }
            DirectiveSiteId::Field(field_definition_id) => {
                schema
                    .graph
                    .inaccessible_field_definitions
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
                    .inaccessible_input_value_definitions
                    .set(input_value_definition_id, false);
                stack.push(schema[input_value_definition_id].ty_record.definition_id.into());
            }
        }
    }
}
