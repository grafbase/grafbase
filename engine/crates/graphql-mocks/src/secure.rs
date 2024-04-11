use async_graphql::{EmptyMutation, EmptySubscription, Object, TypeDirective};

/// A schema that only uses String types.
///
/// This is used to make sure that we're not pruning built in scalars that aren't used
pub type SecureSchema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

#[TypeDirective(
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authenticated() {}

#[TypeDirective(
    name = "requiresScopes",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn requires_scopes(scopes: Vec<Vec<String>>) {}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn anonymous(&self) -> &'static str {
        "Hello anonymous!"
    }

    #[graphql(
        directive = authenticated::apply()
    )]
    async fn must_be_authenticated(&self) -> &'static str {
        "You are authenticated"
    }

    #[graphql(
        directive = requires_scopes::apply(vec![vec!["read".into()]])
    )]
    async fn must_have_read_scope(&self) -> &'static str {
        "You have read scope"
    }

    #[graphql(
        directive = requires_scopes::apply(vec![vec!["write".into()]])
    )]
    async fn must_have_write_scope(&self) -> &'static str {
        "You have write scope"
    }

    #[graphql(
        directive = requires_scopes::apply(vec![vec!["read".into()], vec!["write".into()]])
    )]
    async fn must_have_read_or_write_scope(&self) -> &'static str {
        "You have either read or write scope"
    }

    #[graphql(
        directive = requires_scopes::apply(vec![vec!["read".into(), "write".into()]])
    )]
    async fn must_have_read_and_write_scope(&self) -> &'static str {
        "You have read and write scopes"
    }
}
