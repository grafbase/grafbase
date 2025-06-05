use cynic_parser_deser::ConstDeserializer as _;
use id_newtypes::IdRange;
use itertools::Itertools;
use wrapping::Wrapping;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, DirectiveSiteId, EntityDefinitionId,
    FieldDefinitionId, FieldSetItemRecord, FieldSetRecord, Graph, InputValueDefinitionId, LookupResolverDefinitionId,
    LookupResolverDefinitionRecord, SchemaFieldRecord, TypeRecord, ValueInjection,
    builder::{
        BoundSelectedObjectField, BoundSelectedValue, BoundSelectedValueEntry, DirectivesIngester, Error, GraphBuilder,
        SelectedValueOrField,
        graph::{
            directives::{PossibleCompositeEntityKey, composite::injection::create_requirements_and_injection},
            selections::SelectionsBuilder,
        },
        sdl,
    },
};

use super::injection::{prepend_requirements_and_injection_with_path, try_auto_detect_unique_injection};

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
    let Some(entity_id) = field_definition.ty_record.definition_id.as_entity() else {
        return Err(("can only be used to return objects or interfaces.", field.span()).into());
    };

    let batch = match field_definition.ty_record.wrapping.list_wrappings().len() {
        0 => false,
        1 => true,
        _ => return Err(("output wrapping cannot be multiple lists.", field.span()).into()),
    };

    let explicit_injections = detect_explicit_is_directive_injections(ingester.builder, field, batch, subgraph_name)?;
    let Some(possible_keys) = ingester
        .possible_composite_entity_keys
        .get_mut(&(entity_id, subgraph_id))
    else {
        let ty = ingester.definitions.site_id_to_sdl[&entity_id.into()]
            .as_type()
            .unwrap();
        return Err((
            format!(
                "Type {} doesn't define any keys with @key directive that may be used for @lookup",
                ty.name()
            ),
            ty.span(),
        )
            .into());
    };

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
        lookup_keys,
    );

    Ok(())
}

fn add_lookup_entity_resolvers(
    graph: &mut Graph,
    selections: &SelectionsBuilder,
    lookup_field_id: FieldDefinitionId,
    output: EntityDefinitionId,
    batch: bool,
    lookup_keys: Vec<(FieldSetRecord, IdRange<ArgumentInjectionId>)>,
) {
    let mut resolvers = Vec::new();
    for (key, injection_ids) in lookup_keys {
        debug_assert!(resolvers.is_empty());
        for &resolver_id in &graph.field_definitions[usize::from(lookup_field_id)].resolver_ids {
            let lookup_resolver_id = LookupResolverDefinitionId::from(graph.lookup_resolver_definitions.len());
            graph.lookup_resolver_definitions.push(LookupResolverDefinitionRecord {
                key_record: key.clone(),
                field_definition_id: lookup_field_id,
                resolver_id,
                batch,
                injection_ids,
            });
            resolvers.push(graph.resolver_definitions.len().into());
            graph.resolver_definitions.push(lookup_resolver_id.into());
        }
        let field_ids = match output {
            EntityDefinitionId::Object(id) => graph[id].field_ids,
            EntityDefinitionId::Interface(id) => graph[id].field_ids,
        };
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
    batch: bool,
    subgraph_name: sdl::GraphName<'_>,
) -> Result<ExplicitKeyInjection, Error> {
    let field_definition = &builder.graph[field.id];
    let argument_ids = field_definition.argument_ids;
    let is_directive_injections: Vec<(
        InputValueDefinitionId,
        BoundSelectedValue<InputValueDefinitionId>,
        sdl::Directive<'_>,
    )> = {
        let source = TypeRecord {
            definition_id: builder.graph[field.id].ty_record.definition_id,
            wrapping: if batch {
                Wrapping::default().non_null().list_non_null()
            } else {
                Wrapping::default().non_null()
            },
        };
        argument_ids
            .into_iter()
            .map(|argument_id| {
                let sdl_arg = builder.definitions.site_id_to_sdl[&DirectiveSiteId::from(argument_id)];
                find_field_selection_map(
                    builder,
                    subgraph_name,
                    source,
                    field.id,
                    argument_id,
                    sdl_arg.directives(),
                )
                .map(|opt| {
                    opt.map(|(field_selection_map, is_directive)| (argument_id, field_selection_map, is_directive))
                })
            })
            .filter_map_ok(|x| x)
            .collect::<Result<_, _>>()
    }?;

    let has_one_of = is_directive_injections.iter().any(|(arg_id, _, _)| {
        builder.graph[*arg_id]
            .ty_record
            .definition_id
            .as_input_object()
            .map(|id| builder.graph[id].is_one_of)
            .unwrap_or_default()
    });

    match is_directive_injections.len() {
        0 => Ok(ExplicitKeyInjection::None),
        1 if has_one_of && is_directive_injections.is_empty() => {
            let Some((argument_id, field_selection_map, directive)) = is_directive_injections.into_iter().next() else {
                unreachable!()
            };
            field_selection_map
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
                    let BoundSelectedObjectField { id, value } = object.fields.into_iter().next().unwrap();
                    let result = match value {
                        SelectedValueOrField::Value(value) => create_requirements_and_injection(builder, value)?,
                        SelectedValueOrField::Field(definition_id) => {
                            let field_id = builder.selections.insert_field(SchemaFieldRecord {
                                definition_id,
                                sorted_argument_ids: Default::default(),
                            });
                            (
                                FieldSetRecord::from_iter([FieldSetItemRecord {
                                    field_id,
                                    subselection_record: Default::default(),
                                }]),
                                ValueInjection::Select {
                                    field_id,
                                    next: builder.selections.push_injection(ValueInjection::Identity),
                                },
                            )
                        }
                        SelectedValueOrField::DefaultValue(_) => unreachable!(),
                    };
                    let (requires, injection) =
                        prepend_requirements_and_injection_with_path(builder, path.unwrap_or_default(), result);

                    let value = builder
                        .selections
                        .push_argument_value_injection(ArgumentValueInjection::Value(injection));
                    let mut arguments = vec![ArgumentInjectionRecord {
                        definition_id: argument_id,
                        value: ArgumentValueInjection::Nested {
                            key: builder.graph[id].name_id,
                            value,
                        },
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
                        } else if arg.ty_record.wrapping.is_required() {
                            return Err((
                                format!(
                                    "Argument '{}' is required but is not injected with any @is directive.",
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
                .map(ExplicitKeyInjection::OneOf)
        }
        _ => {
            if is_directive_injections.len() > 1 && has_one_of {
                return Err((
                    "With a @oneOf argument, only one @is directive is supported for @lookup.",
                    field.span(),
                )
                    .into());
            }

            let mut field_set = FieldSetRecord::default();
            let mut arguments = Vec::new();
            for (argument_id, field_selection_map, _) in is_directive_injections {
                let (requires, injection) = create_requirements_and_injection(builder, field_selection_map)?;
                field_set = field_set.union(&requires);
                arguments.push(ArgumentInjectionRecord {
                    definition_id: argument_id,
                    value: ArgumentValueInjection::Value(injection),
                });
            }
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
                } else if arg.ty_record.wrapping.is_required() {
                    return Err((
                        format!(
                            "Argument '{}' is required but is not injected any @is directive.",
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
    }
}

enum ExplicitKeyInjection {
    Exact(FieldSetRecord, IdRange<ArgumentInjectionId>),
    OneOf(Vec<(FieldSetRecord, IdRange<ArgumentInjectionId>)>),
    None,
}

fn find_field_selection_map<'d>(
    builder: &mut GraphBuilder<'_>,
    subgraph_name: sdl::GraphName<'_>,
    source: TypeRecord,
    field_definition_id: FieldDefinitionId,
    argument_id: InputValueDefinitionId,
    directives: impl Iterator<Item = sdl::Directive<'d>>,
) -> Result<Option<(BoundSelectedValue<InputValueDefinitionId>, sdl::Directive<'d>)>, Error> {
    let mut is_directives = directives
        .filter(|dir| dir.name() == "composite__is")
        .map(|dir| {
            dir.deserialize::<sdl::IsDirective>()
                .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                .map(|args| (dir, args))
        })
        .filter_ok(|(_, args)| args.graph == subgraph_name);

    let Some((field_selection_map, is_directive)) = is_directives
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
                    .parse_field_selection_map_for_argument(
                        source,
                        field_definition_id,
                        argument_id,
                        field_selection_map,
                    )
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

    Ok(Some((field_selection_map, is_directive)))
}
