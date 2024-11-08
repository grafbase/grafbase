use std::num::NonZero;

use id_newtypes::{BitSet, IdToMany};
use schema::{RequiresScopeSetIndex, RequiresScopesDirectiveId};
use serde::Deserialize;
use walker::Walk;

use crate::{
    execution::PreExecutionContext,
    operation::{InputValueContext, QueryModifierRule, Variables},
    plan::{
        DataPlanFieldId, OperationPlan, PlanContext, PlanField, PlanResult, QueryModifierDefinition,
        TypenamePlanFieldId,
    },
    response::{ConcreteObjectShapeId, ErrorCode, FieldShapeId, GraphqlError},
    Runtime,
};

#[allow(unused)]
#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct QueryModifications {
    pub is_any_field_skipped: bool,
    pub skipped_data_fields: BitSet<DataPlanFieldId>,
    pub skipped_typename_fields: BitSet<TypenamePlanFieldId>,
    #[indexed_by(ErrorId)]
    pub errors: Vec<GraphqlError>,
    pub concrete_shape_has_error: BitSet<ConcreteObjectShapeId>,
    pub field_shape_id_to_error_ids: IdToMany<FieldShapeId, ErrorId>,
    pub skipped_field_shapes: BitSet<FieldShapeId>,
    pub root_error_ids: Vec<ErrorId>,
    // sorted by scope id
    matched_scopes: Vec<(RequiresScopesDirectiveId, RequiresScopeSetIndex)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ErrorId(NonZero<u16>);

impl QueryModifications {
    pub(crate) async fn build(
        ctx: &PreExecutionContext<'_, impl Runtime>,
        operation_plan: &OperationPlan,
        variables: &Variables,
    ) -> PlanResult<Self> {
        Builder {
            ctx,
            plan_ctx: PlanContext {
                schema: ctx.schema(),
                operation_plan,
            },
            input_value_ctx: InputValueContext {
                schema: ctx.schema(),
                query_input_values: &operation_plan.query_input_values,
                variables,
            },
            field_shape_id_to_error_ids: Default::default(),
            modifications: QueryModifications {
                is_any_field_skipped: false,
                skipped_data_fields: BitSet::with_capacity(operation_plan.data_fields.len()),
                skipped_typename_fields: BitSet::with_capacity(operation_plan.typename_fields.len()),
                concrete_shape_has_error: BitSet::with_capacity(operation_plan.shapes.concrete.len()),
                errors: Vec::new(),
                field_shape_id_to_error_ids: Default::default(),
                root_error_ids: Vec::new(),
                matched_scopes: vec![],
                skipped_field_shapes: BitSet::with_capacity(operation_plan.shapes.fields.len()),
            },
        }
        .build()
        .await
    }

    #[allow(unused)]
    pub(super) fn matched_scope_set(&self, required_scope: RequiresScopesDirectiveId) -> Option<RequiresScopeSetIndex> {
        let index = self
            .matched_scopes
            .binary_search_by_key(&required_scope, |(id, _)| *id)
            .ok()?;

        Some(self.matched_scopes[index].1)
    }
}

struct Builder<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    plan_ctx: PlanContext<'op>,
    input_value_ctx: InputValueContext<'op>,
    field_shape_id_to_error_ids: Vec<(FieldShapeId, ErrorId)>,
    modifications: QueryModifications,
}

impl<'ctx, 'op, R: Runtime> Builder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) async fn build(mut self) -> PlanResult<QueryModifications> {
        let mut scope_jwt_claim = None;

        for modifier in self
            .plan_ctx
            .operation_plan
            .query_modifier_definitions
            .walk(self.plan_ctx)
        {
            match &modifier.rule {
                QueryModifierRule::Authenticated => {
                    if self.ctx.access_token().is_anonymous() {
                        self.handle_modifier_resulted_in_error(
                            modifier,
                            GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated),
                        )
                    }
                }
                QueryModifierRule::RequiresScopes(id) => {
                    let scope_jwt_claim = scope_jwt_claim.get_or_insert_with(|| {
                        self.ctx
                            .access_token()
                            .get_claim("scope")
                            .as_str()
                            .map(|scope| scope.split(' ').collect::<Vec<_>>())
                            .unwrap_or_default()
                    });

                    let Some(selected_scope_set) = id.walk(self.ctx.schema()).matches(scope_jwt_claim) else {
                        self.handle_modifier_resulted_in_error(
                            modifier,
                            GraphqlError::new("Insufficient scopes", ErrorCode::Unauthorized),
                        );
                        continue;
                    };

                    self.record_selected_scope_set(*id, selected_scope_set);
                }
                QueryModifierRule::AuthorizedField {
                    directive_id,
                    definition_id,
                    argument_ids,
                } => {
                    let directive = directive_id.walk(self.ctx.schema());
                    let verdict = self
                        .ctx
                        .hooks()
                        .authorize_edge_pre_execution(
                            definition_id.walk(self.ctx.schema()),
                            self.plan_ctx
                                .hydrate_arguments(
                                    // FIXME: just pass the argument_ids after migrating to QP.
                                    (argument_ids.start..argument_ids.end).into(),
                                    self.input_value_ctx.variables,
                                )
                                .with_selection_set(&directive.arguments),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(error) = verdict {
                        self.handle_modifier_resulted_in_error(modifier, error);
                    }
                }
                QueryModifierRule::AuthorizedDefinition {
                    directive_id,
                    definition_id: definition,
                } => {
                    let directive = directive_id.walk(self.ctx.schema());
                    let result = self
                        .ctx
                        .hooks()
                        .authorize_node_pre_execution(definition.walk(self.ctx.schema()), directive.metadata())
                        .await;

                    if let Err(error) = result {
                        self.handle_modifier_resulted_in_error(modifier, error);
                    }
                }
                QueryModifierRule::SkipInclude {
                    skip_input_value_ids,
                    include_input_value_ids,
                } => {
                    let not_included = include_input_value_ids.iter().any(|id| {
                        let included = bool::deserialize(id.walk(self.input_value_ctx))
                            .expect("at this point we've already checked the argument type");
                        !included
                    });

                    let skipped = skip_input_value_ids.iter().any(|id| {
                        bool::deserialize(id.walk(self.input_value_ctx))
                            .expect("at this point we've already checked the argument type")
                    });

                    if not_included || skipped {
                        self.handle_modifier_resulted_in_skipped_fields(modifier)
                    }
                }
            }
        }

        Ok(self.finalize())
    }

    fn finalize(mut self) -> QueryModifications {
        self.modifications.field_shape_id_to_error_ids = self.field_shape_id_to_error_ids.into();
        let mut field_shape_ids_with_errors = self.modifications.field_shape_id_to_error_ids.ids();
        if let Some(mut current) = field_shape_ids_with_errors.next() {
            'outer: for (concrete_shape_id, shape) in self.plan_ctx.operation_plan.shapes.concrete.iter().enumerate() {
                if current < shape.field_shape_ids.end {
                    let mut i = 0;
                    while let Some(field_shape_id) = shape.field_shape_ids.get(i) {
                        match field_shape_id.cmp(&current) {
                            std::cmp::Ordering::Less => {
                                i += 1;
                            }
                            std::cmp::Ordering::Equal => {
                                self.modifications
                                    .concrete_shape_has_error
                                    .set(ConcreteObjectShapeId::from(concrete_shape_id), true);
                                break;
                            }
                            std::cmp::Ordering::Greater => {
                                let Some(next) = field_shape_ids_with_errors.next() else {
                                    break 'outer;
                                };
                                current = next;
                            }
                        }
                    }
                }
            }
        }
        drop(field_shape_ids_with_errors);

        self.modifications
            .matched_scopes
            .sort_unstable_by_key(|(scope_id, _)| *scope_id);

        self.modifications
    }

    fn handle_modifier_resulted_in_error(&mut self, modifier: QueryModifierDefinition<'op>, error: GraphqlError) {
        let error_id = self.push_error(error);
        if modifier.impacts_root_object {
            self.modifications.root_error_ids.push(error_id);
        }
        self.modifications.is_any_field_skipped = true;
        for field in modifier.impacted_fields() {
            match field {
                PlanField::Typename(field) => {
                    self.modifications.skipped_typename_fields.set(field.id(), true);
                }
                PlanField::Data(field) => {
                    self.modifications.skipped_data_fields.set(field.id(), true);
                    for field_shape_id in field.shapes() {
                        self.field_shape_id_to_error_ids.push((field_shape_id, error_id));
                    }
                }
            }
        }
    }

    fn handle_modifier_resulted_in_skipped_fields(&mut self, modifier: QueryModifierDefinition<'op>) {
        self.modifications.is_any_field_skipped = true;
        for field in modifier.impacted_fields() {
            match field {
                PlanField::Typename(field) => {
                    self.modifications.skipped_typename_fields.set(field.id(), true);
                }
                PlanField::Data(field) => {
                    self.modifications.skipped_data_fields.set(field.id(), true);
                }
            }
        }
    }

    fn push_error(&mut self, error: GraphqlError) -> ErrorId {
        let id = ErrorId::from(self.modifications.errors.len());
        self.modifications.errors.push(error);
        id
    }

    fn record_selected_scope_set(&mut self, id: RequiresScopesDirectiveId, selected_scope_set: RequiresScopeSetIndex) {
        self.modifications.matched_scopes.push((id, selected_scope_set));
    }
}
