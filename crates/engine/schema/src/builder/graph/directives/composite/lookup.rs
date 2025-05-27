use cynic_parser_deser::ConstDeserializer as _;
use id_newtypes::IdRange;
use itertools::Itertools as _;
use wrapping::Wrapping;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, DirectiveSiteId, EntityDefinitionId,
    FieldDefinitionId, FieldDefinitionRecord, FieldSetItemRecord, FieldSetRecord, Graph, InputValueDefinitionId,
    InputValueDefinitionRecord, KeyValueInjectionRecord, LookupResolverDefinitionId, LookupResolverDefinitionRecord,
    TypeDefinitionId, TypeRecord, ValueInjection,
    builder::{
        BoundSelectedObjectField, BoundSelectedValue, BoundSelectedValueEntry, DirectivesIngester, Error, GraphBuilder,
        SelectedValueOrField,
        graph::{directives::PossibleCompositeEntityKey, selections::SelectionsBuilder},
        sdl::{self, GraphName, IsDirective},
    },
};

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
    let Some(entity_id) = field_definition.ty_record.definition_id.as_entity() else {
        return Err(("can only be used to return objects or interfaces.", field.span()).into());
    };

    let batch = match field_definition.ty_record.wrapping.list_wrappings().len() {
        0 => false,
        1 => true,
        _ => return Err(("output wrapping cannot be multiple lists.", field.span()).into()),
    };

    let Some(possible_keys) = ingester
        .possible_composite_entity_keys
        .get_mut(&(entity_id, subgraph_id))
    else {
        let ty = ingester.sdl_definitions[&entity_id.into()].as_type().unwrap();
        return Err((
            format!(
                "Type {} doesn't define any keys with @key directive that may be used for @lookup",
                ty.name()
            ),
            ty.span(),
        )
            .into());
    };

    let argument_ids = field_definition.argument_ids;
    let mut lookup_keys = Vec::new();
    let mut builder = ValueInjectionBuilder {
        subgraph_name,
        root_field_id: field.id,
        source: TypeRecord {
            definition_id: ingester.builder.graph[field.id].ty_record.definition_id,
            wrapping: if batch {
                Wrapping::default().non_null().list_non_null()
            } else {
                Wrapping::default().non_null()
            },
        },
        last_selections_injection_state: ingester.builder.selections.current_injection_state(),
        builder: ingester.builder,
        sdl_definitions: ingester.sdl_definitions,
    };
    for PossibleCompositeEntityKey { key, key_str, used_by } in possible_keys {
        let span = tracing::debug_span!("match_key", key = %key_str);
        let _enter = span.enter();
        let mut candidates = builder.try_build_arguments_value_injections(batch, key, argument_ids)?;
        let Some(candidate) = candidates.pop() else {
            tracing::debug!("No candidiate found");
            continue;
        };
        if !candidates.is_empty() {
            tracing::debug!("Multiple candidiates found, skipping key");
            builder.reset();
            continue;
        }
        builder.save();

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

struct ValueInjectionBuilder<'a, 'b> {
    builder: &'a mut GraphBuilder<'b>,
    sdl_definitions: &'a sdl::SdlDefinitions<'a>,
    subgraph_name: GraphName<'a>,
    root_field_id: FieldDefinitionId,
    source: TypeRecord,
    last_selections_injection_state: [usize; 4],
}

impl<'b> std::ops::Deref for ValueInjectionBuilder<'_, 'b> {
    type Target = GraphBuilder<'b>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl ValueInjectionBuilder<'_, '_> {
    fn save(&mut self) {
        self.last_selections_injection_state = self.selections.current_injection_state();
    }

    fn reset(&mut self) {
        self.builder
            .selections
            .reset_injection_state(self.last_selections_injection_state);
    }

    fn try_build_arguments_value_injections(
        &mut self,
        batch: bool,
        key_fields: &FieldSetRecord,
        argument_ids: IdRange<InputValueDefinitionId>,
    ) -> Result<Vec<IdRange<ArgumentInjectionId>>, Error> {
        self.last_selections_injection_state = self.selections.current_injection_state();
        let mut candidates = Vec::new();

        // Try direct match against arguments
        // We're not supporting cases like `lookup(key1: [ID!], key2: [ID!])` for composite keys.
        // We could, but means creating two lists which can't optimize for.
        if key_fields.len() == 1 || !batch {
            tracing::trace!("Trying to match against arguments directly.");
            candidates.extend(self.try_build_arguments_injections(batch, key_fields, argument_ids)?)
        }

        let is_required_arg_bitset: u64 = {
            let mut bitset = 0;
            for (i, arg) in self.graph[argument_ids].iter().enumerate() {
                if arg.ty_record.is_required() {
                    // More than one required input means we'll never find a single nested input object
                    // we can use.
                    if bitset != 0 {
                        return Ok(candidates);
                    }
                    bitset |= 1 << i;
                }
            }
            bitset
        };

        tracing::trace!("Trying to match against a nested input object.");
        // Try with a nested input object
        for (i, argument_id) in argument_ids.into_iter().enumerate() {
            let arg = &self.graph[argument_id];
            let span = tracing::debug_span!("match_argument", key = %self.ctx[arg.name_id]);
            let _enter = span.enter();
            let Some(input_object_id) = arg.ty_record.definition_id.as_input_object() else {
                continue;
            };
            if (is_required_arg_bitset & !(1 << i)) != 0 {
                tracing::trace!("There exists another required argument.");
                // There exist a one required input, so can't use this argument for the key.
                continue;
            }

            let sdl_arg = self.sdl_definitions[&DirectiveSiteId::from(argument_id)];
            let mut is_directives = sdl_arg
                .directives()
                .filter(|dir| dir.name() == "composite__is")
                .map(|dir| {
                    dir.deserialize::<IsDirective>()
                        .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                        .map(|args| (dir, args))
                })
                .filter_ok(|(_, args)| args.graph == self.subgraph_name);

            let field_selection_map = is_directives
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
                        self.builder
                            .parse_field_selection_map_for_argument(
                                self.source,
                                self.root_field_id,
                                argument_id,
                                field_selection_map,
                            )
                            .map_err(|err| {
                                (
                                    format!("for associated @is directive: {err}"),
                                    is_directive.arguments_span(),
                                )
                            })
                    },
                )
                .transpose()?;

            let arg = &self.graph[argument_id];
            let input_object = &self.graph[input_object_id];

            if input_object.is_one_of {
                if arg.ty_record.wrapping.is_list() {
                    continue;
                }
                if key_fields.len() == 1 {
                    tracing::trace!("Trying to match with oneof input object for a key having a single field.");
                    if let Some(value) = self.try_build_oneof_input_object_single_key_injection(
                        batch,
                        &key_fields[0],
                        input_object.input_field_ids,
                        field_selection_map,
                    )? {
                        debug_assert_eq!(value.len(), 1);
                        let (input_field_id, value) = value.into_iter().next().unwrap();
                        let value = self
                            .builder
                            .selections
                            .push_argument_value_injection(ArgumentValueInjection::Value(value));
                        let range = self
                            .builder
                            .selections
                            .push_argument_injections([ArgumentInjectionRecord {
                                definition_id: argument_id,
                                value: ArgumentValueInjection::Nested {
                                    key: self.graph[input_field_id].name_id,
                                    value,
                                },
                            }]);
                        candidates.push(range)
                    }
                } else {
                    tracing::trace!("Trying to match with oneof input object for a key having multiple fields.");
                    for oneof_field_id in input_object.input_field_ids {
                        let oneof_field = &self.graph[oneof_field_id];
                        let Some(nested_input_object_id) = oneof_field.ty_record.definition_id.as_input_object() else {
                            continue;
                        };
                        if !matches!(
                            (batch, oneof_field.ty_record.wrapping.list_wrappings().len()),
                            (true, 1) | (false, 0)
                        ) {
                            continue;
                        }
                        let name_id = oneof_field.name_id;
                        if let Some(value) = self.try_build_input_object_injections(
                            false,
                            key_fields,
                            self.graph[nested_input_object_id].input_field_ids,
                        )? {
                            let value = self
                                .builder
                                .selections
                                .push_argument_value_injection(ArgumentValueInjection::Value(value));
                            let range = self
                                .builder
                                .selections
                                .push_argument_injections([ArgumentInjectionRecord {
                                    definition_id: argument_id,
                                    value: ArgumentValueInjection::Nested { key: name_id, value },
                                }]);
                            candidates.push(range)
                        }
                    }
                }
            } else if key_fields.len() > 1 {
                tracing::trace!("Trying to match with nested object for a key having multiple fields.");
                if !matches!(
                    (batch, arg.ty_record.wrapping.list_wrappings().len()),
                    (true, 1) | (false, 0)
                ) {
                    continue;
                }

                if let Some(value) =
                    self.try_build_input_object_injections(false, key_fields, input_object.input_field_ids)?
                {
                    let range = self
                        .builder
                        .selections
                        .push_argument_injections([ArgumentInjectionRecord {
                            definition_id: argument_id,
                            value: ArgumentValueInjection::Value(value),
                        }]);
                    candidates.push(range)
                }
            }
        }

        Ok(candidates)
    }

    fn try_build_arguments_injections(
        &mut self,
        batch: bool,
        key_fields: &[FieldSetItemRecord],
        argument_ids: IdRange<InputValueDefinitionId>,
    ) -> Result<Option<IdRange<ArgumentInjectionId>>, Error> {
        assert!(key_fields.len() < 64, "Cannot handle keys with 64 fields or more.");
        let mut missing: u64 = (1 << key_fields.len()) - 1;
        let mut explicit_mapping: u64 = 0;

        let mut argument_injections = Vec::new();
        for argument_id in argument_ids {
            let sdl_arg = self.sdl_definitions[&DirectiveSiteId::from(argument_id)];
            let mut is_directives = sdl_arg
                .directives()
                .filter(|dir| dir.name() == "composite__is")
                .map(|dir| {
                    dir.deserialize::<IsDirective>()
                        .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                        .map(|args| (dir, args))
                })
                .filter_ok(|(_, args)| args.graph == self.subgraph_name);

            let Some((
                is_directive,
                sdl::IsDirective {
                    field: field_selection_map,
                    ..
                },
            )) = is_directives.next().transpose()?
            else {
                continue;
            };

            tracing::trace!(
                "Found @is(field: \"{field_selection_map}\") for {}",
                self.ctx[self.graph[argument_id].name_id]
            );

            if is_directives.next().is_some() {
                return Err((
                    "Multiple @composite__is directives on the same argument are not supported.",
                    sdl_arg.span(),
                )
                    .into());
            }

            let value = self
                .builder
                .parse_field_selection_map_for_argument(
                    self.source,
                    self.root_field_id,
                    argument_id,
                    field_selection_map,
                )
                .map_err(|err| {
                    (
                        format!("for associated @is directive: {err}"),
                        is_directive.arguments_span(),
                    )
                })?;

            // @oneof arguments are treated separately. If we encounter one, we don't map arguments
            // at all.
            let Some(value) = value.into_single() else {
                tracing::trace!("Skipping FieldSelectionMap with alternatives");
                return Ok(None);
            };

            let value = if batch {
                if let BoundSelectedValueEntry::List { path: None, list } = value {
                    let Some(value) = list.0.into_single() else {
                        tracing::trace!("Skipping FieldSelectionMap with alternatives within a list");
                        return Ok(None);
                    };
                    value
                } else {
                    tracing::trace!("Skipping non-list FieldSelectionMap for a batch lookup");
                    return Ok(None);
                }
            } else {
                value
            };

            // We only take care of the case where individual fields are associated to arguments.
            // Composite keys injected in a single argument are treated separately.
            let (path, object) = match value {
                BoundSelectedValueEntry::Path(path) if path.len() == 1 => (path, None),
                BoundSelectedValueEntry::Object {
                    path: Some(path),
                    object,
                } if path.len() == 1 => (path, Some(object)),
                _ => {
                    return Err((
                        "Unsupported FieldSelectionMap for @composite__is directive used in a @composite__lookup context.",
                        is_directive.arguments_span(),
                    )
                    .into());
                }
            };
            let field = path.into_single().unwrap();
            let Some(pos) = key_fields
                .iter()
                .position(|k| self.selections[k.field_id].definition_id == field)
            else {
                tracing::trace!("Could not find the key associated with current FieldSelectionMap");
                return Ok(None);
            };
            explicit_mapping |= 1 << pos;
            missing &= !(1 << pos);
            argument_injections.push(ArgumentInjectionRecord {
                definition_id: argument_id,
                value: ArgumentValueInjection::Value(ValueInjection::Select {
                    field_id: key_fields[pos].field_id,
                    next: if let Some(object) = object {
                        let Some(injection) =
                            self.try_build_key_injection_from_field_selection_map(key_fields, object.fields)?
                        else {
                            return Ok(None);
                        };
                        Some(self.builder.selections.push_injection(injection))
                    } else {
                        None
                    },
                }),
            });
        }

        if missing != 0 {
            for argument_id in argument_ids {
                if let Some((pos, value)) = self
                    .try_auto_detect_unique_input_value_key_mapping(argument_ids, key_fields, batch, argument_id)?
                    // We skip element without a position (default values), they're handled at the
                    // end to avoid duplicate injections.
                    .and_then(|(pos, value)| {
                        // If a key was already explicitly mapped, we skip it.
                        pos.filter(|pos| explicit_mapping & (1 << pos) == 0)
                            .map(|pos| (pos, value))
                    })
                {
                    missing &= !(1 << pos);
                    argument_injections.push(ArgumentInjectionRecord {
                        definition_id: argument_id,
                        value: ArgumentValueInjection::Value(value),
                    });
                }
            }
            if missing != 0 {
                tracing::trace!("Could not match some key fields.");
                return Ok(None);
            }
        }

        for argument_id in argument_ids {
            if argument_injections
                .iter()
                .any(|injection| injection.definition_id == argument_id)
            {
                continue;
            }
            // If the argument is not injected, we inject its default value.
            if let Some(default_value_id) = self.graph[argument_id].default_value_id {
                argument_injections.push(ArgumentInjectionRecord {
                    definition_id: argument_id,
                    value: ArgumentValueInjection::Value(ValueInjection::Const(default_value_id)),
                });
            } else if self.graph[argument_id].ty_record.wrapping.is_required() {
                tracing::trace!("A required input doesn't match any key.");
                return Ok(None);
            }
        }

        Ok(Some(
            self.builder.selections.push_argument_injections(argument_injections),
        ))
    }

    fn try_build_key_injection_from_field_selection_map(
        &mut self,
        key_fields: &[FieldSetItemRecord],
        fields: Vec<BoundSelectedObjectField<InputValueDefinitionId>>,
    ) -> Result<Option<ValueInjection>, Error> {
        assert!(key_fields.len() < 64, "Cannot handle keys with 64 fields or more.");
        let mut missing: u64 = (1 << key_fields.len()) - 1;

        let mut key_value_injections = Vec::new();

        for field in fields {
            let (field_definition_id, object) = match field.value {
                SelectedValueOrField::Value(value) => {
                    let Some(value) = value.into_single() else {
                        return Ok(None);
                    };

                    let (path, object) = match value {
                        BoundSelectedValueEntry::Path(path) if path.len() == 1 => (path, None),
                        BoundSelectedValueEntry::Object {
                            path: Some(path),
                            object,
                        } if path.len() == 1 => (path, Some(object)),
                        _ => {
                            return Err("Unsupported FieldSelectionMap for @composite__is directive used in a @composite__lookup context.".into());
                        }
                    };
                    let field_definition_id = path.into_single().unwrap();
                    (field_definition_id, object)
                }
                SelectedValueOrField::Field(field_definition_id) => (field_definition_id, None),
            };
            let Some(pos) = key_fields
                .iter()
                .position(|k| self.selections[k.field_id].definition_id == field_definition_id)
            else {
                return Ok(None);
            };

            missing &= !(1 << pos);
            key_value_injections.push(KeyValueInjectionRecord {
                key_id: self.graph[field.field_id].name_id,
                value: ValueInjection::Select {
                    field_id: key_fields[pos].field_id,
                    next: if let Some(object) = object {
                        let Some(injection) =
                            self.try_build_key_injection_from_field_selection_map(key_fields, object.fields)?
                        else {
                            return Ok(None);
                        };
                        Some(self.builder.selections.push_injection(injection))
                    } else {
                        None
                    },
                },
            });
        }

        // Check if all key fields have been matched
        if missing != 0 {
            tracing::trace!("Could not match some key fields from field selection map.");
            return Ok(None);
        }

        let range = self.builder.selections.push_key_value_injections(key_value_injections);
        Ok(Some(ValueInjection::Object(range)))
    }

    fn try_build_input_object_injections(
        &mut self,
        batch: bool,
        key_fields: &[FieldSetItemRecord],
        input_field_ids: IdRange<InputValueDefinitionId>,
    ) -> Result<Option<ValueInjection>, Error> {
        assert!(key_fields.len() < 64, "Cannot handle keys with 64 fields or more.");
        let mut missing: u64 = (1 << key_fields.len()) - 1;

        let mut key_value_injections = Vec::new();
        for input_id in input_field_ids {
            if let Some((pos, value)) =
                self.try_auto_detect_unique_input_value_key_mapping(input_field_ids, key_fields, batch, input_id)?
            {
                if let Some(pos) = pos {
                    missing &= !(1 << pos);
                }
                key_value_injections.push(KeyValueInjectionRecord {
                    key_id: self.graph[input_id].name_id,
                    value,
                });
            } else if self.graph[input_id].ty_record.wrapping.is_required() {
                tracing::trace!("A required input doesn't match any key.");
                return Ok(None);
            }
        }

        if missing != 0 {
            tracing::trace!("Could not match some key fields.");
            return Ok(None);
        }

        let range = self.builder.selections.push_key_value_injections(key_value_injections);
        Ok(Some(ValueInjection::Object(range)))
    }

    fn try_build_oneof_input_object_single_key_injection(
        &mut self,
        batch: bool,
        key: &FieldSetItemRecord,
        input_ids: IdRange<InputValueDefinitionId>,
        _field_selection_map: Option<BoundSelectedValue<InputValueDefinitionId>>,
    ) -> Result<Option<Vec<(InputValueDefinitionId, ValueInjection)>>, Error> {
        let mut input_values = Vec::new();
        for input_id in input_ids {
            if let Some((_, value)) = self.try_auto_detect_unique_input_value_key_mapping(
                input_ids,
                std::array::from_ref(key),
                batch,
                input_id,
            )? {
                input_values.push((input_id, value));
            } else if self.graph[input_id].ty_record.wrapping.is_required() {
                tracing::trace!("A required input doesn't match any key.");
                return Ok(None);
            }
        }

        if input_values.is_empty() {
            tracing::trace!("Could not match some key fields.");
            return Ok(None);
        }

        Ok(Some(input_values))
    }

    fn try_auto_detect_unique_input_value_key_mapping(
        &mut self,
        input_ids: IdRange<InputValueDefinitionId>,
        key_fields: &[FieldSetItemRecord],
        batch: bool,
        input_id: InputValueDefinitionId,
    ) -> Result<Option<(Option<usize>, ValueInjection)>, Error> {
        let input = &self.graph[input_id];

        // position() is enough as at most there will be one unique match.
        if let Some(pos) = key_fields.iter().position(|key_field| {
            let field = &self.graph[self.selections[key_field.field_id].definition_id];
            if !can_inject_field_into_input(field, input, batch) {
                tracing::trace!(
                    "Field {} cannot be injected into input {}",
                    self.ctx[field.name_id],
                    self.ctx[input.name_id]
                );
                return false;
            }
            // Either name matches or the types are unique in both input & key and thus no other key/input pair could
            // match.
            field.name_id == input.name_id || {
                let item_ptr = key_field as *const FieldSetItemRecord;
                let input_ty = input.ty_record.non_null();
                input_ids
                    .into_iter()
                    .all(|id| id == input_id || self.graph[id].ty_record.non_null() != input_ty)
                    && key_fields.iter().all(|other_item| {
                        // Comparing pointers directly as they come from the same array.
                        let other_item_ptr = other_item as *const FieldSetItemRecord;
                        let other_field = &self.graph[self.selections[other_item.field_id].definition_id];
                        item_ptr == other_item_ptr || other_field.ty_record != field.ty_record
                    })
            }
        }) {
            Ok(Some((
                Some(pos),
                ValueInjection::Select {
                    field_id: key_fields[pos].field_id,
                    next: if key_fields[pos].subselection_record.is_empty() {
                        None
                    } else {
                        let Some(input_object_id) = input.ty_record.definition_id.as_input_object() else {
                            return Ok(None);
                        };
                        let Some(injection) = self.try_build_input_object_injections(
                            false,
                            &key_fields[pos].subselection_record,
                            self.graph[input_object_id].input_field_ids,
                        )?
                        else {
                            return Ok(None);
                        };
                        Some(self.builder.selections.push_injection(injection))
                    },
                },
            )))
        } else if let Some(default_value_id) = input.default_value_id {
            Ok(Some((None, ValueInjection::Const(default_value_id))))
        } else {
            Ok(None)
        }
    }
}

/// Can inject a `ID` into a `ID!` but not the opposite.
fn can_inject_field_into_input(field: &FieldDefinitionRecord, input: &InputValueDefinitionRecord, batch: bool) -> bool {
    // if it's a union/interface/object, the input will have a different type, So we validate it
    // field by field later.
    match field.ty_record.definition_id {
        TypeDefinitionId::Enum(_) | TypeDefinitionId::Scalar(_) => {
            if field.ty_record.definition_id != input.ty_record.definition_id {
                return false;
            }
        }
        _ => {
            if !input.ty_record.definition_id.is_input_object() {
                return false;
            }
        }
    }
    let mut input = input.ty_record.wrapping;
    let field = field.ty_record.wrapping;
    if batch {
        let mut w = input.to_mutable();
        if w.pop_outermost_list_wrapping().is_none() {
            return false;
        }
        input = w.into();
    }
    input == field || input.non_null() == field
}
