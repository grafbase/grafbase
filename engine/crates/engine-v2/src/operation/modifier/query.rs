use id_newtypes::{BitSet, IdRange, IdToMany};
use schema::{RequiresScopeSetIndex, RequiresScopesDirectiveId, Schema};
use serde::Deserialize;

use crate::operation::FieldSkippingDirective;
use crate::{
    execution::{ErrorId, PlanningResult, PreExecutionContext},
    operation::{
        FieldId, PreparedOperation, PreparedOperationWalker, QueryModifierId, QueryModifierImpactedFieldId,
        QueryModifierRule, Variables,
    },
    response::{ConcreteObjectShapeId, ErrorCode, FieldShapeId, GraphqlError},
    Runtime,
};

#[derive(id_derives::IndexedFields)]
pub(crate) struct QueryModifications {
    pub is_any_field_skipped: bool,
    pub skipped_fields: BitSet<FieldId>,
    #[indexed_by(ErrorId)]
    pub errors: Vec<GraphqlError>,
    pub concrete_shape_has_error: BitSet<ConcreteObjectShapeId>,
    pub field_shape_id_to_error_ids: IdToMany<FieldShapeId, ErrorId>,
    pub skipped_field_shape_ids: BitSet<FieldShapeId>,
    pub root_error_ids: Vec<ErrorId>,
    matched_scopes: Vec<(RequiresScopesDirectiveId, RequiresScopeSetIndex)>,
}

impl QueryModifications {
    pub(crate) async fn build(
        ctx: &PreExecutionContext<'_, impl Runtime>,
        operation: &PreparedOperation,
        variables: &Variables,
    ) -> PlanningResult<Self> {
        Builder {
            ctx,
            operation,
            variables,
            field_shape_id_to_error_ids_builder: Default::default(),
            modifications: Self::default_for(operation),
        }
        .build()
        .await
    }

    pub fn default_for(operation: &PreparedOperation) -> Self {
        QueryModifications {
            is_any_field_skipped: false,
            skipped_fields: BitSet::init_with(false, operation.fields.len()),
            concrete_shape_has_error: BitSet::init_with(false, operation.response_blueprint.shapes.concrete.len()),
            errors: Vec::new(),
            field_shape_id_to_error_ids: Default::default(),
            root_error_ids: Vec::new(),
            matched_scopes: vec![],
            skipped_field_shape_ids: BitSet::init_with(false, operation.fields.len()),
        }
    }

    pub(in crate::operation) fn matched_scope_set(
        &self,
        required_scope: RequiresScopesDirectiveId,
    ) -> Option<RequiresScopeSetIndex> {
        let index = self
            .matched_scopes
            .binary_search_by_key(&required_scope, |(id, _)| *id)
            .ok()?;

        Some(self.matched_scopes[index].1)
    }
}

struct Builder<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: &'op PreparedOperation,
    variables: &'op Variables,
    field_shape_id_to_error_ids_builder: Vec<(FieldShapeId, ErrorId)>,
    modifications: QueryModifications,
}

impl<'ctx, 'op, R: Runtime> Builder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) async fn build(mut self) -> PlanningResult<QueryModifications> {
        let mut scopes = None;

        for (i, modifier) in self.operation.query_modifiers.iter().enumerate() {
            let modifier_id = QueryModifierId::from(i);

            match modifier.rule {
                QueryModifierRule::Authenticated => {
                    if self.ctx.access_token().is_anonymous() {
                        self.handle_modifier_resulted_in_error(
                            modifier_id,
                            modifier.impacted_fields,
                            GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated),
                        )
                    }
                }
                QueryModifierRule::RequiresScopes(id) => {
                    let scopes = scopes.get_or_insert_with(|| {
                        self.ctx
                            .access_token()
                            .get_claim("scope")
                            .as_str()
                            .map(|scope| scope.split(' ').collect::<Vec<_>>())
                            .unwrap_or_default()
                    });

                    let Some(selected_scope_set) = self.schema().walk(id).matches(scopes) else {
                        self.handle_modifier_resulted_in_error(
                            modifier_id,
                            modifier.impacted_fields,
                            GraphqlError::new("Insufficient scopes", ErrorCode::Unauthorized),
                        );
                        continue;
                    };

                    self.record_selected_scope_set(id, selected_scope_set);
                }
                QueryModifierRule::AuthorizedField {
                    directive_id,
                    definition_id,
                    argument_ids,
                } => {
                    let directive = &self.schema().walk(directive_id);
                    let verdict = self
                        .ctx
                        .hooks()
                        .authorize_edge_pre_execution(
                            self.schema().walk(definition_id),
                            self.walker()
                                .walk(argument_ids)
                                .with_selection_set(&directive.arguments),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(err) = verdict {
                        self.handle_modifier_resulted_in_error(modifier_id, modifier.impacted_fields, err);
                    }
                }
                QueryModifierRule::AuthorizedDefinition {
                    directive_id,
                    definition_id: definition,
                } => {
                    let directive = &self.schema().walk(directive_id);
                    let result = self
                        .ctx
                        .hooks()
                        .authorize_node_pre_execution(self.ctx.schema().walk(definition), directive.metadata())
                        .await;

                    if let Err(err) = result {
                        self.handle_modifier_resulted_in_error(modifier_id, modifier.impacted_fields, err);
                    }
                }
                QueryModifierRule::Skip { input_value_id, r#type } => {
                    let walker = self.walker().walk(&self.operation.query_input_values[input_value_id]);
                    let argument =
                        bool::deserialize(walker).expect("at this point we've already checked the argument type");
                    let skipped = match r#type {
                        FieldSkippingDirective::Skip => argument,
                        FieldSkippingDirective::Include => !argument,
                    };
                    if skipped {
                        self.modifications.is_any_field_skipped = true;
                        for &field_id in &self.operation[modifier.impacted_fields] {
                            self.modifications.skipped_fields.set(field_id, true);
                            for field_shape_id in
                                self.operation.response_blueprint.field_to_shape_ids.find_all(field_id)
                            {
                                self.modifications.skipped_field_shape_ids.set(*field_shape_id, true);
                            }
                        }
                    }
                }
            }
        }

        Ok(self.finalize())
    }

    fn finalize(mut self) -> QueryModifications {
        self.modifications.field_shape_id_to_error_ids = self.field_shape_id_to_error_ids_builder.into();
        let mut field_shape_ids_with_errors = self.modifications.field_shape_id_to_error_ids.ids();
        if let Some(mut current) = field_shape_ids_with_errors.next() {
            'outer: for (concrete_shape_id, shape) in
                self.operation.response_blueprint.shapes.concrete.iter().enumerate()
            {
                if current < shape.field_shape_ids.end() {
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

    fn handle_modifier_resulted_in_error(
        &mut self,
        id: QueryModifierId,
        impacted_fields: IdRange<QueryModifierImpactedFieldId>,
        error: GraphqlError,
    ) {
        self.modifications.is_any_field_skipped = true;
        let error_id = self.push_error(error);
        if self.operation.root_query_modifier_ids.contains(&id) {
            self.modifications.root_error_ids.push(error_id);
        }
        for &field_id in &self.operation[impacted_fields] {
            self.modifications.skipped_fields.set(field_id, true);
            for field_shape_id in self.operation.response_blueprint.field_to_shape_ids.find_all(field_id) {
                self.field_shape_id_to_error_ids_builder
                    .push((*field_shape_id, error_id));
            }
        }
    }

    fn push_error(&mut self, error: GraphqlError) -> ErrorId {
        let id = ErrorId::from(self.modifications.errors.len());
        self.modifications.errors.push(error);
        id
    }

    fn walker(&self) -> PreparedOperationWalker<'op, ()> {
        PreparedOperationWalker {
            schema: self.ctx.schema(),
            operation: self.operation,
            variables: self.variables,
            item: (),
        }
    }

    fn schema(&self) -> &'ctx Schema {
        &self.ctx.engine.schema
    }

    fn record_selected_scope_set(&mut self, id: RequiresScopesDirectiveId, selected_scope_set: RequiresScopeSetIndex) {
        self.modifications.matched_scopes.push((id, selected_scope_set));
    }
}
