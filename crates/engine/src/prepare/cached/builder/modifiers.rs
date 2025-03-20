use super::{SolveResult, Solver};

use crate::prepare::*;
use extension_catalog::ExtensionId;
use id_newtypes::IdRange;
use im::HashMap;
use query_solver::{
    Edge, Node,
    petgraph::{Direction, graph::NodeIndex, visit::EdgeRef},
};
use schema::{CompositeTypeId, DefinitionId, InjectionStage, Schema, StringId, TypeSystemDirective};
use walker::Walk;

impl Solver<'_> {
    pub(super) fn populate_modifiers_after_partition_generation(&mut self) -> SolveResult<()> {
        let mut accumulator = ModifierAccumulator {
            deduplicated_query_modifier_rules: HashMap::new(),
            query_modifiers: vec![
                QueryModifierRecord {
                    rule: QueryModifierRule::Authenticated,
                    impacts_root_object: false,
                    impacted_field_ids: Vec::new(),
                };
                self.solution.deduplicated_flat_sorted_executable_directives.len()
            ],
            deduplicated_response_modifier_rules: HashMap::new(),
            response_modifier_definitions: Vec::new(),
        };

        for (directives, id) in
            std::mem::take(&mut self.solution.deduplicated_flat_sorted_executable_directives).into_iter()
        {
            accumulator.query_modifiers[usize::from(id)].rule = QueryModifierRule::Executable { directives };
        }

        let node_to_field = std::mem::take(&mut self.node_to_field);
        for (node_ix, field_id) in node_to_field.iter().enumerate() {
            let node_ix = NodeIndex::new(node_ix);
            let Some(field_id) = field_id else {
                continue;
            };
            if let Node::Field { id, .. } = self.solution.graph[node_ix] {
                if let Some(id) = self.solution[id].flat_directive_id {
                    accumulator.query_modifiers[usize::from(id)]
                        .impacted_field_ids
                        .push(*field_id);
                }
            }

            let PartitionFieldId::Data(field_id) = *field_id else {
                continue;
            };
            let field_definition = self.output.query_plan[field_id].definition_id.walk(self.schema);
            for directive in field_definition.directives() {
                let rule = match directive {
                    TypeSystemDirective::Authenticated(_) => Rule::Query(QueryModifierRule::Authenticated),
                    TypeSystemDirective::Authorized(dir) => {
                        if dir.node_record.is_some() {
                            if self.output.query_plan[field_id].output_id.is_none() {
                                let output_id = Some(self.create_new_response_object_set_definition(node_ix));
                                self.output.query_plan[field_id].output_id = output_id;
                            }
                            Rule::Resp(ResponseModifierRule::AuthorizedEdgeChild {
                                directive_id: dir.id,
                                definition_id: field_definition.id,
                            })
                        } else if dir.fields_record.is_some() {
                            self.ensure_parent_field_ouput_is_tracked(field_id, node_ix, &node_to_field)?;
                            Rule::Resp(ResponseModifierRule::AuthorizedParentEdge {
                                directive_id: dir.id,
                                definition_id: field_definition.id,
                            })
                        } else if !dir.arguments.is_empty() {
                            Rule::Query(QueryModifierRule::AuthorizedFieldWithArguments {
                                directive_id: dir.id,
                                definition_id: field_definition.id,
                                argument_ids: self.output.query_plan[field_id].argument_ids,
                            })
                        } else {
                            Rule::Query(QueryModifierRule::AuthorizedField {
                                directive_id: dir.id,
                                definition_id: field_definition.id,
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
                                target: QueryModifierTarget::Site(field_definition.id.into()),
                            }),
                            InjectionStage::Query => Rule::Query(QueryModifierRule::Extension {
                                directive_id: directive.id,
                                target: QueryModifierTarget::FieldWithArguments(
                                    field_definition.id,
                                    self.output.query_plan[field_id].argument_ids,
                                ),
                            }),
                            InjectionStage::Response => {
                                self.ensure_parent_field_ouput_is_tracked(field_id, node_ix, &node_to_field)?;
                                let query_rule = Rule::Query(QueryModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: if directive
                                        .argument_records()
                                        .iter()
                                        .any(|arg| arg.injection_stage.is_query())
                                    {
                                        QueryModifierTarget::FieldWithArguments(
                                            field_definition.id,
                                            self.output.query_plan[field_id].argument_ids,
                                        )
                                    } else {
                                        QueryModifierTarget::Site(field_definition.id.into())
                                    },
                                });
                                accumulator.insert(query_rule, Some(field_id));

                                Rule::Resp(ResponseModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: ResponseModifierRuleTarget::Field(field_definition.id),
                                })
                            }
                        }
                    }
                    TypeSystemDirective::RequiresScopes(dir) => Rule::Query(QueryModifierRule::RequiresScopes(dir.id)),
                    TypeSystemDirective::Cost(_)
                    | TypeSystemDirective::Deprecated(_)
                    | TypeSystemDirective::ListSize(_) => continue,
                };
                accumulator.insert(rule, Some(field_id));
            }

            if self.output.query_plan[field_id]
                .parent_field_id
                .map(|id| {
                    let parent_output_id = self.output.query_plan[id]
                        .definition_id
                        .walk(self.schema)
                        .ty()
                        .definition_id;
                    CompositeTypeId::maybe_from(parent_output_id).expect("Could not have children fields otherwise")
                        != field_definition.parent_entity_id.into()
                })
                .unwrap_or_default()
            {
                let definition_id = DefinitionId::from(field_definition.parent_entity_id);
                for directive in field_definition.parent_entity().directives() {
                    let rule = match directive {
                        TypeSystemDirective::Authenticated(_) => Rule::Query(QueryModifierRule::Authenticated),
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
                                    target: QueryModifierTarget::Site(definition_id.into()),
                                }),
                                InjectionStage::Query => {
                                    unreachable!("Cannot depend on query arguments, it's not a field.")
                                }
                                InjectionStage::Response => {
                                    self.ensure_parent_field_ouput_is_tracked(field_id, node_ix, &node_to_field)?;
                                    let query_rule = Rule::Query(QueryModifierRule::Extension {
                                        directive_id: directive.id,
                                        target: QueryModifierTarget::Site(definition_id.into()),
                                    });
                                    accumulator.insert(query_rule, Some(field_id));
                                    Rule::Resp(ResponseModifierRule::Extension {
                                        directive_id: directive.id,
                                        target: ResponseModifierRuleTarget::FieldParentEntity(
                                            field_definition.parent_entity_id,
                                        ),
                                    })
                                }
                            }
                        }
                        TypeSystemDirective::Cost(_)
                        | TypeSystemDirective::Deprecated(_)
                        | TypeSystemDirective::ListSize(_)
                        | TypeSystemDirective::Authorized(_) => continue,
                    };
                    accumulator.insert(rule, Some(field_id));
                }
            }

            let output_definition = field_definition.ty().definition();
            for directive in output_definition.directives() {
                let rule = match directive {
                    TypeSystemDirective::Authenticated(_) => Rule::Query(QueryModifierRule::Authenticated),
                    TypeSystemDirective::Authorized(dir) => {
                        if dir.fields_record.is_some() {
                            Rule::Resp(ResponseModifierRule::AuthorizedEdgeChild {
                                directive_id: dir.id,
                                definition_id: field_definition.id,
                            })
                        } else {
                            Rule::Query(QueryModifierRule::AuthorizedDefinition {
                                directive_id: dir.id,
                                definition_id: output_definition.id(),
                            })
                        }
                    }
                    TypeSystemDirective::RequiresScopes(dir) => Rule::Query(QueryModifierRule::RequiresScopes(dir.id)),
                    TypeSystemDirective::Extension(directive) => {
                        if !directive.kind.is_authorization() {
                            continue;
                        }
                        match directive.max_arguments_stage() {
                            InjectionStage::Static => Rule::Query(QueryModifierRule::Extension {
                                directive_id: directive.id,
                                target: QueryModifierTarget::Site(output_definition.id().into()),
                            }),
                            InjectionStage::Query => {
                                unreachable!("Cannot depend on query arguments, it's not a field.")
                            }
                            InjectionStage::Response => {
                                if self.output.query_plan[field_id].output_id.is_none() {
                                    let output_id = Some(self.create_new_response_object_set_definition(node_ix));
                                    self.output.query_plan[field_id].output_id = output_id;
                                }
                                let query_rule = Rule::Query(QueryModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: QueryModifierTarget::Site(output_definition.id().into()),
                                });
                                accumulator.insert(query_rule, Some(field_id));
                                Rule::Resp(ResponseModifierRule::Extension {
                                    directive_id: directive.id,
                                    target: ResponseModifierRuleTarget::FieldOutput(output_definition.id()),
                                })
                            }
                        }
                    }
                    TypeSystemDirective::Cost(_)
                    | TypeSystemDirective::Deprecated(_)
                    | TypeSystemDirective::ListSize(_) => continue,
                };
                accumulator.insert(rule, Some(field_id));
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
                        target: QueryModifierTarget::Site(self.output.operation.root_object_id.into()),
                    }
                }
                TypeSystemDirective::RequiresScopes(dir) => QueryModifierRule::RequiresScopes(dir.id),
                TypeSystemDirective::Cost(_)
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::ListSize(_) => continue,
            };
            accumulator.insert(Rule::Query(rule), None);
        }

        self.node_to_field = node_to_field;
        self.output.query_plan.query_modifiers = QueryModifiers::build(self.schema, accumulator.query_modifiers);
        self.output.query_plan.response_modifier_definitions = accumulator.response_modifier_definitions;

        Ok(())
    }
}

enum Rule {
    Query(QueryModifierRule),
    Resp(ResponseModifierRule),
}

struct ModifierAccumulator {
    deduplicated_query_modifier_rules: HashMap<QueryModifierRule, usize>,
    query_modifiers: Vec<QueryModifierRecord>,
    deduplicated_response_modifier_rules: HashMap<ResponseModifierRule, usize>,
    response_modifier_definitions: Vec<ResponseModifierDefinitionRecord>,
}

impl ModifierAccumulator {
    fn insert(&mut self, rule: Rule, field_id: Option<PartitionDataFieldId>) {
        match rule {
            Rule::Query(rule) => {
                let ix = self
                    .deduplicated_query_modifier_rules
                    .entry(rule.clone())
                    .or_insert_with(|| {
                        self.query_modifiers.push(QueryModifierRecord {
                            rule,
                            impacts_root_object: false,
                            impacted_field_ids: Vec::new(),
                        });
                        self.query_modifiers.len() - 1
                    });
                if let Some(field_id) = field_id {
                    self.query_modifiers[*ix].impacted_field_ids.push(field_id.into());
                } else {
                    self.query_modifiers[*ix].impacts_root_object = true;
                }
            }
            Rule::Resp(rule) => {
                let ix = self
                    .deduplicated_response_modifier_rules
                    .entry(rule)
                    .or_insert_with(|| {
                        self.response_modifier_definitions
                            .push(ResponseModifierDefinitionRecord {
                                rule,
                                impacted_field_ids: Vec::new(),
                            });
                        self.response_modifier_definitions.len() - 1
                    });
                let Some(field_id) = field_id else {
                    unreachable!(
                        "Modifying the root object/__typename with post-execution authorization is not supported."
                    );
                };
                self.response_modifier_definitions[*ix]
                    .impacted_field_ids
                    .push(field_id);
            }
        }
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

                let modifier_ix: QueryModifierId = native_end.into();
                let directive_group_ix: usize = 0;
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
                    let directive = match modifier.rule {
                        QueryModifierRule::Extension { directive_id, .. } => &schema[directive_id],
                        _ => unreachable!(),
                    };
                    if extension_group.0 != directive.extension_id {
                        directive_group.1.end = ix.into();
                        by_directive.push(directive_group);

                        extension_group.1.end = by_directive.len().into();
                        extension_group.2.end = ix.into();
                        by_extension.push(extension_group);

                        directive_group = (directive.name_id, (ix..ix).into());
                        let n = by_directive.len();
                        extension_group = (directive.extension_id, (n..n).into(), (ix..ix).into());
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

impl Solver<'_> {
    fn ensure_parent_field_ouput_is_tracked(
        &mut self,
        field_id: PartitionDataFieldId,
        node_ix: NodeIndex,
        node_to_field: &[Option<PartitionFieldId>],
    ) -> SolveResult<()> {
        if self.output.query_plan[field_id]
            .parent_field_id
            .map(|id| self.output.query_plan[id].output_id.is_none())
            .unwrap_or_default()
        {
            let parent_ix = self
                .solution
                .graph
                .edges_directed(node_ix, Direction::Incoming)
                .find(|edge| matches!(edge.weight(), Edge::Field))
                .expect("Must have a parent field node or root")
                .source();
            let Some(PartitionFieldId::Data(parent_field_id)) = node_to_field[parent_ix.index()] else {
                tracing::error!("@authorized with fields on root field isn't supported yet");
                return Err(SolveError::InternalError);
            };
            let output_id = Some(self.create_new_response_object_set_definition(parent_ix));
            self.output.query_plan[parent_field_id].output_id = output_id;
        }

        Ok(())
    }
}
