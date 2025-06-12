use std::{num::NonZero, ops::Deref};

use futures::future::FutureExt;
use id_newtypes::{BitSet, IdRange, IdToMany};
use operation::{InputValueContext, Variables};
use runtime::extension::{
    AuthorizationDecisions, AuthorizationExtension as _, QueryAuthorizationDecisions, QueryElement,
};
use schema::DirectiveSiteId;
use serde::Deserialize;
use walker::Walk;

use crate::{
    Runtime,
    execution::find_matching_denied_header,
    prepare::{
        CachedOperation, CachedOperationContext, ConcreteShapeId, DataFieldId, Derive, ErrorCode, FieldShapeId,
        GraphqlError, PartitionField, PlanFieldArguments, PrepareContext, QueryModifierId, QueryModifierRecord,
        QueryModifierRule, QueryModifierTarget, QueryOrStaticExtensionDirectiveArugmentsView, RequiredFieldSetRecord,
        TypenameFieldId, create_extension_directive_query_view,
    },
};

use super::PlanResult;

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct QueryModifications {
    pub included_response_data_fields: BitSet<DataFieldId>,
    pub included_response_typename_fields: BitSet<TypenameFieldId>,
    pub included_subgraph_request_data_fields: BitSet<DataFieldId>,
    #[indexed_by(QueryErrorId)]
    pub errors: Vec<GraphqlError>,
    pub concrete_shape_has_error: BitSet<ConcreteShapeId>,
    pub field_shape_id_to_error_ids: IdToMany<FieldShapeId, QueryErrorId>,
    pub root_error_ids: Vec<QueryErrorId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct QueryErrorId(NonZero<u16>);

impl QueryModifications {
    pub(crate) async fn build(
        ctx: &mut PrepareContext<'_, impl Runtime>,
        cached: &CachedOperation,
        variables: &Variables,
    ) -> PlanResult<Self> {
        Builder {
            operation_ctx: CachedOperationContext {
                schema: ctx.schema(),
                cached,
            },
            input_value_ctx: InputValueContext {
                schema: ctx.schema(),
                query_input_values: &cached.operation.query_input_values,
                variables,
            },
            ctx,
            field_id_to_error_ids: Default::default(),
            modifications: QueryModifications {
                included_response_data_fields: cached.query_plan.response_data_fields.clone(),
                included_response_typename_fields: cached.query_plan.response_typename_fields.clone(),
                // We initialize the subgraph_request_data_fields later from the
                // response_data_fields.
                included_subgraph_request_data_fields: Default::default(),
                concrete_shape_has_error: BitSet::with_capacity(cached.shapes.concrete.len()),
                errors: Vec::new(),
                field_shape_id_to_error_ids: Default::default(),
                root_error_ids: Vec::new(),
            },
        }
        .build()
        .await
    }
}

struct Builder<'ctx, 'op, R: Runtime> {
    ctx: &'op mut PrepareContext<'ctx, R>,
    operation_ctx: CachedOperationContext<'op>,
    input_value_ctx: InputValueContext<'op>,
    field_id_to_error_ids: Vec<(DataFieldId, QueryErrorId)>,
    modifications: QueryModifications,
}

impl<'ctx, 'op, R: Runtime> Builder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) async fn build(mut self) -> PlanResult<QueryModifications> {
        // Hooks authorization will go away and @authenticiated &
        // @requiresScopes will be an extension. So we're left with skip/include in the host part
        // which don't need I/O. So no need to parallelize that today.
        let modifiers = &self.operation_ctx.cached.query_plan.query_modifiers;
        self.handle_native_modifiers(&modifiers[modifiers.native_ids]).await?;

        if !modifiers.by_extension.is_empty() {
            self.handle_extensions().await?;
        }

        Ok(self.finalize())
    }

    async fn handle_extensions(&mut self) -> PlanResult<()> {
        let modifiers = &self.operation_ctx.cached.query_plan.query_modifiers;
        let schema = self.ctx.schema();
        let operation_ctx = self.operation_ctx;
        let variables = &self.input_value_ctx.variables;

        let subgraph_default_headers_override = self.ctx.request_context.subgraph_default_headers.clone();

        let extensions = modifiers
            .by_extension
            .iter()
            .copied()
            .map(|(extension_id, directive_range, query_elements_range)| {
                (extension_id, directive_range.into(), query_elements_range.into())
            })
            .collect::<Vec<_>>();

        let (mut subgraph_default_headers_override, decisions) = self
            .ctx
            .extensions()
            .authorize_query(
                &self.ctx.request_context.extension_context,
                subgraph_default_headers_override,
                self.ctx.access_token().as_ref(),
                extensions,
                modifiers
                    .by_directive
                    .iter()
                    .copied()
                    .map(|(name_id, range)| (schema[name_id].as_str(), range.into())),
                modifiers
                    .records
                    .iter()
                    .map(|modifier| match modifier {
                        QueryModifierRecord {
                            rule: QueryModifierRule::Extension { directive_id, target },
                            ..
                        } => (directive_id, target),
                        _ => unreachable!("Not an extension modifier"),
                    })
                    .map(move |(directive_id, target)| {
                        let directive = directive_id.walk(operation_ctx);
                        match &target {
                            QueryModifierTarget::FieldWithArguments(definition, argument_ids) => QueryElement {
                                site: DirectiveSiteId::from(*definition).walk(operation_ctx),
                                arguments: QueryOrStaticExtensionDirectiveArugmentsView::Query(
                                    create_extension_directive_query_view(
                                        schema,
                                        directive,
                                        argument_ids.walk(operation_ctx),
                                        variables,
                                    ),
                                ),
                            },
                            QueryModifierTarget::Site(id) => QueryElement {
                                site: id.walk(operation_ctx),
                                arguments: QueryOrStaticExtensionDirectiveArugmentsView::Static(
                                    directive.static_arguments(),
                                ),
                            },
                        }
                    }),
            )
            .boxed()
            .await?;

        // TODO: Use http::HeaderMap.retain if it comes out.
        let denied_header_names = subgraph_default_headers_override
            .keys()
            .filter_map(|name| find_matching_denied_header(name))
            .collect::<Vec<_>>();
        for name in denied_header_names {
            subgraph_default_headers_override.remove(name);
        }
        self.ctx.gql_context.subgraph_default_headers_override = Some(subgraph_default_headers_override);
        for QueryAuthorizationDecisions {
            query_elements_range,
            decisions,
            ..
        } in decisions
        {
            self.handle_extension_decisions(query_elements_range.into(), decisions);
        }

        Ok(())
    }

    fn handle_extension_decisions(
        &mut self,
        modifier_ids: IdRange<QueryModifierId>,
        decisions: AuthorizationDecisions,
    ) {
        let modifiers = &self.operation_ctx.cached.query_plan.query_modifiers;
        match decisions {
            AuthorizationDecisions::GrantAll => (),
            AuthorizationDecisions::DenyAll(error) => {
                let error_id = self.push_error(error);
                for modifier in &modifiers[modifier_ids] {
                    self.deny_field(modifier, error_id);
                }
            }
            AuthorizationDecisions::DenySome {
                element_to_error,
                mut errors,
            } => {
                let offset = self.modifications.errors.len();
                self.modifications.errors.append(&mut errors);
                for (element_ix, error_ix) in element_to_error {
                    let modifier = &modifiers[modifier_ids.get(element_ix as usize).unwrap()];
                    self.deny_field(modifier, (offset + error_ix as usize).into());
                }
            }
        }
    }

    async fn handle_native_modifiers(&mut self, query_modifiers: &'op [QueryModifierRecord]) -> PlanResult<()> {
        let mut scope_jwt_claim = None;

        for modifier in query_modifiers {
            match &modifier.rule {
                QueryModifierRule::Extension { .. } => unreachable!("Not a native modifier"),
                QueryModifierRule::Authenticated => {
                    if self.ctx.access_token().is_anonymous() {
                        let error_id =
                            self.push_error(GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated));
                        self.deny_field(modifier, error_id);
                    }
                }
                QueryModifierRule::RequiresScopes(id) => {
                    let scope_jwt_claim = scope_jwt_claim.get_or_insert_with(|| {
                        self.ctx
                            .access_token()
                            .get_claim("scope")
                            .and_then(|value| value.as_str())
                            .map(|scope| scope.split(' ').collect::<Vec<_>>())
                            .unwrap_or_default()
                    });

                    if id.walk(self.ctx.schema()).matches(scope_jwt_claim).is_none() {
                        let error_id =
                            self.push_error(GraphqlError::new("Insufficient scopes", ErrorCode::Unauthorized));
                        self.deny_field(modifier, error_id);
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
                            PlanFieldArguments::empty(self.operation_ctx)
                                .query_view(&directive.arguments, self.input_value_ctx.variables),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(error) = verdict {
                        let error_id = self.push_error(error);
                        self.deny_field(modifier, error_id);
                    }
                }
                QueryModifierRule::AuthorizedFieldWithArguments {
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
                            argument_ids
                                .walk(self.operation_ctx)
                                .query_view(&directive.arguments, self.input_value_ctx.variables),
                            directive.metadata(),
                        )
                        .await;
                    if let Err(error) = verdict {
                        let error_id = self.push_error(error);
                        self.deny_field(modifier, error_id);
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
                        let error_id = self.push_error(error);
                        self.deny_field(modifier, error_id);
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
                        self.skip_field(modifier)
                    }
                }
            }
        }

        Ok(())
    }

    fn finalize(self) -> QueryModifications {
        let Self {
            mut modifications,
            operation_ctx,
            field_id_to_error_ids,
            ..
        } = self;
        modifications.included_subgraph_request_data_fields = modifications.included_response_data_fields.clone();

        // Unless required for other fields, they're not needed anymore.
        let mut field_shape_id_to_error_id = Vec::with_capacity(field_id_to_error_ids.len());
        for (id, error_id) in field_id_to_error_ids {
            modifications.included_subgraph_request_data_fields.set(id, false);
            for id in id.walk(operation_ctx).shape_ids() {
                field_shape_id_to_error_id.push((id, error_id));
            }
        }
        modifications.field_shape_id_to_error_ids = field_shape_id_to_error_id.into();

        let mut requires_stack: Vec<&'op RequiredFieldSetRecord> =
            Vec::with_capacity(operation_ctx.query_partitions().len() * 2);
        let mut derive_stack: Vec<DataFieldId> = Vec::new();

        for id in modifications.included_response_data_fields.ones() {
            let field = id.walk(operation_ctx);
            if !field.required_fields_record.is_empty() {
                requires_stack.push(&field.as_ref().required_fields_record);
            }
            if !field.required_fields_record_by_supergraph.is_empty() {
                requires_stack.push(&field.as_ref().required_fields_record_by_supergraph);
            }
            if let Some(Derive::From(id))
            | Some(Derive::Root {
                batch_field_id: Some(id),
            }) = field.derive
            {
                derive_stack.push(id);
            }
        }

        // TODO: Don't include partitions without included subgraph fields.
        for query_partition in operation_ctx.query_partitions() {
            requires_stack.push(&query_partition.as_ref().required_fields_record);
        }

        while !requires_stack.is_empty() || !derive_stack.is_empty() {
            for id in derive_stack.drain(..) {
                if !modifications.included_subgraph_request_data_fields.put(id) {
                    let field = id.walk(operation_ctx);
                    // if it wasn't already processed
                    if !modifications.included_response_data_fields[field.id]
                        && !field.required_fields_record.is_empty()
                    {
                        requires_stack.push(&field.as_ref().required_fields_record);
                    }
                }
            }
            while let Some(required_fields) = requires_stack.pop() {
                for item in required_fields.deref() {
                    modifications
                        .included_subgraph_request_data_fields
                        .set(item.data_field_id, true);
                    requires_stack.push(&item.subselection_record);
                    let field = item.data_field_id.walk(operation_ctx);
                    if !field.required_fields_record.is_empty() {
                        requires_stack.push(&field.as_ref().required_fields_record);
                    }
                    if let Some(Derive::From(id))
                    | Some(Derive::Root {
                        batch_field_id: Some(id),
                    }) = field.derive
                    {
                        derive_stack.push(id);
                    }
                }
            }
        }

        // Identify all concrete shapes with errors.
        let mut field_shape_ids_with_errors = modifications.field_shape_id_to_error_ids.ids();
        if let Some(mut current) = field_shape_ids_with_errors.next() {
            'outer: for (concrete_shape_id, shape) in operation_ctx.cached.shapes.concrete.iter().enumerate() {
                if current < shape.field_shape_ids.end {
                    let mut i = 0;
                    while let Some(field_shape_id) = shape.field_shape_ids.get(i) {
                        match field_shape_id.cmp(&current) {
                            std::cmp::Ordering::Less => {
                                i += 1;
                            }
                            std::cmp::Ordering::Equal => {
                                modifications
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

        modifications
    }

    fn deny_field(&mut self, modifier: &'op QueryModifierRecord, error_id: QueryErrorId) {
        if modifier.impacts_root_object {
            self.modifications.root_error_ids.push(error_id);
        }
        for field in modifier.impacted_field_ids.walk(self.operation_ctx) {
            let PartitionField::Data(field) = field else {
                unreachable!()
            };
            self.field_id_to_error_ids.push((field.id, error_id));
        }
    }

    fn skip_field(&mut self, modifier: &'op QueryModifierRecord) {
        for field in modifier.impacted_field_ids.walk(self.operation_ctx) {
            match field {
                PartitionField::Typename(field) => {
                    self.modifications
                        .included_response_typename_fields
                        .set(field.id, false);
                }
                PartitionField::Data(field) => {
                    self.modifications.included_response_data_fields.set(field.id, false);
                }
                // Can never be skipped, it's always necessary if any partition field is needed.
                PartitionField::Lookup(_) => unreachable!(),
            }
        }
    }

    fn push_error(&mut self, error: GraphqlError) -> QueryErrorId {
        let id = QueryErrorId::from(self.modifications.errors.len());
        self.modifications.errors.push(error);
        id
    }
}
