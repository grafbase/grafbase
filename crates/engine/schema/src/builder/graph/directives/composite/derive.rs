use cynic_parser_deser::ConstDeserializer as _;
use itertools::Itertools as _;

use crate::{
    DeriveDefinitionRecord, DeriveMappingRecord, DeriveObjectFieldRecord, DeriveObjectRecord,
    DeriveScalarAsFieldRecord, DirectiveSiteId, EntityDefinitionId, FieldDefinitionId, FieldSetRecord, Graph,
    ResolverDefinitionId, ResolverDefinitionRecord, SubgraphId, TypeRecord,
    builder::{
        BoundSelectedObjectField, BoundSelectedObjectValue, BoundSelectedValueEntry, DirectivesIngester, Error,
        GraphBuilder, PossibleCompositeEntityKey, SelectedValueOrField,
        sdl::{self, IsDirective},
    },
};

#[tracing::instrument(skip_all, fields(field=%def.name()))]
pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::FieldSdlDefinition<'sdl>,
    directive: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    let sdl::DerivedDirective { graph } = directive.deserialize().map_err(|err| {
        (
            format!(
                "At {}, invalid composite__lookup directive: {}",
                def.to_site_string(ingester),
                err
            ),
            directive.arguments_span(),
        )
    })?;
    let subgraph_id = ingester.subgraphs.try_get(graph, directive.arguments_span())?;

    let target = ingester.graph[def.id].ty_record;
    let Some(target_entity_id) = target.definition_id.as_entity() else {
        return Err((
            "@derive can only be used on fields to compute an object/interface.",
            def.span(),
        )
            .into());
    };
    let target_field_ids = match target_entity_id {
        EntityDefinitionId::Interface(id) => ingester.graph[id].field_ids,
        EntityDefinitionId::Object(id) => ingester.graph[id].field_ids,
    };
    let source_id = ingester.graph[def.id].parent_entity_id;
    let Some(possible_keys) = ingester
        .possible_composite_entity_keys
        .get(&(target_entity_id, subgraph_id))
    else {
        let ty = ingester.sdl_definitions[&target_entity_id.into()].as_type().unwrap();
        return Err((
            format!(
                "Type {} doesn't define any keys with @key directive that may be used for @derive",
                ty.name()
            ),
            ty.span(),
        )
            .into());
    };

    let ctx = DeriveContext {
        builder: ingester.builder,
        subgraph_id,
        def,
        directive,
        source_id,
        target,
        possible_keys,
    };

    let sdl_field = ingester.sdl_definitions[&DirectiveSiteId::Field(def.id)];
    let mut is_directives = sdl_field
        .directives()
        .filter(|dir| dir.name() == "composite__is")
        .map(|dir| {
            dir.deserialize::<IsDirective>()
                .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                .map(|args| (dir, args))
        })
        .filter_ok(|(_, args)| args.graph == graph);

    let is_directive = is_directives.next().transpose()?;
    if is_directives.next().is_some() {
        return Err((
            "Multiple @composite__is directives on the same field are not supported.",
            sdl_field.span(),
        )
            .into());
    }

    let definition = if let Some((
        is_directive,
        sdl::IsDirective {
            field: field_selection_map,
            ..
        },
    )) = is_directive
    {
        ctx.explicit_derive(is_directive, field_selection_map)
    } else {
        ctx.auto_detect_derive()
    }?;

    if let DeriveMappingRecord::Object(DeriveObjectRecord { field_records }) = &definition.mapping_record {
        for id in target_field_ids {
            if field_records.iter().any(|record| record.to_id == id) {
                continue;
            }
            let field = &ingester.graph[id];
            if field.exists_in_subgraph_ids.contains(&subgraph_id)
                && !field
                    .resolver_ids
                    .iter()
                    .any(|id| get_subgraph_id(&ingester.graph, *id) != subgraph_id)
            {
                return Err((
                    format!(
                        "Field {}.{} is unprovidable for this @derive",
                        ingester[ingester.definition_name_id(source_id.into())],
                        ingester[field.name_id]
                    ),
                    directive.name_span(),
                )
                    .into());
            }
        }
    }

    assert!(
        ingester.graph[def.id].derive_ids.is_empty(),
        "Suppor derived on multiple subgraphs."
    );
    let start = ingester.graph.derive_definitions.len();
    ingester.graph.derive_definitions.push(definition);
    ingester.graph[def.id].derive_ids = (start..(start + 1)).into();

    Ok(())
}

struct DeriveContext<'a, 'sdl> {
    builder: &'a mut GraphBuilder<'sdl>,
    subgraph_id: SubgraphId,
    def: sdl::FieldSdlDefinition<'sdl>,
    directive: sdl::Directive<'sdl>,
    source_id: EntityDefinitionId,
    target: TypeRecord,
    possible_keys: &'a [PossibleCompositeEntityKey<'sdl>],
}

fn get_subgraph_id(graph: &Graph, id: ResolverDefinitionId) -> SubgraphId {
    match &graph[id] {
        ResolverDefinitionRecord::FieldResolverExtension(record) => graph[record.directive_id].subgraph_id,
        ResolverDefinitionRecord::GraphqlFederationEntity(record) => record.endpoint_id.into(),
        ResolverDefinitionRecord::GraphqlRootField(record) => record.endpoint_id.into(),
        ResolverDefinitionRecord::Introspection => SubgraphId::Introspection,
        ResolverDefinitionRecord::Lookup(id) => get_subgraph_id(graph, graph[*id].resolver_id),
        ResolverDefinitionRecord::SelectionSetResolverExtension(record) => record.subgraph_id.into(),
    }
}

impl DeriveContext<'_, '_> {
    fn explicit_derive(
        self,
        is_directive: sdl::Directive<'_>,
        field_selection_map: &str,
    ) -> Result<DeriveDefinitionRecord, Error> {
        let Self {
            builder,
            subgraph_id,
            def,
            directive,
            source_id,
            possible_keys,
            ..
        } = self;

        let value = builder
            .parse_field_selection_map_for_derived_field(source_id, subgraph_id, def.id, field_selection_map)
            .map_err(|err| {
                (
                    format!("for associated @is directive: {err}"),
                    is_directive.arguments_span(),
                )
            })?
            .into_single()
            .ok_or_else(|| {
                (
                    "for associated @is directive, derived fields do not support multiple alternatives",
                    is_directive.arguments_span(),
                )
            })?;

        match value {
            BoundSelectedValueEntry::List { path: Some(path), list } => {
                let Some(batch_field_id) = path.into_single() else {
                    return Err("Derived field from a list cannot be nested".into());
                };
                let Some(BoundSelectedValueEntry::Object { path: None, object }) = list.0.into_single() else {
                    return Err("Derived field from a list cannot be nested".into());
                };

                let field = &builder.graph[batch_field_id];
                let mapping_record = if field.ty_record.definition_id.is_composite_type() {
                    DeriveMappingRecord::Object(create_explicit_object_mapping(
                        builder,
                        object,
                        possible_keys.iter().map(|key| &key.key),
                    )?)
                } else {
                    if object.fields.len() != 1 {
                        return Err("A scalar key can only be mapped to a single field for @derive".into());
                    }
                    let BoundSelectedObjectField { id: field_id, value } = object.fields.into_iter().next().unwrap();
                    if value
                        .into_value()
                        .and_then(|value| value.into_single())
                        .is_some_and(|value| matches!(value, BoundSelectedValueEntry::Identity))
                    {
                        DeriveMappingRecord::ScalarAsField(DeriveScalarAsFieldRecord { field_id })
                    } else {
                        return Err("A scalar key can only be mapped to a single field for @derive".into());
                    }
                };

                Ok(DeriveDefinitionRecord {
                    subgraph_id,
                    batch_field_id: Some(batch_field_id),
                    mapping_record,
                })
            }
            BoundSelectedValueEntry::Object { path: None, object } => {
                let object = create_explicit_object_mapping(builder, object, possible_keys.iter().map(|key| &key.key))?;
                Ok(DeriveDefinitionRecord {
                    subgraph_id,
                    batch_field_id: None,
                    mapping_record: DeriveMappingRecord::Object(object),
                })
            }
            _ => Err((
                "Unsupported mapping for @derive field. Nested fields are not supported.",
                directive.arguments_span(),
            )
                .into()),
        }
    }

    fn auto_detect_derive(self) -> Result<DeriveDefinitionRecord, Error> {
        let Self {
            builder,
            subgraph_id,
            def,
            directive,
            source_id,
            target,
            possible_keys,
            ..
        } = self;

        let prefix = if target.wrapping.is_list() {
            def.name().strip_suffix('s').unwrap_or(def.name())
        } else {
            def.name()
        };
        let possible_source_fields = match source_id {
            EntityDefinitionId::Interface(id) => builder.graph[id].field_ids,
            EntityDefinitionId::Object(id) => builder.graph[id].field_ids,
        }
        .into_iter()
        .filter(|id| *id != def.id)
        .filter_map(|id| {
            let field = &builder.graph[id];
            builder[field.name_id]
                .strip_prefix(prefix)
                .map(|suffix| (suffix.replace('_', "").to_lowercase(), id, field.ty_record))
        });

        if target.wrapping.is_list() {
            let mut matches = possible_source_fields
                .filter_map(|(name, batch_field_id, batch_field_ty)| {
                    let ty = batch_field_ty.without_list()?;
                    if let Some(possible_source_id) = ty.definition_id.as_entity() {
                        let possible_nested_fields = match possible_source_id {
                            EntityDefinitionId::Interface(id) => builder.graph[id].field_ids,
                            EntityDefinitionId::Object(id) => builder.graph[id].field_ids,
                        }
                        .into_iter()
                        .filter(|id| *id != def.id)
                        .map(|id| {
                            let field = &builder.graph[id];
                            let name = builder[field.name_id].replace('_', "").to_lowercase();
                            (name, id, field.ty_record)
                        })
                        .collect();
                        detect_object_mapping(
                            builder,
                            possible_nested_fields,
                            possible_keys.iter().map(|key| &key.key),
                        )
                        .map(|object| vec![(batch_field_id, DeriveMappingRecord::Object(object))])
                    } else if ty.definition_id.is_scalar() {
                        Some(
                            possible_keys
                                .iter()
                                .filter_map(
                                    |PossibleCompositeEntityKey { key, .. }| {
                                        if key.len() == 1 { Some(key) } else { None }
                                    },
                                )
                                .filter_map(|key| {
                                    let key_item = key.iter().next().unwrap();
                                    let key_field = &builder.selections[key_item.field_id];
                                    let def = &builder.graph[key_field.definition_id];
                                    if (name == builder[def.name_id]
                                        || name.strip_suffix("s").is_some_and(|name| name == builder[def.name_id]))
                                        && def.ty_record == ty
                                    {
                                        Some((
                                            batch_field_id,
                                            DeriveMappingRecord::ScalarAsField(DeriveScalarAsFieldRecord {
                                                field_id: key_field.definition_id,
                                            }),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    }
                })
                .flatten();
            let Some((batch_field_id, mapping_record)) = matches.next() else {
                return Err(("Derived field must match at least one @key", directive.arguments_span()).into());
            };
            if let Some(other) = matches.next() {
                if batch_field_id != other.0 {
                    return Err(format!(
                        "matched multiple batch fields: {} and {}",
                        builder[builder.graph[batch_field_id].name_id], builder[builder.graph[other.0].name_id],
                    )
                    .into());
                } else {
                    return Err(format!(
                        "ambiguous match for field {}",
                        builder[builder.graph[batch_field_id].name_id],
                    )
                    .into());
                }
            }
            Ok(DeriveDefinitionRecord {
                subgraph_id,
                batch_field_id: Some(batch_field_id),
                mapping_record,
            })
        } else {
            let Some(object) = detect_object_mapping(
                builder,
                possible_source_fields.collect(),
                possible_keys.iter().map(|key| &key.key),
            ) else {
                return Err("Derived field must match at least one @key".into());
            };
            Ok(DeriveDefinitionRecord {
                subgraph_id,
                batch_field_id: None,
                mapping_record: DeriveMappingRecord::Object(object),
            })
        }
    }
}

fn detect_object_mapping<'k>(
    builder: &GraphBuilder<'_>,
    possible_source_fields: Vec<(String, FieldDefinitionId, TypeRecord)>,
    keys: impl IntoIterator<Item = &'k FieldSetRecord>,
) -> Option<DeriveObjectRecord> {
    let mut field_records = Vec::new();
    let mut possible_mapping_records = Vec::new();

    'keys: for key in keys {
        possible_mapping_records.clear();
        if possible_source_fields.len() < key.len() {
            continue;
        }

        for item in key {
            if !item.subselection_record.is_empty() {
                continue 'keys;
            }
            let key_field_id = builder.selections[item.field_id].definition_id;
            let key_field = &builder.graph[key_field_id];
            let key_name = builder[key_field.name_id].to_lowercase().replace('_', "");

            let mut matches = possible_source_fields
                .iter()
                .filter(|(name, _, ty)| *name == key_name && *ty == key_field.ty_record);

            // Find matching field by name and type
            if let (Some((_, parent_field_id, _)), None) = (matches.next(), matches.next()) {
                possible_mapping_records.push(DeriveObjectFieldRecord {
                    from_id: *parent_field_id,
                    to_id: key_field_id,
                });
            } else {
                continue 'keys;
            }
        }
        field_records.append(&mut possible_mapping_records);
    }

    if field_records.is_empty() {
        return None;
    }

    field_records.sort_unstable();
    field_records.dedup();
    Some(DeriveObjectRecord { field_records })
}

fn create_explicit_object_mapping<'k>(
    builder: &GraphBuilder<'_>,
    object: BoundSelectedObjectValue<FieldDefinitionId>,
    keys: impl IntoIterator<Item = &'k FieldSetRecord>,
) -> Result<DeriveObjectRecord, Error> {
    let mut field_records = Vec::new();
    for BoundSelectedObjectField { id: to_id, value } in object.fields {
        let from_id = match value {
            SelectedValueOrField::Value(value) => value
                .into_single()
                .and_then(|value| value.into_path())
                .and_then(|path| path.into_single())
                .ok_or("Derived object fields can only be mapped to parent scalar/enum fields")?,
            SelectedValueOrField::Field(id) => id,
        };
        field_records.push(DeriveObjectFieldRecord { from_id, to_id });
    }

    let mut found_matching_key = false;
    'keys: for key in keys {
        if key.len() > field_records.len() {
            continue;
        }
        for item in key {
            if !item.subselection_record.is_empty() {
                continue 'keys;
            }
            let id = builder.selections[item.field_id].definition_id;
            if !field_records.iter().any(|record| record.to_id == id) {
                continue 'keys;
            }
        }
        found_matching_key = true;
        break;
    }
    if !found_matching_key {
        return Err("Derived field must match at least one @key".into());
    }
    Ok(DeriveObjectRecord { field_records })
}
