//! Auth module belonging to dynaql.
//!
//! It's where every auth rules will belong now, it's no more a Layer, it's a part of dynaql.
//! It's no more part of the Runtime with an extension but part of the definition of the schema.
//!
//!
//! ----------------------------------------------------------------------------
//! /!\ Right now we have duplicated the AuthRules here and in the
//! `common/graphql-extension/authorization`. If you do a change take it into account.
//! ----------------------------------------------------------------------------

use crate::registry::MetaType;
use crate::{Context, ServerError, ServerResult};
use dynaql_parser::Positioned;
use grafbase::auth::{ExecutionAuth, Operations};
use logworker::warn;
use query_planning::logical_plan::LogicalPlan;

pub struct AuthContext<'a> {
    exec: Option<&'a ExecutionAuth>,
    // TODO: We should really remove that and put a proper tracing integration.
    trace_id: String,
}

impl<'a> AuthContext<'a> {
    /// You can get the associated [`AuthContext`] from anywhere you have access to the
    /// [`Context`].
    pub fn new(ctx: &'_ Context<'a>) -> Self {
        // global_ops and groups_from_token are set early on when authorizing
        // the API request. global_ops is based on the top-level auth directive
        // and may be overriden here on the model and field level.
        let exec = ctx.data::<ExecutionAuth>().ok();
        let trace_id = ctx.trace_id();

        Self { exec, trace_id }
    }

    /// Check done while creating the [`LogicalQuery`].
    pub fn check_resolving_logical_query(
        &self,
        ctx: &'_ Context<'a>,
        root: &MetaType,
    ) -> ServerResult<()> {
        if let Some(exec) = self.exec {
            let field_name = ctx.item.node.name.node.as_str();
            let meta_field = root.field_by_name(field_name);
            let auth = meta_field.and_then(|f| f.auth.as_ref());
            let required_operation = meta_field.and_then(|f| f.required_operation.as_ref());
            let parent_type = root.name();

            let global_ops = exec.global_ops();
            let groups_from_token = match exec {
                ExecutionAuth::ApiKey => None,
                ExecutionAuth::Token(token) => Some(token.groups_from_token()),
            };

            let model_ops = auth
                .map(|auth| auth.allowed_ops(groups_from_token))
                .unwrap_or(global_ops); // Fall back to global auth if model auth is not configured
                                        //
            if let Some(required_op) = required_operation {
                if !model_ops.contains(*required_op) {
                    let msg = format!(
                    "Unauthorized to access {parent_type}.{field_name} (missing {required_op} operation)"
                );
                    warn!(self.trace_id, "{msg} auth={auth:?}", auth = auth);
                    return Err(ServerError::new(msg, None));
                }

                // There is no check for the input variables as transactions are not supported with
                // the query planning YET.
                // And moreover I do believe we don't want the check to happen the way it is
                // implemented right now: we might want to split the check when we are translating
                // those input into an associated plan.

                // Assume we're resolving a field to be returned by a query or
                // mutation when `required_op` is None (objects are agnostic to
                // operations) and auth is set.
            } else if let Some(auth) = auth {
                let field_ops = auth.allowed_ops(groups_from_token);

                if !field_ops.intersects(Operations::READ) {
                    let msg = format!(
                        "Unauthorized to access {parent_type}.{field_name}",
                    );
                    warn!(self.trace_id, "{msg} field_ops={field_ops:?}");
                    return Err(ServerError::new(msg, None));
                }
            }
        }

        Ok(())
    }

    /// In Auth rules it's possible we sometime want to change or impact the fetching layer, but we
    /// don't want to directly interfere with this layer. For instance we might want a user to only
    /// fetch the items he has the right to see.
    ///
    /// For those case, while creating the [`LogicalQuery`], every plan is passed to this function
    /// to modify and to add the associated LogicalPlan when needed without interefering with the
    /// data layer.
    ///
    /// For instance, we might want to add a `Filter` plan when we want to fetch only items related
    /// to an User, but at the end it'll be the [`QueryPlanner`] itself which will decide how to
    /// optimize the query plan if the associated datasource is able to merge this [`Filter`] plan
    /// inside the plan fetching the actual entities.
    pub fn auth_middleware_logical_plan(
        &self,
        _ctx: &'_ Context<'a>,
        _root: &MetaType,
        lp: Positioned<LogicalPlan>,
    ) -> ServerResult<LogicalPlan> {
        // Not implemented yet.
        Ok(lp.node)
    }
}
