use std::{collections::BTreeSet, mem::take};

use itertools::Itertools;

use crate::{
    EnumDefinitionId, FieldProvidesRecord, FieldRequiresRecord, Graph, InputObjectDefinitionId, InterfaceDefinitionId,
    JoinImplementsDefinitionRecord, JoinMemberDefinitionRecord, ScalarDefinitionId, SubgraphId, SubgraphTypeRecord,
    UnionDefinitionId,
    builder::{
        Error,
        sdl::{self, GraphName},
        subgraphs::SubgraphsBuilder,
    },
};

use super::DirectivesIngester;

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn ingest_federation_directives(&mut self, def: sdl::SdlDefinition<'sdl>, directives: &[sdl::Directive<'sdl>]) {
        match def {
            sdl::SdlDefinition::SchemaDirective(_) => unreachable!(),
            sdl::SdlDefinition::Scalar(def) => self.ingest_scalar_definition_federation_directive(def.id, directives),
            sdl::SdlDefinition::Object(def) => self.ingest_object_definition_federation_directive(def, directives),
            sdl::SdlDefinition::Interface(def) => {
                self.ingest_interface_definition_federation_directive(def, directives)
            }
            sdl::SdlDefinition::Union(def) => self.ingest_union_definition_federation_directive(def.id, directives),
            sdl::SdlDefinition::Enum(def) => self.ingest_enum_definition_federation_directive(def.id, directives),
            sdl::SdlDefinition::InputObject(def) => {
                self.ingest_input_object_definition_federation_directive(def.id, directives)
            }
            sdl::SdlDefinition::FieldDefinition(def) => self.ingest_field_federation_directives(def, directives),
            sdl::SdlDefinition::InputFieldDefinition(_)
            | sdl::SdlDefinition::ArgumentDefinition(_)
            | sdl::SdlDefinition::EnumValue(_) => {}
        }
    }

    fn ingest_enum_definition_federation_directive(
        &mut self,
        id: EnumDefinitionId,
        directives: &[sdl::Directive<'sdl>],
    ) {
        update_exists_in_subgraph_ids(
            &self.builder.ctx.subgraphs,
            &mut self.builder.graph[id].exists_in_subgraph_ids,
            directives,
            &mut self.errors,
        )
    }

    fn ingest_input_object_definition_federation_directive(
        &mut self,
        id: InputObjectDefinitionId,
        directives: &[sdl::Directive<'sdl>],
    ) {
        update_exists_in_subgraph_ids(
            &self.builder.ctx.subgraphs,
            &mut self.builder.graph[id].exists_in_subgraph_ids,
            directives,
            &mut self.errors,
        )
    }

    fn ingest_interface_definition_federation_directive(
        &mut self,
        def: sdl::InterfaceSdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) {
        if self.graph[def.id]
            .exists_in_subgraph_ids
            .contains(&SubgraphId::Introspection)
        {
            return;
        }

        let mut exists_in_subgraph_ids = take(&mut self.graph[def.id].exists_in_subgraph_ids);
        for result in directives.iter().filter_map(sdl::as_join_type) {
            let (join_type, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match self.subgraphs.try_get(join_type.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            exists_in_subgraph_ids.push(subgraph_id);
            if join_type.is_interface_object {
                self.graph[def.id].is_interface_object_in_ids.push(subgraph_id);
            }
        }
        if exists_in_subgraph_ids.is_empty() {
            exists_in_subgraph_ids = self.subgraphs.all.clone()
        } else {
            exists_in_subgraph_ids.sort_unstable();
        }
        self.graph[def.id].exists_in_subgraph_ids = exists_in_subgraph_ids;
    }

    fn ingest_object_definition_federation_directive(
        &mut self,
        def: sdl::ObjectSdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) {
        if self.graph[def.id]
            .exists_in_subgraph_ids
            .contains(&SubgraphId::Introspection)
        {
            return;
        }

        let mut join_implements = Vec::new();
        for result in directives.iter().filter_map(sdl::as_join_implements) {
            let (dir, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match self.subgraphs.try_get(dir.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            match self.definitions.get_interface_id(dir.interface, span) {
                Ok(interface_id) => {
                    join_implements.push(JoinImplementsDefinitionRecord {
                        subgraph_id,
                        interface_id,
                    });
                }
                Err(err) => self.errors.push(err),
            }
        }
        self.graph[def.id].join_implement_records = join_implements;

        self.graph[def.id]
            .join_implement_records
            .sort_by_key(|record| (record.subgraph_id, record.interface_id));

        let mut exists_in_subgraph_ids = take(&mut self.graph[def.id].exists_in_subgraph_ids);
        for result in directives.iter().filter_map(sdl::as_join_type) {
            let (join_type, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match self.subgraphs.try_get(join_type.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            exists_in_subgraph_ids.push(subgraph_id);
        }

        if exists_in_subgraph_ids.is_empty() {
            exists_in_subgraph_ids = self.subgraphs.all.clone()
        } else {
            exists_in_subgraph_ids.sort_unstable();
        }
        self.graph[def.id].exists_in_subgraph_ids = exists_in_subgraph_ids;
    }

    fn ingest_scalar_definition_federation_directive(
        &mut self,
        id: ScalarDefinitionId,
        directives: &[sdl::Directive<'sdl>],
    ) {
        update_exists_in_subgraph_ids(
            &self.builder.ctx.subgraphs,
            &mut self.builder.graph[id].exists_in_subgraph_ids,
            directives,
            &mut self.errors,
        )
    }

    fn ingest_union_definition_federation_directive(
        &mut self,
        id: UnionDefinitionId,
        directives: &[sdl::Directive<'sdl>],
    ) {
        if self.graph[id]
            .exists_in_subgraph_ids
            .contains(&SubgraphId::Introspection)
        {
            return;
        }

        let mut join_members = Vec::new();
        for result in directives.iter().filter_map(sdl::as_join_union_member) {
            let (dir, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match self.subgraphs.try_get(dir.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            match self.definitions.get_object_id(dir.member, span) {
                Ok(member_id) => {
                    join_members.push(JoinMemberDefinitionRecord { subgraph_id, member_id });
                }
                Err(err) => self.errors.push(err),
            }
        }
        self.graph[id].join_member_records = join_members;

        self.graph[id]
            .join_member_records
            .sort_by_key(|record| (record.subgraph_id, record.member_id));

        let mut exists_in_subgraph_ids = take(&mut self.builder.graph[id].exists_in_subgraph_ids);
        for result in directives.iter().filter_map(sdl::as_join_type) {
            let (join_type, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match self.subgraphs.try_get(join_type.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            exists_in_subgraph_ids.push(subgraph_id);
        }
        if exists_in_subgraph_ids.is_empty() {
            exists_in_subgraph_ids = self.builder.subgraphs.all.clone()
        } else {
            exists_in_subgraph_ids.sort_unstable();
        }
        self.graph[id].exists_in_subgraph_ids = exists_in_subgraph_ids;
    }

    fn ingest_field_federation_directives(
        &mut self,
        def: sdl::FieldSdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) {
        if self.graph[def.id]
            .exists_in_subgraph_ids
            .contains(&SubgraphId::Introspection)
        {
            return;
        }

        let field = &mut self.graph[def.id];
        let mut subgraph_type_records = take(&mut field.subgraph_type_records);
        let mut requires_records = take(&mut field.requires_records);
        let mut provides_records = take(&mut field.provides_records);
        // BTreeSet to ensures consistent ordering of resolvers.
        let mut resolvable_in = take(&mut field.exists_in_subgraph_ids)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let parent_entity_id = field.parent_entity_id;
        let ty_record = field.ty_record;

        let mut has_join_field = false;
        let mut overrides = Vec::new();
        for result in directives.iter().filter_map(sdl::as_join_field) {
            let (dir, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match dir.graph {
                Some(name) => match self.subgraphs.try_get(name, span) {
                    Ok(id) => Some(id),
                    Err(err) => {
                        self.errors.push(err);
                        continue;
                    }
                },
                None => None,
            };

            // If there is a @join__field we rely solely on that to define the subgraphs in
            // which this field exists. It may not specify a subgraph at all, in that case it's
            // a interfaceObject field.
            has_join_field = true;
            if let Some(subgraph_id) = subgraph_id {
                if let Some(ty_str) = dir.r#type {
                    let ty = match self.parse_type(ty_str, span) {
                        Ok(ty) => ty,
                        Err(err) => {
                            self.errors.push(err);
                            continue;
                        }
                    };
                    if ty != ty_record {
                        subgraph_type_records.push(SubgraphTypeRecord {
                            subgraph_id,
                            ty_record: ty,
                        });
                    }
                }
                if !dir.external {
                    if let Some(provides) = dir.provides.filter(|fields| !fields.is_empty()) {
                        let Some(parent) = ty_record.definition_id.as_composite_type() else {
                            self.errors.push(
                                Error::new(format!(
                                    "Field {}.{} cannot have @provides",
                                    def.parent.name(),
                                    def.name()
                                ))
                                .span(span),
                            );
                            continue;
                        };
                        let provides = match self.parse_field_set(parent, provides) {
                            Ok(fs) => fs,
                            Err(err) => {
                                self.errors.push(
                                    Error::new(format!(
                                        "At {}, invalid provides FieldSet: {err}",
                                        def.to_site_string(self)
                                    ))
                                    .span(span),
                                );
                                continue;
                            }
                        };
                        provides_records.push(FieldProvidesRecord {
                            subgraph_id,
                            field_set_record: provides,
                        });
                    }
                    if let Some(requires) = dir.requires.filter(|fields| !fields.is_empty()) {
                        let requires = match self.parse_field_set(parent_entity_id.into(), requires) {
                            Ok(fs) => fs,
                            Err(err) => {
                                self.errors.push(
                                    Error::new(format!(
                                        "At {}, invalid requires FieldSet: {err}",
                                        def.to_site_string(self)
                                    ))
                                    .span(span),
                                );
                                continue;
                            }
                        };
                        requires_records.push(FieldRequiresRecord {
                            subgraph_id,
                            field_set_record: requires,
                            injection_ids: Default::default(),
                        });
                    }
                    resolvable_in.insert(subgraph_id);
                }
            }

            if let Some(name) = dir.r#override
                && let Ok(graph) = self.subgraphs.try_get(GraphName(name), span)
            {
                overrides.push(graph);
            }
        }

        let mut parent_has_join_type = false;
        let mut parent_directives = Vec::new();
        parent_directives.extend(def.parent.directives());
        if let Some(ext) = self.sdl.type_extensions.get(def.parent.name()) {
            parent_directives.extend(ext.iter().flat_map(|ext| ext.directives()));
        }
        for result in parent_directives.iter().filter_map(sdl::as_join_type) {
            let (dir, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    self.errors.push(err);
                    continue;
                }
            };

            parent_has_join_type = true;
            if !has_join_field {
                let subgraph_id = match self.subgraphs.try_get(dir.graph, span) {
                    Ok(id) => id,
                    Err(err) => {
                        self.errors.push(err);
                        continue;
                    }
                };
                // If there is no @join__field we rely solely @join__type to define the subgraphs
                // in which this field is resolvable in.
                resolvable_in.insert(subgraph_id);
            }
        }

        // Remove any overridden subgraphs
        for subgraph_id in overrides.iter() {
            resolvable_in.remove(subgraph_id);
        }

        // If there is no @join__field and no @join__type at all, we assume this field to be
        // available everywhere.
        let exists_in_subgraph_ids = if !has_join_field && !parent_has_join_type {
            self.subgraphs.all.clone()
        } else {
            resolvable_in.into_iter().collect::<Vec<_>>()
        };

        let field = &mut self.graph[def.id];
        field.subgraph_type_records = subgraph_type_records;
        field.exists_in_subgraph_ids = exists_in_subgraph_ids;
        field.provides_records = provides_records;
        field.requires_records = requires_records;
    }
}

fn update_exists_in_subgraph_ids(
    subgraphs: &SubgraphsBuilder<'_>,
    exists_in_subgraph_ids: &mut Vec<SubgraphId>,
    directives: &[sdl::Directive<'_>],
    errors: &mut Vec<Error>,
) {
    if exists_in_subgraph_ids.contains(&SubgraphId::Introspection) {
        return;
    }

    for result in directives.iter().filter_map(sdl::as_join_type) {
        let (join_type, span) = match result {
            Ok(v) => v,
            Err(err) => {
                errors.push(err);
                continue;
            }
        };
        let subgraph_id = match subgraphs.try_get(join_type.graph, span) {
            Ok(id) => id,
            Err(err) => {
                errors.push(err);
                continue;
            }
        };
        exists_in_subgraph_ids.push(subgraph_id);
    }
    if exists_in_subgraph_ids.is_empty() {
        *exists_in_subgraph_ids = subgraphs.all.clone()
    } else {
        exists_in_subgraph_ids.sort_unstable();
    }
}

pub(super) fn add_not_fully_implemented_in(graph: &mut Graph) {
    let mut not_fully_implemented_in_ids = Vec::new();
    for (ix, interface) in graph.interface_definitions.iter_mut().enumerate() {
        let interface_id = InterfaceDefinitionId::from(ix);

        // For every possible type implementing this interface.
        for object_id in &interface.possible_type_ids {
            let object = &graph.object_definitions[usize::from(*object_id)];

            // Check in which subgraphs these are resolved.
            for subgraph_id in &interface.exists_in_subgraph_ids {
                // The object implements the interface if it defines az `@join__implements`
                // corresponding to the interface and to the subgraph.
                if object.implements_interface_in_subgraph(subgraph_id, &interface_id) {
                    continue;
                }

                not_fully_implemented_in_ids.push(*subgraph_id);
            }
        }

        not_fully_implemented_in_ids.sort_unstable();
        // Sorted by the subgraph id
        interface
            .not_fully_implemented_in_ids
            .extend(not_fully_implemented_in_ids.drain(..).dedup())
    }

    let mut exists_in_subgraph_ids = Vec::new();
    for union in graph.union_definitions.iter_mut() {
        exists_in_subgraph_ids.clear();
        exists_in_subgraph_ids.extend(union.join_member_records.iter().map(|join| join.subgraph_id));
        exists_in_subgraph_ids.sort_unstable();
        exists_in_subgraph_ids.dedup();

        for object_id in &union.possible_type_ids {
            for subgraph_id in exists_in_subgraph_ids.iter().copied() {
                // The object implements the interface if it defines az `@join__implements`
                // corresponding to the interface and to the subgraph.
                if union
                    .join_member_records
                    .binary_search_by(|probe| probe.subgraph_id.cmp(&subgraph_id).then(probe.member_id.cmp(object_id)))
                    .is_err()
                {
                    not_fully_implemented_in_ids.push(subgraph_id);
                }
            }
        }

        not_fully_implemented_in_ids.sort_unstable();
        // Sorted by the subgraph id
        union
            .not_fully_implemented_in_ids
            .extend(not_fully_implemented_in_ids.drain(..).dedup())
    }
}
