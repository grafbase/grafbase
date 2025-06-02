mod cost;
mod deprecated;
mod list_size;
mod requires_scopes;

use crate::{
    Graph, TypeDefinitionId, TypeSystemDirectiveId, UnionDefinitionId,
    builder::{Error, sdl},
};

use super::DirectivesIngester;

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn ingest_non_federation_aware_directives(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) -> Result<(), Error> {
        let mut directive_ids = Vec::new();

        let mut inaccessible = false;
        for &directive in directives {
            match directive.name() {
                "inaccessible" => inaccessible = true,
                "authenticated" => directive_ids.push(TypeSystemDirectiveId::Authenticated),
                "requiresScopes" => {
                    directive_ids.push(self.create_requires_scopes_directive(def, directive)?);
                }
                "deprecated" => {
                    directive_ids.push(self.create_deprecated_directive(def, directive)?);
                }
                "cost" => {
                    directive_ids.push(self.create_cost_directive(def, directive)?);
                }
                "listSize" => {
                    directive_ids.push(self.create_list_size_directive(def, directive)?);
                }
                "oneOf" => {
                    let sdl::SdlDefinition::InputObject(_) = def else {
                        return Err(("@oneOf can only be used on input objects.", directive.name_span()).into());
                    };
                    // Only directive to be processed immediately as rely on it for default values.
                }
                "extension__directive" if !self.for_operation_analytics_only => {
                    let dir = sdl::parse_extension_directive(directive)?;
                    let subgraph_id = self.subgraphs.try_get(dir.graph, directive.arguments_span())?;
                    let extension = self.extensions.get(dir.extension);
                    let id = self
                        .ingest_extension_directive(def, subgraph_id, extension, dir.name, dir.arguments)
                        .map_err(|txt| (txt, directive.arguments_span()))?;
                    directive_ids.push(TypeSystemDirectiveId::Extension(id))
                }
                name if name.starts_with("composite__") && !self.for_operation_analytics_only => self
                    .ingest_composite_directive_before_federation(def, directive)
                    .map_err(|err| err.with_span_if_absent(directive.arguments_span()))?,
                _ => {}
            };
        }

        match def {
            sdl::SdlDefinition::SchemaDirective(_) => unreachable!(), // Handled separately
            sdl::SdlDefinition::Scalar(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_scalar_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Object(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                if inaccessible {
                    self.graph.inaccessible_object_definitions.set(def.id, true);
                    for interface_id in &self.builder.graph.object_definitions[usize::from(def.id)].interface_ids {
                        self.builder
                            .graph
                            .interface_has_inaccessible_implementor
                            .set(*interface_id, true);
                    }
                }
            }
            sdl::SdlDefinition::Interface(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_interface_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Union(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_union_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::Enum(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_enum_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::InputObject(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible_input_object_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::FieldDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_field_definitions.set(def.id, inaccessible);
            }
            sdl::SdlDefinition::InputFieldDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible_input_value_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::ArgumentDefinition(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph
                    .inaccessible_input_value_definitions
                    .set(def.id, inaccessible);
            }
            sdl::SdlDefinition::EnumValue(def) => {
                self.graph[def.id].directive_ids = directive_ids;
                self.graph.inaccessible_enum_values.set(def.id, inaccessible);
            }
        }

        Ok(())
    }
}

pub(super) fn finalize_inaccessible(graph: &mut Graph) {
    // Must be done after ingesting all @inaccessible for objects.
    for (ix, union) in graph.union_definitions.iter().enumerate() {
        let id = UnionDefinitionId::from(ix);
        for possible_type in &union.possible_type_ids {
            if graph.inaccessible_object_definitions[*possible_type] {
                graph.union_has_inaccessible_member.set(id, true);
                break;
            }
        }
    }

    // Any field or input_value having an inaccessible type is marked as inaccessible.
    // Composition should ensure all of this is consistent, but we ensure it.
    fn is_definition_inaccessible(graph: &Graph, definition_id: TypeDefinitionId) -> bool {
        match definition_id {
            TypeDefinitionId::Scalar(id) => graph.inaccessible_scalar_definitions[id],
            TypeDefinitionId::Object(id) => graph.inaccessible_object_definitions[id],
            TypeDefinitionId::Interface(id) => graph.inaccessible_interface_definitions[id],
            TypeDefinitionId::Union(id) => graph.inaccessible_union_definitions[id],
            TypeDefinitionId::Enum(id) => graph.inaccessible_enum_definitions[id],
            TypeDefinitionId::InputObject(id) => graph.inaccessible_input_object_definitions[id],
        }
    }

    for (ix, field) in graph.field_definitions.iter().enumerate() {
        if is_definition_inaccessible(graph, field.ty_record.definition_id) {
            graph.inaccessible_field_definitions.set(ix.into(), true);
        }
    }

    for (ix, input_value) in graph.input_value_definitions.iter().enumerate() {
        if is_definition_inaccessible(graph, input_value.ty_record.definition_id)
            || input_value.is_internal_in_id.is_some()
        {
            graph.inaccessible_input_value_definitions.set(ix.into(), true);
        }
    }
}
