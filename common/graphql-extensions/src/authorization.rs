use std::collections::HashSet;
use std::sync::Arc;

use dynaql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo};
use dynaql::graph_entities::ResponseNodeId;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::Registry;
use dynaql::{Operations, ServerError, ServerResult};
use dynaql_value::{indexmap::IndexMap, ConstValue};

use gateway_protocol::ExecutionAuth;
use log::warn;

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

// Use ExecutionAuth and AuthConfig to allow/deny the request
#[async_trait::async_trait]
impl Extension for AuthExtension {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<ResponseNodeId>> {
        lazy_static::lazy_static! {
            static ref EMPTY_INDEX_MAP: IndexMap<dynaql_value::Name, ConstValue> = IndexMap::new();
        }

        // global_ops and groups_from_token are set early on when authorizing
        // the API request. global_ops is based on the top-level auth directive
        // and may be overriden here on the model and field level.
        let execution_auth = ctx
            .data::<ExecutionAuth>()
            .expect("auth must be injected into the context");

        let global_ops = execution_auth.global_ops();
        let groups_from_token = match execution_auth {
            ExecutionAuth::ApiKey => None,
            ExecutionAuth::Token(token) => Some(token.groups_from_token()),
        };
        let model_ops = info
            .auth
            .map(|auth| auth.allowed_ops(groups_from_token))
            .unwrap_or(global_ops); // Fall back to global auth if model auth is not configured

        if let Some(required_op) = info.required_operation {
            if !model_ops.contains(required_op) {
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
                    let input_fields = info
                        .input_values
                        .iter()
                        .find_map(|(name, val)| match name.node.as_str() {
                            INPUT_ARG => match val.as_ref() {
                                Some(ConstValue::Object(obj)) => Some(obj),
                                _ => None,
                            },
                            _ => None,
                        })
                        .unwrap_or(&EMPTY_INDEX_MAP);

                    self.check_input(CheckInputOptions {
                        input_fields,
                        type_name: guess_type_name(&info, required_op),
                        mutation_name: info.name,
                        registry: &ctx.schema_env.registry,
                        required_op,
                        model_ops,
                        global_ops,
                        groups_from_token,
                    })?;
                }
                (MUTATION_TYPE, Operations::DELETE) => {
                    self.check_delete(
                        guess_type_name(&info, required_op),
                        info.name,
                        &ctx.schema_env.registry,
                        model_ops,
                        groups_from_token,
                    )?;
                }
                _ => {}
            }
        // Assume we're resolving a field to be returned by a query or
        // mutation when required_op is None (objects are agnostic to
        // operations) and auth is set.
        } else if let Some(auth) = info.auth {
            let field_ops = auth.allowed_ops(groups_from_token);

            if !field_ops.intersects(Operations::READ) {
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

struct CheckInputOptions<'a> {
    input_fields: &'a IndexMap<dynaql_value::Name, ConstValue>,
    type_name: &'a str,
    mutation_name: &'a str,
    registry: &'a Registry,
    required_op: Operations,
    model_ops: Operations,
    global_ops: Operations,
    groups_from_token: Option<&'a HashSet<String>>,
}

impl AuthExtension {
    pub fn new(trace_id: String) -> Self {
        Self { trace_id }
    }

    // Only allow create/update when the user is authorized to access ALL fields passed as input
    fn check_input(&self, opts: CheckInputOptions<'_>) -> Result<(), ServerError> {
        let type_fields = opts
            .registry
            .types
            .get(opts.type_name)
            .expect("type must exist")
            .fields()
            .expect("type must have fields");

        for (field_name, field_value) in opts.input_fields {
            let field = type_fields.get(field_name.as_str()).expect("field must exist");

            let field_ops = field
                .auth
                .as_ref()
                .map(|auth| auth.allowed_ops(opts.groups_from_token))
                .unwrap_or(opts.model_ops);

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
                        if let Some(ConstValue::Object(obj)) = obj.get(CREATE_FIELD) {
                            self.check_input(CheckInputOptions {
                                input_fields: obj,
                                type_name: target_type,
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
                                opts.global_ops,
                                opts.groups_from_token,
                            )?;
                        }
                    }
                    // Examples: todoCreate(input: { items: [{ create: ... }, { create: ... }] })
                    //           todoUpdate(input: { items: [{ link: "some-id" }, { link: "some-id" }] })
                    ConstValue::List(list) => {
                        for item in list {
                            if let ConstValue::Object(obj) = item {
                                if let Some(ConstValue::Object(obj)) = obj.get(CREATE_FIELD) {
                                    self.check_input(CheckInputOptions {
                                        input_fields: obj,
                                        type_name: target_type,
                                        ..opts
                                    })?;
                                } else if matches!(obj.get(LINK_FIELD), Some(ConstValue::String(_target_id)))
                                    || matches!(obj.get(UNLINK_FIELD), Some(ConstValue::String(_target_id)))
                                {
                                    self.check_link_or_unlink(
                                        target_type,
                                        opts.mutation_name,
                                        opts.registry,
                                        opts.global_ops,
                                        opts.groups_from_token,
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
    fn check_link_or_unlink(
        &self,
        type_name: &str,
        mutation_name: &str,
        registry: &Registry,
        global_ops: Operations,
        groups_from_token: Option<&HashSet<String>>,
    ) -> Result<(), ServerError> {
        self.check_input(CheckInputOptions {
            input_fields: &vec![(dynaql::Name::new("id"), ConstValue::String("ignored".to_string()))]
                .into_iter()
                .collect(),
            type_name,
            mutation_name,
            registry,
            required_op: Operations::GET,
            model_ops: global_ops, // Fall back to global ops because id has inherited model-level auth already
            global_ops,
            groups_from_token,
        })
    }

    // Only allow delete when the user is authorized to delete ALL fields of the type
    // TODO: Check fields of nested types once we support cascading deletes
    fn check_delete(
        &self,
        type_name: &str,
        mutation_name: &str,
        registry: &Registry,
        model_ops: Operations,
        groups_from_token: Option<&HashSet<String>>,
    ) -> Result<(), ServerError> {
        let type_fields = registry
            .types
            .get(type_name)
            .expect("type must exist")
            .fields()
            .expect("type must have fields");

        for (name, field) in type_fields {
            let field_ops = field
                .auth
                .as_ref()
                .map(|auth| auth.allowed_ops(groups_from_token))
                .unwrap_or(model_ops);

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
fn guess_type_name<'a>(info: &'a ResolveInfo<'_>, required_op: Operations) -> &'a str {
    let suffix = match required_op {
        Operations::CREATE => "CreatePayload",
        Operations::UPDATE => "UpdatePayload",
        Operations::DELETE => "DeletePayload",
        _ => panic!("unexpected operation"),
    };

    info.return_type
        .strip_suffix(suffix)
        .expect("must be the expected Payload type")
}
