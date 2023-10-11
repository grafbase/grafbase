mod create_many;
mod create_one;
mod delete_many;
mod delete_one;
mod find_many;
mod find_one;
mod query;
mod update_many;
mod update_one;

use super::{context::PostgresContext, Operation};
use crate::{registry::resolvers::ResolvedValue, Error};

pub(super) async fn execute(ctx: PostgresContext<'_>, operation: Operation) -> Result<ResolvedValue, Error> {
    match operation {
        Operation::FindOne => find_one::execute(ctx).await,
        Operation::FindMany => find_many::execute(ctx).await,
        Operation::DeleteOne => delete_one::execute(ctx).await,
        Operation::DeleteMany => delete_many::execute(ctx).await,
        Operation::CreateOne => create_one::execute(ctx).await,
        Operation::CreateMany => create_many::execute(ctx).await,
        Operation::UpdateOne => update_one::execute(ctx).await,
        Operation::UpdateMany => update_many::execute(ctx).await,
    }
}
