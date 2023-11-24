//! ----------------------------------------------------------------------------
//! The Auth is going to be injected inside engine instead of just living as an
//! Extension as it's adding complexity without much gain.
//! ----------------------------------------------------------------------------
use std::sync::Arc;

use common_types::auth::{ExecutionAuth, Operations};
use engine::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo},
    graph_entities::ResponseNodeId,
    registry::{relations::MetaRelation, ModelName, NamedType, Registry, TypeReference},
    AuthConfig, ServerError, ServerResult,
};
use engine_value::ConstValue;
use log::{trace, warn};

const INPUT_ARG: &str = "input";
const CREATE_FIELD: &str = "create";
const LINK_FIELD: &str = "link";
const UNLINK_FIELD: &str = "unlink";
const MUTATION_TYPE: &str = "Mutation";

/// Authorization extension
///
/// This extension will check that the user is authorized to execute the GraphQL operation.
pub struct AuthExtension {
    trace_id: String,
}

impl ExtensionFactory for AuthExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(AuthExtension::new(self.trace_id.clone()))
    }
}

// Use ExecutionAuth from ctx and AuthConfig from ResolveInfo (global) or MetaField  to allow/deny the request.
#[async_trait::async_trait]
impl Extension for AuthExtension {
    /// Called at prepare request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: engine::Request,
        next: engine::extensions::NextPrepareRequest<'_>,
    ) -> ServerResult<engine::Request> {
        let auth_context = ctx
            .data::<ExecutionAuth>()
            .expect("auth must be injected into the context");
        let request = if auth_context.is_introspection_allowed() {
            request
        } else {
            request.disable_introspection()
        };
        next.run(ctx, request).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<ResponseNodeId>> {
        if info.parent_type.starts_with("__") || info.parent_type.starts_with("[__") || info.name.starts_with("__") {
            return next.run(ctx, info).await;
        }

        let execution_auth = ctx
            .data::<ExecutionAuth>()
            .expect("auth must be injected into the context");
        let auth_fn = |auth: Option<&AuthConfig>, default_ops: Operations| {
            auth.map(|auth| match execution_auth {
                ExecutionAuth::ApiKey => common_types::auth::API_KEY_OPS,
                ExecutionAuth::Token(token) => auth.private_public_and_group_based_ops(token.groups_from_token()),
                ExecutionAuth::Public { .. } => auth.allowed_public_ops,
            })
            .unwrap_or(default_ops)
        };
        // Get the allowed operation from the parsed schema.
        let model_allowed_ops = auth_fn(info.auth, execution_auth.global_ops()); // Fall back to global auth if model auth is not configured
        trace!(
            self.trace_id,
            "Resolving {parent_type}.{name}, auth: {auth:?} allowed ops as {model_allowed_ops:?}, required {required_op:?}",
            parent_type = info.parent_type,
            name = info.name,
            auth = info.auth,
            required_op = info.required_operation
        );

        // Required operation is inferred from the current request.
        if let Some(required_op) = info.required_operation {
            if !model_allowed_ops.contains(required_op) {
                let msg = format!(
                    "Unauthorized to access {parent_type}.{name} (missing {required_op} operation)",
                    parent_type = info.parent_type,
                    name = info.name
                );
                warn!(self.trace_id, "{msg} auth={auth:?}", auth = info.auth);
                return Err(ServerError::new(msg, None));
            }

            match (info.parent_type, required_op) {
                (MUTATION_TYPE, Operations::CREATE | Operations::UPDATE) => {
                    let input = info
                        .input_values
                        .iter()
                        .find_map(|(name, val)| val.as_ref().filter(|_| name.as_str() == INPUT_ARG))
                        .unwrap_or(&ConstValue::Null);
                    let global_allowed_ops = execution_auth.global_ops();

                    if let Some(type_name) = guess_batch_operation_type_name(&info, required_op) {
                        let inputs = match input {
                            obj @ ConstValue::Object(_) => vec![obj],
                            ConstValue::List(objs) => objs.iter().collect(),
                            _ => vec![],
                        };
                        for input in inputs {
                            self.check_input(CheckInputOptions {
                                input: match input {
                                    ConstValue::Object(args) => args.get(INPUT_ARG),
                                    _ => None,
                                }
                                .unwrap_or(&ConstValue::Null),
                                type_name: type_name.clone(),
                                mutation_name: info.name,
                                registry: &ctx.schema_env.registry,
                                required_op,
                                model_allowed_ops,
                                global_allowed_ops,
                                auth_fn: &auth_fn,
                            })?;
                        }
                    } else {
                        self.check_input(CheckInputOptions {
                            input,
                            type_name: guess_type_name(&info, required_op),
                            mutation_name: info.name,
                            registry: &ctx.schema_env.registry,
                            required_op,
                            model_allowed_ops,
                            global_allowed_ops,
                            auth_fn: &auth_fn,
                        })?;
                    }
                }
                (MUTATION_TYPE, Operations::DELETE) => {
                    self.check_delete(
                        guess_batch_operation_type_name(&info, required_op)
                            .unwrap_or_else(|| guess_type_name(&info, required_op)),
                        info.name,
                        &ctx.schema_env.registry,
                        model_allowed_ops,
                        &auth_fn,
                    )?;
                }
                _ => {}
            }
        // Assume we're resolving a field to be returned by a query or
        // mutation when required_op is None (objects are agnostic to
        // operations) and auth is set.
        } else if let Some(auth) = info.auth {
            let field_ops = auth_fn(Some(auth), Operations::empty());
            trace!(self.trace_id, "Field level auth. field_ops:{field_ops}");
            if !field_ops.intersects(Operations::READ) {
                // FIXME: Field rule should not have operations configurable.
                let msg = format!(
                    "Unauthorized to access {type_name}.{field_name}",
                    type_name = info.parent_type,
                    field_name = info.name,
                );
                warn!(self.trace_id, "{msg} field_ops={field_ops:?}");
                return Err(ServerError::new(msg, None));
            }
        }

        next.run(ctx, info).await
    }
}

struct CheckInputOptions<'a, F: Fn(Option<&AuthConfig>, Operations) -> Operations> {
    input: &'a ConstValue,
    type_name: NamedType<'a>,
    mutation_name: &'a str,
    registry: &'a Registry,
    required_op: Operations,
    model_allowed_ops: Operations,
    global_allowed_ops: Operations,
    auth_fn: &'a F,
}

impl AuthExtension {
    pub fn new(trace_id: String) -> Self {
        Self { trace_id }
    }

    // Only allow create/update when the user is authorized to access ALL fields passed as input
    fn check_input<F: Fn(Option<&AuthConfig>, Operations) -> Operations>(
        &self,
        opts: CheckInputOptions<'_, F>,
    ) -> Result<(), ServerError> {
        let ConstValue::Object(input_fields) = opts.input else {
            return Ok(());
        };

        log::info!(self.trace_id, "{:?}", opts.mutation_name);
        log::info!(self.trace_id, "{:?}", opts.type_name);
        log::info!(self.trace_id, "{:?}", input_fields);
        let type_fields = opts
            .registry
            .lookup(&opts.type_name)
            .expect("type must exist")
            .fields()
            .expect("type must have fields");

        for (field_name, field_value) in input_fields {
            let field = type_fields.get(field_name.as_str()).expect("field must exist");

            let field_ops = (opts.auth_fn)(field.auth.as_ref(), opts.model_allowed_ops);

            log::trace!(self.trace_id, "check_input.{field_name} ${field_ops}");

            if !field_ops.contains(opts.required_op) {
                let msg = format!(
                    "Unauthorized to access {MUTATION_TYPE}.{mutation_name} (missing {required_op} operation on {type_name}.{field_name})",
                    mutation_name = opts.mutation_name,
                    required_op = opts.required_op,
                    type_name = opts.type_name,
                );

                warn!(self.trace_id, "{msg} auth={auth:?}", auth = field.auth);
                return Err(ServerError::new(msg, None));
            }

            // Handle relations via create, link, and unlink
            if let Some(MetaRelation { relation, .. }) = &field.relation {
                let target_type = &relation.1;

                match field_value {
                    // Example: todoCreate(input: { items: { create: ... } })
                    ConstValue::Object(obj) => {
                        if let Some(input) = obj.get(CREATE_FIELD) {
                            self.check_input(CheckInputOptions {
                                input,
                                type_name: target_type.as_str().into(),
                                ..opts
                            })?;
                        }
                        // Examples: todoCreate(input: { items: { link: "some-id" } })
                        //           todoUpdate(input: { items: { unlink: "some-id" } })
                        else if matches!(obj.get(LINK_FIELD), Some(ConstValue::String(_target_id)))
                            || matches!(obj.get(UNLINK_FIELD), Some(ConstValue::String(_target_id)))
                        {
                            self.check_link_or_unlink(
                                target_type,
                                opts.mutation_name,
                                opts.registry,
                                opts.global_allowed_ops,
                                opts.auth_fn,
                            )?;
                        }
                    }
                    // Examples: todoCreate(input: { items: [{ create: ... }, { create: ... }] })
                    //           todoUpdate(input: { items: [{ link: "some-id" }, { link: "some-id" }] })
                    ConstValue::List(list) => {
                        for item in list {
                            if let ConstValue::Object(obj) = item {
                                if let Some(input) = obj.get(CREATE_FIELD) {
                                    self.check_input(CheckInputOptions {
                                        input,
                                        type_name: target_type.named_type(),
                                        ..opts
                                    })?;
                                } else if matches!(obj.get(LINK_FIELD), Some(ConstValue::String(_target_id)))
                                    || matches!(obj.get(UNLINK_FIELD), Some(ConstValue::String(_target_id)))
                                {
                                    self.check_link_or_unlink(
                                        target_type,
                                        opts.mutation_name,
                                        opts.registry,
                                        opts.global_allowed_ops,
                                        opts.auth_fn,
                                    )?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    // Only allow (un)link when the user can read the target type's id
    fn check_link_or_unlink<F: Fn(Option<&AuthConfig>, Operations) -> Operations>(
        &self,
        type_name: &ModelName,
        mutation_name: &str,
        registry: &Registry,
        global_ops: Operations,
        auth_fn: &F,
    ) -> Result<(), ServerError> {
        self.check_input(CheckInputOptions {
            input: &ConstValue::Object(
                vec![(engine::Name::new("id"), ConstValue::String("ignored".to_string()))]
                    .into_iter()
                    .collect(),
            ),
            type_name: type_name.named_type(),
            mutation_name,
            registry,
            required_op: Operations::GET,
            model_allowed_ops: global_ops, // Fall back to global ops because id has inherited model-level auth already
            global_allowed_ops: global_ops,
            auth_fn,
        })
    }

    // Only allow delete when the user is authorized to delete ALL fields of the type
    // TODO: Check fields of nested types once we support cascading deletes
    fn check_delete<F: Fn(Option<&AuthConfig>, Operations) -> Operations>(
        &self,
        type_name: NamedType<'_>,
        mutation_name: &str,
        registry: &Registry,
        model_ops: Operations,
        auth_fn: &F,
    ) -> Result<(), ServerError> {
        let type_fields = registry
            .lookup(&type_name)
            .expect("type must exist")
            .fields()
            .expect("type must have fields");

        for (name, field) in type_fields {
            let field_ops = auth_fn(field.auth.as_ref(), model_ops);

            if !field_ops.contains(Operations::DELETE) {
                let msg = format!(
                    "Unauthorized to access {MUTATION_TYPE}.{mutation_name} (missing delete operation on {type_name}.{name})"
                );
                warn!(self.trace_id, "{msg} auth={auth:?}", auth = field.auth);
                return Err(ServerError::new(msg, None));
            }
        }

        Ok(())
    }
}

// HACK to get underlying type, which is not available in ResolveInfo
#[allow(clippy::panic)]
fn guess_type_name(info: &ResolveInfo<'_>, required_op: Operations) -> NamedType<'static> {
    let suffix = match required_op {
        Operations::CREATE => "CreatePayload",
        Operations::UPDATE => "UpdatePayload",
        Operations::DELETE => "DeletePayload",
        _ => panic!("unexpected operation"),
    };

    info.return_type
        .named_type()
        .as_str()
        .strip_suffix(suffix)
        .expect("must be the expected Payload type")
        .to_owned()
        .into()
}

// HACK: we're deprecating the database, so continuing the previous hack...
#[allow(clippy::panic)]
fn guess_batch_operation_type_name(info: &ResolveInfo<'_>, required_op: Operations) -> Option<NamedType<'static>> {
    let suffix = match required_op {
        Operations::CREATE => "CreateManyPayload",
        Operations::UPDATE => "UpdateManyPayload",
        Operations::DELETE => "DeleteManyPayload",
        _ => panic!("unexpected operation"),
    };
    info.return_type
        .named_type()
        .as_str()
        .strip_suffix(suffix)
        .map(|name| name.to_owned().into())
}
