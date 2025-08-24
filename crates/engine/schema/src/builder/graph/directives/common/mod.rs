mod cost;
mod deprecated;
mod list_size;

use id_newtypes::IdToMany;

use crate::{
    CompositeTypeId, EntityDefinitionId, EnumDefinitionId, FieldDefinitionId, Graph, InputObjectDefinitionId,
    InputValueDefinitionId, InputValueParentDefinitionId, InterfaceDefinitionId, ObjectDefinitionId, TypeDefinitionId,
    TypeSystemDirectiveId, UnionDefinitionId,
    builder::{Error, sdl},
};

use super::DirectivesIngester;

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn ingest_non_federation_aware_directives(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) {
        let mut directive_ids = Vec::new();

        let mut inaccessible = false;
        for &directive in directives {
            match directive.name() {
                "inaccessible" => inaccessible = true,
                "deprecated" => match self.create_deprecated_directive(def, directive) {
                    Ok(id) => directive_ids.push(id),
                    Err(err) => self.errors.push(err),
                },
                "cost" => match self.create_cost_directive(def, directive) {
                    Ok(id) => directive_ids.push(id),
                    Err(err) => self.errors.push(err),
                },
                "listSize" => match self.create_list_size_directive(def, directive) {
                    Ok(id) => directive_ids.push(id),
                    Err(err) => self.errors.push(err),
                },
                "oneOf" => {
                    let sdl::SdlDefinition::InputObject(_) = def else {
                        self.errors
                            .push(Error::new("@oneOf can only be used on input objects.").span(directive.name_span()));
                        continue;
                    };
                    // Only directive to be processed immediately as rely on it for default values.
                }
                "extension__directive" if !self.for_operation_analytics_only => {
                    match sdl::parse_extension_directive(directive) {
                        Ok(dir) => match self.subgraphs.try_get(dir.graph, directive.arguments_span()) {
                            Ok(subgraph_id) => {
                                let extension = self.extensions.get(dir.extension);
                                match self.ingest_extension_directive(
                                    def,
                                    subgraph_id,
                                    extension,
                                    dir.name,
                                    dir.arguments,
                                ) {
                                    Ok(id) => directive_ids.push(TypeSystemDirectiveId::Extension(id)),
                                    Err(txt) => self.errors.push(Error::new(txt).span(directive.arguments_span())),
                                }
                            }
                            Err(err) => self.errors.push(err),
                        },
                        Err(err) => self.errors.push(err),
                    }
                }
                name if name.starts_with("composite__") && !self.for_operation_analytics_only => {
                    if let Err(err) = self.ingest_composite_directive_before_federation(def, directive) {
                        self.errors.push(err.span_if_absent(directive.arguments_span()));
                    }
                }
                _ => {}
            };
        }

        match def {
            sdl::SdlDefinition::SchemaDirective(_) => unreachable!(), // Handled separately
            sdl::SdlDefinition::Scalar(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.scalar_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Object(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.object_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Interface(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.interface_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Union(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.union_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Enum(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.enum_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::InputObject(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible
                    .input_object_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::FieldDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.field_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::InputFieldDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible
                    .input_value_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::ArgumentDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible
                    .input_value_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::EnumValue(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible.enum_values.set(def.id, inaccessible);
            }
        }
    }
}

// We don't always have full control over the schema, when coming from a file or schema contract, so we ensure that inaccessibility use is
// consistent and ends up in a usable schema.
pub(in crate::builder) fn finalize_inaccessible(graph: &mut Graph) {
    // If a field argument/input field is inaccessible AND required then the parent field/input object must also
    // be inaccessible as there is no way for a client to provide the necessary inputs to avoid a
    // validation failure.
    let mut output_to_input_value = Vec::with_capacity(graph.input_value_definitions.len() >> 2);
    for (ix, input_value) in graph.input_value_definitions.iter().enumerate() {
        let id = InputValueDefinitionId::from(ix);
        let is_output_inaccessible = match input_value.ty_record.definition_id {
            TypeDefinitionId::Scalar(id) => graph.inaccessible.scalar_definitions[id],
            TypeDefinitionId::Object(id) => graph.inaccessible.object_definitions[id],
            TypeDefinitionId::Interface(id) => graph.inaccessible.interface_definitions[id],
            TypeDefinitionId::Union(id) => graph.inaccessible.union_definitions[id],
            TypeDefinitionId::Enum(id) => graph.inaccessible.enum_definitions[id],
            TypeDefinitionId::InputObject(input_object_id) => {
                output_to_input_value.push((input_object_id, id));
                graph.inaccessible.input_object_definitions[input_object_id]
            }
        };

        let is_inaccessible = if is_output_inaccessible || input_value.is_internal_in_id.is_some() {
            graph.inaccessible.input_value_definitions.set(id, true);
            true
        } else {
            graph.inaccessible.input_value_definitions[id]
        };
        // We don't propagate inaccessible input values if they're internal and thus not part
        // of the schema.
        if is_inaccessible && input_value.ty_record.is_required() && input_value.is_internal_in_id.is_none() {
            match input_value.parent_id {
                InputValueParentDefinitionId::Field(field_id) => {
                    graph.inaccessible.field_definitions.set(field_id, true);
                }
                InputValueParentDefinitionId::InputObject(input_object_id) => {
                    graph.inaccessible.input_object_definitions.set(input_object_id, true)
                }
            }
        }
    }

    // Propagating upwards inaccessibility if an input_value is required.
    let mut inaccessible_input_objects = graph.inaccessible.input_object_definitions.ones().collect::<Vec<_>>();
    let output_to_input_value = IdToMany::from(output_to_input_value);
    while let Some(id) = inaccessible_input_objects.pop() {
        for &id in output_to_input_value.find_all(id) {
            if !graph.inaccessible.input_value_definitions.put(id) {
                let input_value = &graph.input_value_definitions[usize::from(id)];
                if input_value.ty_record.is_required() && input_value.is_internal_in_id.is_none() {
                    match input_value.parent_id {
                        InputValueParentDefinitionId::Field(field_id) => {
                            graph.inaccessible.field_definitions.set(field_id, true);
                        }
                        InputValueParentDefinitionId::InputObject(input_object_id) => {
                            if !graph.inaccessible.input_object_definitions.put(input_object_id) {
                                inaccessible_input_objects.push(input_object_id);
                            }
                        }
                    }
                }
            }
        }
    }

    // Interfaces, unions, enums and input objects must have at least one field/member.
    for ix in 0..graph.object_definitions.len() {
        let id = ObjectDefinitionId::from(ix);
        if graph.inaccessible.object_definitions[id] {
            continue;
        }

        if graph[id]
            .field_ids
            .into_iter()
            .all(|id| graph.inaccessible.field_definitions[id])
        {
            graph.inaccessible.object_definitions.set(id, true);
        }
    }

    for ix in 0..graph.interface_definitions.len() {
        let id = InterfaceDefinitionId::from(ix);
        if graph.inaccessible.interface_definitions[id] {
            continue;
        }

        if graph[id]
            .field_ids
            .into_iter()
            .all(|id| graph.inaccessible.field_definitions[id])
        {
            graph.inaccessible.interface_definitions.set(id, true);
        }
    }

    let mut object_to_union = Vec::new();
    for ix in 0..graph.union_definitions.len() {
        let id = UnionDefinitionId::from(ix);
        if graph.inaccessible.union_definitions[id] {
            continue;
        }
        if graph[id]
            .possible_type_ids
            .iter()
            .all(|id| graph.inaccessible.object_definitions[*id])
        {
            graph.inaccessible.union_definitions.set(id, true);
        } else {
            object_to_union.extend(graph[id].possible_type_ids.iter().map(|object_id| (*object_id, id)));
        }
    }
    let object_to_unions = IdToMany::from(object_to_union);

    for ix in 0..graph.enum_definitions.len() {
        let id = EnumDefinitionId::from(ix);
        if graph.inaccessible.enum_definitions[id] {
            continue;
        }
        if graph[id]
            .value_ids
            .into_iter()
            .all(|id| graph.inaccessible.enum_values[id])
        {
            graph.inaccessible.enum_definitions.set(id, true);
        }
    }

    for ix in 0..graph.input_object_definitions.len() {
        let id = InputObjectDefinitionId::from(ix);
        if graph.inaccessible.input_object_definitions[id] {
            continue;
        }
        if graph[id]
            .input_field_ids
            .into_iter()
            .all(|id| graph.inaccessible.input_value_definitions[id])
        {
            graph.inaccessible.input_object_definitions.set(id, true);
        }
    }

    // Any field or input_value having an inaccessible type is marked as inaccessible.
    let mut newly_inaccessible_types = Vec::<CompositeTypeId>::new();
    let mut composite_type_to_fields = Vec::<(CompositeTypeId, FieldDefinitionId)>::new();
    for (ix, field) in graph.field_definitions.iter().enumerate() {
        let id = FieldDefinitionId::from(ix);
        let is_output_inaccessible = match field.ty_record.definition_id {
            TypeDefinitionId::Scalar(id) => graph.inaccessible.scalar_definitions[id],
            TypeDefinitionId::Object(object_id) => {
                composite_type_to_fields.push((object_id.into(), id));
                graph.inaccessible.object_definitions[object_id]
            }
            TypeDefinitionId::Interface(interface_id) => {
                composite_type_to_fields.push((interface_id.into(), id));
                graph.inaccessible.interface_definitions[interface_id]
            }
            TypeDefinitionId::Union(union_id) => {
                composite_type_to_fields.push((union_id.into(), id));
                graph.inaccessible.union_definitions[union_id]
            }
            TypeDefinitionId::Enum(id) => graph.inaccessible.enum_definitions[id],
            TypeDefinitionId::InputObject(_) => unreachable!(),
        };
        if is_output_inaccessible && !graph.inaccessible.field_definitions.put(id) {
            match graph[id].parent_entity_id {
                EntityDefinitionId::Interface(id) => {
                    if graph[id]
                        .field_ids
                        .into_iter()
                        .all(|id| graph.inaccessible.field_definitions[id])
                        && !graph.inaccessible.interface_definitions.put(id)
                    {
                        newly_inaccessible_types.push(id.into());
                    }
                }
                EntityDefinitionId::Object(id) => {
                    if graph[id]
                        .field_ids
                        .into_iter()
                        .all(|id| graph.inaccessible.field_definitions[id])
                        && !graph.inaccessible.object_definitions.put(id)
                    {
                        newly_inaccessible_types.push(id.into());
                    }
                }
            }
        }
    }

    // Propagating upwards inaccessibility if all fields of an entity are inaccessible or if an
    // union has no accessible members.
    let composite_type_to_fields = IdToMany::from(composite_type_to_fields);
    while let Some(id) = newly_inaccessible_types.pop() {
        if let Some(id) = id.as_object() {
            for &union_id in object_to_unions.find_all(id) {
                if graph[union_id]
                    .possible_type_ids
                    .iter()
                    .all(|id| graph.inaccessible.object_definitions[*id])
                    && !graph.inaccessible.union_definitions.put(union_id)
                {
                    newly_inaccessible_types.push(union_id.into());
                }
            }
        }
        for &field_id in composite_type_to_fields.find_all(id) {
            if !graph.inaccessible.field_definitions.put(field_id) {
                match graph[field_id].parent_entity_id {
                    EntityDefinitionId::Interface(id) => {
                        if graph[id]
                            .field_ids
                            .into_iter()
                            .all(|id| graph.inaccessible.field_definitions[id])
                            && !graph.inaccessible.interface_definitions.put(id)
                        {
                            newly_inaccessible_types.push(id.into());
                        }
                    }
                    EntityDefinitionId::Object(id) => {
                        if graph[id]
                            .field_ids
                            .into_iter()
                            .all(|id| graph.inaccessible.field_definitions[id])
                            && !graph.inaccessible.object_definitions.put(id)
                        {
                            newly_inaccessible_types.push(id.into());
                        }
                    }
                }
            }
        }
    }

    graph.union_has_inaccessible_member.set_all(false);
    graph.interface_has_inaccessible_implementor.set_all(false);

    // Must be done after ingesting all @inaccessible for objects.
    for (ix, union) in graph.union_definitions.iter().enumerate() {
        let id = UnionDefinitionId::from(ix);
        for possible_type in &union.possible_type_ids {
            if graph.inaccessible.object_definitions[*possible_type] {
                graph.union_has_inaccessible_member.set(id, true);
                break;
            }
        }
    }

    for (ix, interface) in graph.interface_definitions.iter().enumerate() {
        let id = ix.into();
        for implementor in &interface.possible_type_ids {
            if graph.inaccessible.object_definitions[*implementor] {
                graph.interface_has_inaccessible_implementor.set(id, true);
                break;
            }
        }
    }
}
