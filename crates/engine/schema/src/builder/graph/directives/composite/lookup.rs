use cynic_parser_deser::ConstDeserializer as _;
use id_newtypes::IdRange;
use itertools::Itertools;
use wrapping::Wrapping;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, DirectiveSiteId, EntityDefinitionId,
    FieldDefinitionId, FieldSetItemRecord, FieldSetRecord, Graph, InputValueDefinitionId, LookupResolverDefinitionId,
    LookupResolverDefinitionRecord, SchemaFieldRecord, StringId, SubgraphId, TypeRecord, ValueInjection,
    builder::{
        BoundFieldValue, BoundSelectedObjectField, BoundSelectedValueEntry, BoundValue, DirectivesIngester, Error,
        GraphBuilder, PossibleCompositeEntityKeys,
        graph::{
            directives::{
                PossibleCompositeEntityKey,
                composite::injection::{
                    create_requirements_and_injection_for_selected_value, prepend_requirements_and_injection_with_path,
                },
            },
            selections::SelectionsBuilder,
        },
        sdl,
    },
};

use super::injection::{create_requirements_and_injections, try_auto_detect_unique_injection};

#[tracing::instrument(name = "ingest_composite_loop", fields(field = %field.to_site_string(ingester)), skip_all)]
pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    field: sdl::FieldSdlDefinition<'sdl>,
    directive: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    let sdl::LookupDirective { graph: subgraph_name } = directive.deserialize().map_err(|err| {
        (
            format!(
                "At {}, invalid composite__lookup directive: {}",
                field.to_site_string(ingester),
                err
            ),
            directive.arguments_span(),
        )
    })?;
    let subgraph_id = ingester.subgraphs.try_get(subgraph_name, directive.arguments_span())?;

    let graph = &ingester.builder.graph;
    let field_definition = &graph[field.id];
    let argument_ids = field_definition.argument_ids;

    let LookupEntity {
        batch,
        entity_id,
        possible_keys,
        namespace_key,
    } = detect_lookup_entity(
        ingester.builder,
        &mut ingester.possible_composite_entity_keys,
        subgraph_id,
        field_definition.ty_record,
    )
    .map_err(|err| err.with_span_if_absent(field.span()))?;

    let explicit_injections = detect_explicit_is_directive_injections(
        ingester.builder,
        field,
        TypeRecord {
            definition_id: entity_id.into(),
            wrapping: if batch {
                Wrapping::default().non_null().list_non_null()
            } else {
                Wrapping::default().non_null()
            },
        },
        SubgraphInfo {
            id: subgraph_id,
            name: subgraph_name,
        },
    )?;
    let mut lookup_keys = Vec::new();
    for PossibleCompositeEntityKey { key, key_str, used_by } in possible_keys {
        let span = tracing::debug_span!("match_key", key = %key_str);
        let _enter = span.enter();

        let candidate = match &explicit_injections {
            ExplicitKeyInjection::None => try_auto_detect_unique_injection(ingester.builder, batch, key, argument_ids)?,
            ExplicitKeyInjection::Exact(field_set, argument_ids) => {
                if field_set == key {
                    Some(*argument_ids)
                } else {
                    None
                }
            }
            ExplicitKeyInjection::OneOf(alternatives) => alternatives
                .iter()
                .find_map(|(fs, candidate)| if fs == key { Some(*candidate) } else { None }),
        };

        let Some(candidate) = candidate else {
            tracing::debug!("No matching arguments found for key {key_str}");
            continue;
        };

        if let Some(used_by) = used_by {
            return Err((
                format!(
                    "matching a key already used by a separate @lookup field: {}",
                    used_by.to_site_string(ingester)
                ),
                field.span(),
            )
                .into());
        }
        *used_by = Some(field);
        lookup_keys.push((key.clone(), candidate));
    }

    if lookup_keys.is_empty() {
        return Err(("no matching @key directive was found", field.span()).into());
    };

    add_lookup_entity_resolvers(
        &mut ingester.builder.graph,
        &ingester.builder.selections,
        field.id,
        entity_id,
        batch,
        namespace_key,
        lookup_keys,
    );

    Ok(())
}

struct LookupEntity<'a, 'sdl> {
    batch: bool,
    entity_id: EntityDefinitionId,
    possible_keys: &'a mut Vec<PossibleCompositeEntityKey<'sdl>>,
    namespace_key: Option<StringId>,
}

fn detect_lookup_entity<'a, 'sdl>(
    builder: &GraphBuilder<'_>,
    possible_composite_entity_keys: &'a mut PossibleCompositeEntityKeys<'sdl>,
    subgraph_id: SubgraphId,
    output: TypeRecord,
) -> Result<LookupEntity<'a, 'sdl>, Error> {
    let Some(entity_id) = output.definition_id.as_entity() else {
        return Err("can only be used to return objects or interfaces.".into());
    };

    if possible_composite_entity_keys.contains_key(&(entity_id, subgraph_id)) {
        let batch = match output.wrapping.list_wrappings().len() {
            0 => false,
            1 => true,
            _ => return Err(("output wrapping cannot be multiple lists.").into()),
        };

        return Ok(LookupEntity {
            batch,
            entity_id,
            namespace_key: None,
            // Looks really stupid, but for some reason Rust complains about
            // possible_composite_entity_keys still being borrowed despite the change in the 2024
            // edition.
            possible_keys: possible_composite_entity_keys
                .get_mut(&(entity_id, subgraph_id))
                .expect("should be present since we checked it before"),
        });
    }

    let output_sdl = builder.definitions.site_id_to_sdl[&entity_id.into()].as_type().unwrap();

    let field_ids = match entity_id {
        EntityDefinitionId::Object(id) => builder.graph[id].field_ids,
        EntityDefinitionId::Interface(id) => builder.graph[id].field_ids,
    };
    let mut candidates = Vec::new();
    for field in builder.graph[field_ids].iter() {
        if let Some(entity_id) = field.ty_record.definition_id.as_entity()
            && possible_composite_entity_keys.contains_key(&(entity_id, subgraph_id))
        {
            candidates.push((field, entity_id));
        }
    }
    if let Some((namespace_field, entity_id)) = candidates.pop() {
        if let Some((other_field, _)) = candidates.pop() {
            let message = format!(
                "Type {} doesn't define any keys with @key directive that may be used for @lookup. Tried treating it as a namespace type, but it has multiple fields that may be used for @lookup: {} and {}",
                output_sdl.name(),
                builder.ctx[namespace_field.name_id],
                builder.ctx[other_field.name_id]
            );
            return Err((message, output_sdl.span()).into());
        }

        let batch = match namespace_field.ty_record.wrapping.list_wrappings().len() {
            0 => false,
            1 => true,
            _ => return Err(("output wrapping cannot be multiple lists.").into()),
        };

        return Ok(LookupEntity {
            batch,
            entity_id,
            namespace_key: Some(namespace_field.name_id),
            // Looks really stupid, but for some reason Rust complains about
            // possible_composite_entity_keys still being borrowed despite the change in the 2024
            // edition.
            possible_keys: possible_composite_entity_keys
                .get_mut(&(entity_id, subgraph_id))
                .expect("should be present since we checked it before"),
        });
    }

    let message = format!(
        "Type {} doesn't define any keys with @key directive that may be used for @lookup. Tried treating it as a namespace type, but it didn't have any fields that may be used for @lookup.",
        output_sdl.name()
    );
    Err((message, output_sdl.span()).into())
}

fn add_lookup_entity_resolvers(
    graph: &mut Graph,
    selections: &SelectionsBuilder,
    lookup_field_id: FieldDefinitionId,
    output: EntityDefinitionId,
    guest_batch: bool,
    namespace_key_id: Option<StringId>,
    lookup_keys: Vec<(FieldSetRecord, IdRange<ArgumentInjectionId>)>,
) {
    let field_ids = match output {
        EntityDefinitionId::Object(id) => graph[id].field_ids,
        EntityDefinitionId::Interface(id) => graph[id].field_ids,
    };
    let mut resolvers = Vec::new();
    for (key, injection_ids) in lookup_keys {
        debug_assert!(resolvers.is_empty());
        for &resolver_id in &graph.field_definitions[usize::from(lookup_field_id)].resolver_ids {
            let lookup_resolver_id = LookupResolverDefinitionId::from(graph.lookup_resolver_definitions.len());
            graph.lookup_resolver_definitions.push(LookupResolverDefinitionRecord {
                key_record: key.clone(),
                field_definition_id: lookup_field_id,
                resolver_id,
                guest_batch,
                injection_ids,
                namespace_key_id,
            });
            resolvers.push(graph.resolver_definitions.len().into());
            graph.resolver_definitions.push(lookup_resolver_id.into());
        }
        for field_id in field_ids {
            // If part of the key we can't be provided by this resolver.
            if key
                .iter()
                .all(|item| selections[item.field_id].definition_id != field_id)
            {
                graph[field_id].resolver_ids.extend_from_slice(&resolvers);
            }
        }
        resolvers.clear();
    }
}

fn detect_explicit_is_directive_injections(
    builder: &mut GraphBuilder<'_>,
    field: sdl::FieldSdlDefinition<'_>,
    source: TypeRecord,
    subgraph: SubgraphInfo<'_>,
) -> Result<ExplicitKeyInjection, Error> {
    let field_definition = &builder.graph[field.id];
    let argument_ids = field_definition.argument_ids;
    let injections = {
        argument_ids
            .into_iter()
            .map(|argument_id| {
                let sdl_arg = builder.definitions.site_id_to_sdl[&DirectiveSiteId::from(argument_id)];
                find_field_selection_map(builder, subgraph, source, argument_id, sdl_arg.directives()).map(|opt| {
                    opt.map(|(field_selection_map, is_directive)| (argument_id, field_selection_map, is_directive))
                })
            })
            .filter_map_ok(|x| x)
            .collect::<Result<Vec<_>, _>>()
    }?;

    if injections.is_empty() {
        return Ok(ExplicitKeyInjection::None);
    }

    // We need to split this case to generate a different candidate for each alternative, as each
    // of them can match a different key.
    if injections.len() == 1
        && injections
            .first()
            .map(|(arg_id, value, _)| {
                let is_one_of = builder.graph[*arg_id]
                    .ty_record
                    .definition_id
                    .as_input_object()
                    .map(|id| builder.graph[id].is_one_of)
                    .unwrap_or_default();
                matches!(value, BoundValue::Value(value) if value.alternatives.len() > 1 && is_one_of)
            })
            .unwrap_or_default()
    {
        let Some((argument_id, BoundValue::Value(value), directive)) = injections.into_iter().next() else {
            unreachable!()
        };
        return value
            .alternatives
            .into_iter()
            .map(|entry| {
                let BoundSelectedValueEntry::Object { path, object } = entry else {
                    unreachable!()
                };
                if object.fields.len() != 1 {
                    return Err((
                        "With a @oneOf input object argument, only one field can be provided per alternative.",
                        directive.arguments_span(),
                    )
                        .into());
                }
                let BoundSelectedObjectField {
                    id: oneof_field_id,
                    value,
                } = object.fields.into_iter().next().unwrap();
                let result = match value {
                    BoundFieldValue::Value(value) => {
                        create_requirements_and_injection_for_selected_value(builder, value)?
                    }
                    BoundFieldValue::Field(definition_id) => {
                        let field_id = builder.selections.insert_field(SchemaFieldRecord {
                            definition_id,
                            sorted_argument_ids: Default::default(),
                        });
                        let requires = FieldSetRecord::from_iter([FieldSetItemRecord {
                            field_id,
                            subselection_record: Default::default(),
                        }]);
                        let value = ValueInjection::Select {
                            field_id,
                            next: builder.selections.push_injection(ValueInjection::Identity),
                        };
                        (requires, value)
                    }
                    BoundFieldValue::DefaultValue(_) => unreachable!(),
                };
                let (requires, value) =
                    prepend_requirements_and_injection_with_path(builder, path.unwrap_or_default(), result);

                let nested = builder.selections.push_argument_injections([ArgumentInjectionRecord {
                    definition_id: oneof_field_id,
                    value: ArgumentValueInjection::Value(value),
                }]);
                let mut arguments = vec![ArgumentInjectionRecord {
                    definition_id: argument_id,
                    value: ArgumentValueInjection::InputObject(nested),
                }];
                for id in argument_ids {
                    if id == argument_id {
                        continue;
                    }
                    let arg = &builder.graph[id];
                    if let Some(default_value_id) = arg.default_value_id {
                        arguments.push(ArgumentInjectionRecord {
                            definition_id: id,
                            value: ArgumentValueInjection::Value(ValueInjection::DefaultValue(default_value_id)),
                        });
                    } else if arg.ty_record.wrapping.is_non_null() {
                        return Err((
                            format!(
                                "Argument '{}' is required but is not injected by any @is directive.",
                                builder.ctx[arg.name_id]
                            ),
                            field.span(),
                        )
                            .into());
                    }
                }
                let range = builder.selections.push_argument_injections(arguments);
                Ok((requires, range))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(ExplicitKeyInjection::OneOf);
    }

    let (field_set, mut arguments) =
        create_requirements_and_injections(builder, injections.into_iter().map(|(a, v, _)| (a, v)))?;
    let present_ids = arguments.iter().map(|record| record.definition_id).collect::<Vec<_>>();
    for id in argument_ids {
        if present_ids.contains(&id) {
            continue;
        }
        let arg = &builder.graph[id];
        if let Some(default_value_id) = arg.default_value_id {
            arguments.push(ArgumentInjectionRecord {
                definition_id: id,
                value: ArgumentValueInjection::Value(ValueInjection::DefaultValue(default_value_id)),
            });
        } else if arg.ty_record.wrapping.is_non_null() {
            return Err((
                format!(
                    "Argument '{}' is required but is not injected by any @is directive.",
                    builder.ctx[arg.name_id]
                ),
                field.span(),
            )
                .into());
        }
    }

    Ok(ExplicitKeyInjection::Exact(
        field_set,
        builder.selections.push_argument_injections(arguments),
    ))
}

enum ExplicitKeyInjection {
    Exact(FieldSetRecord, IdRange<ArgumentInjectionId>),
    OneOf(Vec<(FieldSetRecord, IdRange<ArgumentInjectionId>)>),
    None,
}

fn find_field_selection_map<'d>(
    builder: &mut GraphBuilder<'_>,
    subgraph: SubgraphInfo<'_>,
    source: TypeRecord,
    argument_id: InputValueDefinitionId,
    directives: impl Iterator<Item = sdl::Directive<'d>>,
) -> Result<Option<(BoundValue, sdl::Directive<'d>)>, Error> {
    let mut is_directives = directives
        .filter(|dir| dir.name() == "composite__is")
        .map(|dir| {
            dir.deserialize::<sdl::IsDirective>()
                .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                .map(|args| (dir, args))
        })
        .filter_ok(|(_, args)| args.graph == subgraph.name);

    let Some((bound_value, is_directive)) = is_directives
        .next()
        .transpose()?
        .map(
            |(
                is_directive,
                sdl::IsDirective {
                    field: field_selection_map,
                    ..
                },
            )| {
                tracing::trace!(
                    "Found @is(field: \"{field_selection_map}\") for {}",
                    builder.ctx[builder.graph[argument_id].name_id]
                );
                builder
                    .parse_field_selection_map_for_argument(source, subgraph.id, argument_id, field_selection_map)
                    .map(|field_selection_map| (field_selection_map, is_directive))
                    .map_err(|err| {
                        (
                            format!("for associated @is directive: {err}"),
                            is_directive.arguments_span(),
                        )
                    })
            },
        )
        .transpose()?
    else {
        return Ok(None);
    };

    if is_directives.next().is_some() {
        return Err((
            "Multiple @composite__is directives on the same argument are not supported.",
            is_directive.name_span(),
        )
            .into());
    }

    Ok(Some((bound_value, is_directive)))
}

#[derive(Clone, Copy)]
struct SubgraphInfo<'sdl> {
    id: SubgraphId,
    name: sdl::GraphName<'sdl>,
}
