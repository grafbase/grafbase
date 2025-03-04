use super::{SolveResult, Solver};

use crate::prepare::*;
use extension_catalog::ExtensionId;
use id_newtypes::IdRange;
use im::HashMap;
use query_solver::{
    Edge, Node,
    petgraph::{Direction, graph::NodeIndex, visit::EdgeRef},
};
use schema::{InjectionStage, Schema, StringId, TypeSystemDirective};
use walker::Walk;

impl Solver<'_> {
    pub(super) fn populate_modifiers_after_partition_generation(&mut self) -> SolveResult<()> {
        let mut response_modifier_definitions = Vec::new();
        let mut query_modifiers = vec![
            QueryModifierRecord {
                rule: QueryModifierRule::Authenticated,
                impacts_root_object: false,
                impacted_field_ids: Vec::new(),
            };
            self.solution.deduplicated_flat_sorted_executable_directives.len()
        ];
        for (directives, id) in
            std::mem::take(&mut self.solution.deduplicated_flat_sorted_executable_directives).into_iter()
        {
            query_modifiers[usize::from(id)].rule = QueryModifierRule::Executable { directives };
        }

        let mut deduplicated_query_modifier_rules = HashMap::new();
        let mut deduplicated_response_modifier_rules = HashMap::new();
        enum Rule {
            Query(QueryModifierRule),
            Resp(ResponseModifierRule),
        }
        let node_to_field = std::mem::take(&mut self.node_to_field);
        for (node_ix, field_id) in node_to_field.iter().enumerate() {
            let node_ix = NodeIndex::new(node_ix);
            let Some(field_id) = field_id else {
                continue;
            };
            if let PartitionFieldId::Data(field_id) = *field_id {
                let definition = self.output.query_plan[field_id].definition_id.walk(self.schema);
                for directive in definition.directives() {
                    let rule = match directive {
                        TypeSystemDirective::Authenticated(_) => Rule::Query(QueryModifierRule::Authenticated),
                        TypeSystemDirective::Authorized(dir) => {
                            if dir.node_record.is_some() {
                                if self.output.query_plan[field_id].output_id.is_none() {
                                    let output_id = Some(self.create_new_response_object_set_definition(node_ix));
                                    self.output.query_plan[field_id].output_id = output_id;
                                    for id in self.output.query_plan[field_id]
                                        .selection_set_record
                                        .data_field_ids_ordered_by_type_conditions_then_position
                                    {
                                        self.output.query_plan[id].parent_field_output_id = output_id;
                                    }
                                }
                                Rule::Resp(ResponseModifierRule::AuthorizedEdgeChild {
                                    directive_id: dir.id,
                                    definition_id: definition.id,
                                })
                            } else if dir.fields_record.is_some() {
                                if self.output.query_plan[field_id].parent_field_output_id.is_none() {
                                    let parent_ix = self
                                        .solution
                                        .graph
                                        .edges_directed(node_ix, Direction::Incoming)
                                        .find(|edge| matches!(edge.weight(), Edge::Field))
                                        .expect("Must have a parent field node or root")
                                        .source();
                                    let Some(PartitionFieldId::Data(parent_field_id)) =
                                        node_to_field[parent_ix.index()]
                                    else {
                                        tracing::error!("@authorized with fields on root field isn't supported yet");
                                        return Err(SolveError::InternalError);
                                    };
                                    let output_id = Some(self.create_new_response_object_set_definition(parent_ix));
                                    self.output.query_plan[parent_field_id].output_id = output_id;
                                    for id in self.output.query_plan[parent_field_id]
                                        .selection_set_record
                                        .data_field_ids_ordered_by_type_conditions_then_position
                                    {
                                        self.output.query_plan[id].parent_field_output_id = output_id;
                                    }
                                }
                                Rule::Resp(ResponseModifierRule::AuthorizedParentEdge {
                                    directive_id: dir.id,
                                    definition_id: definition.id,
                                })
                            } else if !dir.arguments.is_empty() {
                                Rule::Query(QueryModifierRule::AuthorizedFieldWithArguments {
                                    directive_id: dir.id,
                                    definition_id: definition.id,
                                    argument_ids: self.output.query_plan[field_id].argument_ids,
                                })
                            } else {
                                Rule::Query(QueryModifierRule::AuthorizedField {
                                    directive_id: dir.id,
                                    definition_id: definition.id,
                                })
                            }
                        }
                        TypeSystemDirective::Extension(directive) => {
                            if !directive.kind.is_authorization() {
                                continue;
                            }
                            match directive.max_arguments_stage() {
                                InjectionStage::Static => Rule::Query(QueryModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: ModifierTarget::Field(definition.id),
                                }),
                                InjectionStage::Query => Rule::Query(QueryModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: ModifierTarget::FieldWithArguments(
                                        definition.id,
                                        self.output.query_plan[field_id].argument_ids,
                                    ),
                                }),
                                InjectionStage::Response => unimplemented!("Not handled yet, GB-8610"),
                            }
                        }
                        TypeSystemDirective::RequiresScopes(dir) => {
                            Rule::Query(QueryModifierRule::RequiresScopes(dir.id))
                        }
                        TypeSystemDirective::Cost(_)
                        | TypeSystemDirective::Deprecated(_)
                        | TypeSystemDirective::ListSize(_) => continue,
                    };
                    match rule {
                        Rule::Query(rule) => {
                            let ix = deduplicated_query_modifier_rules
                                .entry(rule.clone())
                                .or_insert_with(|| {
                                    query_modifiers.push(QueryModifierRecord {
                                        rule,
                                        impacts_root_object: false,
                                        impacted_field_ids: Vec::new(),
                                    });
                                    query_modifiers.len() - 1
                                });
                            query_modifiers[*ix].impacted_field_ids.push(field_id.into());
                        }
                        Rule::Resp(rule) => {
                            let ix = deduplicated_response_modifier_rules.entry(rule).or_insert_with(|| {
                                response_modifier_definitions.push(ResponseModifierDefinitionRecord {
                                    rule,
                                    impacted_field_ids: Vec::new(),
                                });
                                response_modifier_definitions.len() - 1
                            });
                            response_modifier_definitions[*ix].impacted_field_ids.push(field_id);
                        }
                    }
                }

                let output_definition = definition.ty().definition();
                for directive in output_definition.directives() {
                    let rule = match directive {
                        TypeSystemDirective::Authenticated(_) => Rule::Query(QueryModifierRule::Authenticated),
                        TypeSystemDirective::Authorized(dir) => {
                            if dir.fields_record.is_some() {
                                Rule::Resp(ResponseModifierRule::AuthorizedEdgeChild {
                                    directive_id: dir.id,
                                    definition_id: definition.id,
                                })
                            } else {
                                Rule::Query(QueryModifierRule::AuthorizedDefinition {
                                    directive_id: dir.id,
                                    definition_id: output_definition.id(),
                                })
                            }
                        }
                        TypeSystemDirective::RequiresScopes(dir) => {
                            Rule::Query(QueryModifierRule::RequiresScopes(dir.id))
                        }
                        TypeSystemDirective::Extension(directive) => {
                            if !directive.kind.is_authorization() {
                                continue;
                            }
                            match directive.max_arguments_stage() {
                                InjectionStage::Static => Rule::Query(QueryModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: ModifierTarget::Definition(output_definition.id()),
                                }),
                                InjectionStage::Query => {
                                    unreachable!("Cannot depend on query arguments, it's not a field.")
                                }
                                InjectionStage::Response => unimplemented!("Not handled yet, GB-8610"),
                            }
                        }
                        TypeSystemDirective::Cost(_)
                        | TypeSystemDirective::Deprecated(_)
                        | TypeSystemDirective::ListSize(_) => continue,
                    };
                    match rule {
                        Rule::Query(rule) => {
                            let ix = deduplicated_query_modifier_rules
                                .entry(rule.clone())
                                .or_insert_with(|| {
                                    query_modifiers.push(QueryModifierRecord {
                                        rule,
                                        impacts_root_object: false,
                                        impacted_field_ids: Vec::new(),
                                    });
                                    query_modifiers.len() - 1
                                });
                            query_modifiers[*ix].impacted_field_ids.push(field_id.into());
                        }
                        Rule::Resp(rule) => {
                            let ix = deduplicated_response_modifier_rules.entry(rule).or_insert_with(|| {
                                response_modifier_definitions.push(ResponseModifierDefinitionRecord {
                                    rule,
                                    impacted_field_ids: Vec::new(),
                                });
                                response_modifier_definitions.len() - 1
                            });
                            response_modifier_definitions[*ix].impacted_field_ids.push(field_id);
                        }
                    }
                }
            }
            let Node::Field { id, .. } = self.solution.graph[node_ix] else {
                continue;
            };
            if let Some(id) = self.solution[id].flat_directive_id {
                query_modifiers[usize::from(id)].impacted_field_ids.push(*field_id);
            }
        }

        for directive in self.output.operation.root_object_id.walk(self.schema).directives() {
            let rule = match directive {
                TypeSystemDirective::Authenticated(_) => QueryModifierRule::Authenticated,
                TypeSystemDirective::Authorized(dir) => QueryModifierRule::AuthorizedDefinition {
                    directive_id: dir.id,
                    definition_id: self.output.operation.root_object_id.into(),
                },
                TypeSystemDirective::Extension(directive) => {
                    if !directive.kind.is_authorization() {
                        continue;
                    }
                    QueryModifierRule::Extension {
                        directive_id: directive.id,
                        target: ModifierTarget::Definition(self.output.operation.root_object_id.into()),
                    }
                }
                TypeSystemDirective::RequiresScopes(dir) => QueryModifierRule::RequiresScopes(dir.id),
                TypeSystemDirective::Cost(_)
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::ListSize(_) => continue,
            };
            let ix = deduplicated_query_modifier_rules
                .entry(rule.clone())
                .or_insert_with(|| {
                    query_modifiers.push(QueryModifierRecord {
                        rule,
                        impacts_root_object: true,
                        impacted_field_ids: Vec::new(),
                    });
                    query_modifiers.len() - 1
                });
            query_modifiers[*ix].impacts_root_object = true;
        }

        self.node_to_field = node_to_field;
        self.output.query_plan.query_modifiers = QueryModifiers::build(self.schema, query_modifiers);
        self.output.query_plan.response_modifier_definitions = response_modifier_definitions;

        Ok(())
    }
}

impl QueryModifiers {
    fn build(schema: &Schema, mut records: Vec<QueryModifierRecord>) -> Self {
        records.sort_unstable_by(|l, r| match (&l.rule, &r.rule) {
            (
                QueryModifierRule::Extension { directive_id: l, .. },
                QueryModifierRule::Extension { directive_id: r, .. },
            ) => {
                let l = &schema[*l];
                let r = &schema[*r];
                l.extension_id
                    .cmp(&r.extension_id)
                    .then_with(|| schema[l.name_id].cmp(&schema[r.name_id]))
            }
            (QueryModifierRule::Extension { .. }, _) => std::cmp::Ordering::Greater,
            (_, QueryModifierRule::Extension { .. }) => std::cmp::Ordering::Less,
            (_, _) => std::cmp::Ordering::Equal,
        });

        let mut by_extension = Vec::<(
            ExtensionId,
            IdRange<QueryModifierByDirectiveGroupId>,
            IdRange<QueryModifierId>,
        )>::new();
        let mut by_directive = Vec::<(StringId, IdRange<QueryModifierId>)>::new();
        let mut iter = records.iter().enumerate();
        let native_end = loop {
            let Some((native_end, modifier)) = iter.next() else {
                break records.len();
            };

            if let QueryModifierRule::Extension { directive_id, .. } = modifier.rule {
                let directive = &schema[directive_id];

                let modifier_ix = by_directive
                    .last()
                    .map(|(_, range)| range.end)
                    .unwrap_or(native_end.into());
                let directive_group_ix = by_directive.len();
                let mut extension_group = (
                    directive.extension_id,
                    IdRange::<QueryModifierByDirectiveGroupId>::from(directive_group_ix..directive_group_ix),
                    IdRange::<QueryModifierId>::from(modifier_ix..modifier_ix),
                );

                let mut directive_group = (
                    directive.name_id,
                    IdRange::<QueryModifierId>::from(modifier_ix..modifier_ix),
                );

                for (ix, modifier) in iter {
                    let extension_id = match modifier.rule {
                        QueryModifierRule::Extension { directive_id, .. } => schema[directive_id].extension_id,
                        _ => unreachable!(),
                    };
                    if extension_group.0 != extension_id {
                        directive_group.1.end = ix.into();
                        by_directive.push(directive_group);

                        extension_group.1.end = by_directive.len().into();
                        extension_group.2.end = ix.into();
                        by_extension.push(extension_group);

                        directive_group = (directive.name_id, (ix..ix).into());
                        let n = by_directive.len();
                        extension_group = (extension_id, (n..n).into(), (ix..ix).into());
                    } else if directive_group.0 != directive.name_id {
                        directive_group.1.end = ix.into();
                        by_directive.push(directive_group);
                        directive_group = (directive.name_id, (ix..ix).into());
                    }
                }
                directive_group.1.end = records.len().into();
                by_directive.push(directive_group);
                extension_group.1.end = by_directive.len().into();
                extension_group.2.end = records.len().into();
                by_extension.push(extension_group);

                break native_end;
            }
        };

        QueryModifiers {
            native_ids: (0..native_end).into(),
            by_extension,
            by_directive,
            records,
        }
    }
}
