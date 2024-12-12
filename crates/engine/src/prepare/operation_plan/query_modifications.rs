use std::{num::NonZero, ops::Deref};

use id_newtypes::{BitSet, IdToMany};
use operation::{InputValueContext, Variables};
use query_solver::QueryOrSchemaFieldArgumentIds;
use serde::Deserialize;
use walker::Walk;

use crate::{
    prepare::{
        CachedOperation, CachedOperationContext, ConcreteShapeId, ErrorCode, FieldShapeId, GraphqlError,
        PartitionDataFieldId, PartitionField, PartitionTypenameFieldId, PrepareContext, QueryModifier,
        QueryModifierRule, RequiredFieldSetRecord,
    },
    Runtime,
};

use super::PlanResult;

#[allow(unused)]
#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct QueryModifications {
    pub is_any_field_skipped: bool,
    pub response_data_fields: BitSet<PartitionDataFieldId>,
    pub response_typename_fields: BitSet<PartitionTypenameFieldId>,
    pub subgraph_request_data_fields: BitSet<PartitionDataFieldId>,
    #[indexed_by(ErrorId)]
    pub errors: Vec<GraphqlError>,
    pub concrete_shape_has_error: BitSet<ConcreteShapeId>,
    pub field_shape_id_to_error_ids: IdToMany<FieldShapeId, ErrorId>,
    pub skipped_field_shapes: BitSet<FieldShapeId>,
    pub root_error_ids: Vec<ErrorId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ErrorId(NonZero<u16>);

impl QueryModifications {
    pub(crate) async fn build(
        ctx: &PrepareContext<'_, impl Runtime>,
        cached: &CachedOperation,
        variables: &Variables,
    ) -> PlanResult<Self> {
        Builder {
            ctx,
            operation_ctx: CachedOperationContext {
                schema: ctx.schema(),
                cached,
            },
            input_value_ctx: InputValueContext {
                schema: ctx.schema(),
                query_input_values: &cached.operation.query_input_values,
                variables,
            },
            field_shape_id_to_error_ids: Default::default(),
            modifications: QueryModifications {
                is_any_field_skipped: false,
                response_data_fields: cached.query_plan.response_data_fields.clone(),
                response_typename_fields: cached.query_plan.response_typename_fields.clone(),
                subgraph_request_data_fields: Default::default(),
                concrete_shape_has_error: BitSet::with_capacity(cached.shapes.concrete.len()),
                errors: Vec::new(),
                field_shape_id_to_error_ids: Default::default(),
                root_error_ids: Vec::new(),
                skipped_field_shapes: BitSet::with_capacity(cached.shapes.fields.len()),
            },
        }
        .build()
        .await
    }
}

struct Builder<'ctx, 'op, R: Runtime> {
    ctx: &'op PrepareContext<'ctx, R>,
    operation_ctx: CachedOperationContext<'op>,
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

        for modifier in self.operation_ctx.query_modifiers() {
            match &modifier.rule {
                QueryModifierRule::Authenticated => {
                    if self.ctx.access_token().is_anonymous() {
                        self.handle_authorization_modifier(
                            modifier,
                            AuthorizationModifierResult::Denied(Some(GraphqlError::new(
                                "Unauthenticated",
                                ErrorCode::Unauthenticated,
                            ))),
                        );
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

                    if id.walk(self.ctx.schema()).matches(scope_jwt_claim).is_none() {
                        self.handle_authorization_modifier(
                            modifier,
                            AuthorizationModifierResult::Denied(Some(GraphqlError::new(
                                "Insufficient scopes",
                                ErrorCode::Unauthorized,
                            ))),
                        );
                        continue;
                    };
                }
                QueryModifierRule::AuthorizedField {
                    directive_id,
                    definition_id,
                } => {
                    let directive = directive_id.walk(self.ctx.schema());
                    let verdict = self
                        .ctx
                        .hooks()
                        .authorize_edge_pre_execution(
                            definition_id.walk(self.ctx.schema()),
                            QueryOrSchemaFieldArgumentIds::default()
                                .walk(self.operation_ctx)
                                .view(&directive.arguments, self.input_value_ctx.variables),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(error) = verdict {
                        self.handle_authorization_modifier(modifier, AuthorizationModifierResult::Denied(Some(error)));
                    }
                }
                QueryModifierRule::AuthorizedFieldWithArguments {
                    directive_id,
                    definition_id,
                    argument_ids,
                } => {
                    tracing::warn!("with args");
                    let directive = directive_id.walk(self.ctx.schema());
                    let verdict = self
                        .ctx
                        .hooks()
                        .authorize_edge_pre_execution(
                            definition_id.walk(self.ctx.schema()),
                            argument_ids
                                .walk(self.operation_ctx)
                                .view(&directive.arguments, self.input_value_ctx.variables),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(error) = verdict {
                        self.handle_authorization_modifier(modifier, AuthorizationModifierResult::Denied(Some(error)));
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
                        self.handle_authorization_modifier(modifier, AuthorizationModifierResult::Denied(Some(error)));
                    }
                }
                QueryModifierRule::Executable { directives } => {
                    // GraphQL spec:
                    //   Stated conversely, the field or fragment must not be queried if either the @skip condition is true or the @include condition is false.
                    let is_skipped = directives.iter().any(|directive| match directive {
                        operation::ExecutableDirectiveId::Include(directive) => {
                            !bool::deserialize(directive.condition.walk(self.input_value_ctx))
                                .expect("at this point we've already checked the argument type")
                        }
                        operation::ExecutableDirectiveId::Skip(directive) => {
                            bool::deserialize(directive.condition.walk(self.input_value_ctx))
                                .expect("at this point we've already checked the argument type")
                        }
                    });

                    if is_skipped {
                        self.handle_skipped_field(modifier)
                    }
                }
            }
        }

        Ok(self.finalize())
    }

    fn finalize(mut self) -> QueryModifications {
        self.modifications.subgraph_request_data_fields = self.modifications.response_data_fields.clone();
        let mut requires_stack: Vec<&'op RequiredFieldSetRecord> =
            Vec::with_capacity(self.operation_ctx.query_partitions().len() * 2);

        for field in self.operation_ctx.data_fields() {
            if self.modifications.response_data_fields[field.id] {
                if !field.required_fields_record.is_empty() {
                    requires_stack.push(&field.as_ref().required_fields_record);
                }
                if !field.required_fields_record_by_supergraph.is_empty() {
                    requires_stack.push(&field.as_ref().required_fields_record_by_supergraph);
                }
            }
        }
        // TODO: Don't include partitions without included subgraph fields.
        for query_partition in self.operation_ctx.query_partitions() {
            requires_stack.push(&query_partition.as_ref().required_fields_record);
        }

        while let Some(required_fields) = requires_stack.pop() {
            for item in required_fields.deref() {
                self.modifications
                    .subgraph_request_data_fields
                    .set(item.data_field_id, true);
                requires_stack.push(&item.subselection_record);
                let field = item.data_field_id.walk(self.operation_ctx);
                if !field.required_fields_record.is_empty() {
                    requires_stack.push(&field.as_ref().required_fields_record);
                }
                if !field.required_fields_record_by_supergraph.is_empty() {
                    requires_stack.push(&field.as_ref().required_fields_record_by_supergraph);
                }
            }
        }

        for id in self.modifications.subgraph_request_data_fields.zeroes() {
            for field_shape_id in id.walk(self.operation_ctx).shapes() {
                self.modifications.skipped_field_shapes.set(field_shape_id, true);
            }
        }

        // Identify all concrete shapes with errors.
        self.modifications.field_shape_id_to_error_ids = self.field_shape_id_to_error_ids.into();
        let mut field_shape_ids_with_errors = self.modifications.field_shape_id_to_error_ids.ids();
        if let Some(mut current) = field_shape_ids_with_errors.next() {
            'outer: for (concrete_shape_id, shape) in self.operation_ctx.cached.shapes.concrete.iter().enumerate() {
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
                                    .set(ConcreteShapeId::from(concrete_shape_id), true);
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
    }

    fn handle_authorization_modifier(&mut self, modifier: QueryModifier<'op>, result: AuthorizationModifierResult) {
        match result {
            AuthorizationModifierResult::Denied(None) => {
                todo!()
            }
            AuthorizationModifierResult::Denied(Some(error)) => {
                let error_id = self.push_error(error);
                if modifier.impacts_root_object {
                    self.modifications.root_error_ids.push(error_id);
                }
                self.modifications.is_any_field_skipped = true;
                for field in modifier.impacted_fields() {
                    match field {
                        PartitionField::Typename(field) => {
                            self.modifications.response_typename_fields.set(field.id, false);
                        }
                        PartitionField::Data(field) => {
                            self.modifications.response_data_fields.set(field.id, false);
                            for field_shape_id in field.shapes() {
                                self.field_shape_id_to_error_ids.push((field_shape_id, error_id));
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_skipped_field(&mut self, modifier: QueryModifier<'op>) {
        self.modifications.is_any_field_skipped = true;
        for field in modifier.impacted_fields() {
            match field {
                PartitionField::Typename(field) => {
                    self.modifications.response_typename_fields.set(field.id, false);
                }
                PartitionField::Data(field) => {
                    self.modifications.response_data_fields.set(field.id, false);
                }
            }
        }
    }

    fn push_error(&mut self, error: GraphqlError) -> ErrorId {
        let id = ErrorId::from(self.modifications.errors.len());
        self.modifications.errors.push(error);
        id
    }
}

enum AuthorizationModifierResult {
    Denied(Option<GraphqlError>),
}
