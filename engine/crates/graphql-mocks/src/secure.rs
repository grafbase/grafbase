use async_graphql::{EmptyMutation, EmptySubscription, Object, TypeDirective, Union};

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

#[TypeDirective(
    name = "authorized",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authorized(rule: String, arguments: Option<String>) {}

#[derive(Default)]
pub struct Query;

pub struct Check;

#[Object]
impl Check {
    async fn anonymous(&self) -> &'static str {
        "Hello anonymous!"
    }

    #[graphql(
        directive = authenticated::apply()
    )]
    async fn faillible_must_be_authenticated(&self) -> Option<&'static str> {
        Some("You are authenticated")
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

    #[graphql(
        directive = authorized::apply("x-grafbase-client-name-header-is-defined".into(), None)
    )]
    async fn grafbase_client_is_defined(&self) -> &'static str {
        "You have properly set the x-grafbase-client-name header"
    }

    #[graphql(
        directive = authorized::apply("sensitive-input-id".into(), Some("id".into()))
    )]
    async fn sensitive_id(&self, id: i64) -> &'static str {
        let _ = id;
        "You have access to the sensistive data"
    }
}

pub struct User;

#[Object]
impl User {
    async fn name(&self) -> &str {
        "rusty"
    }
}

#[derive(Union)]
enum Entity {
    User(User),
    Check(Check),
}

#[Object]
impl Query {
    async fn check(&self) -> Check {
        Check
    }

    async fn nullable_check(&self) -> Option<Check> {
        Some(Check)
    }

    async fn entity(&self, check: bool) -> Entity {
        if check {
            Entity::Check(Check)
        } else {
            Entity::User(User)
        }
    }

    async fn entities_nullable(&self, check: bool) -> Vec<Option<Entity>> {
        let mut out = vec![Some(Entity::User(User))];
        if check {
            out.push(Some(Entity::Check(Check)));
        }
        out
    }

    async fn entities(&self, check: bool) -> Vec<Entity> {
        let mut out = vec![Entity::User(User)];
        if check {
            out.push(Entity::Check(Check));
        }
        out
    }
}
