use async_graphql::{EmptyMutation, EmptySubscription, Object, SimpleObject, TypeDirective, Union};

#[derive(Default)]
pub struct SecureSchema;

impl crate::Subgraph for SecureSchema {
    fn name(&self) -> String {
        "secure".to_string()
    }

    async fn start(self) -> crate::MockGraphQlServer {
        let schema = async_graphql::Schema::<Query, EmptyMutation, EmptySubscription>::default();
        crate::MockGraphQlServer::new(schema).await
    }
}

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
fn authorized(
    arguments: Option<String>,
    fields: Option<String>,
    node: Option<String>,
    metadata: Option<Vec<Vec<String>>>,
) {
}

#[derive(Default)]
pub struct Query;

#[derive(Default, SimpleObject)]
#[graphql(
    directive = authorized::apply(None,  None,None, None)
)]
pub struct AuthorizedNode {
    pub id: String,
}

#[derive(Default, SimpleObject)]
#[graphql(
    directive = authorized::apply(None, None,  None,Some(vec![vec!["admin".into()]]))
)]
pub struct AuthorizedWithMetdataNode {
    pub id: String,
}

struct Node;

#[Object]
impl Node {
    async fn authorized(&self) -> AuthorizedNode {
        AuthorizedNode { id: "1a".to_string() }
    }

    async fn authorized_with_metadata(&self) -> AuthorizedWithMetdataNode {
        AuthorizedWithMetdataNode { id: "2a".to_string() }
    }

    async fn nullable_authorized(&self) -> Option<AuthorizedNode> {
        Some(AuthorizedNode { id: "1b".to_string() })
    }

    async fn nullable_authorized_with_metadata(&self) -> Option<AuthorizedWithMetdataNode> {
        Some(AuthorizedWithMetdataNode { id: "2b".to_string() })
    }

    async fn always_happy(&self) -> &'static str {
        "A dog"
    }
}

struct AuthorizedEdgeWithFields {
    id: String,
}

#[Object]
impl AuthorizedEdgeWithFields {
    async fn id(&self) -> &str {
        &self.id
    }

    #[graphql(
        directive = authorized::apply(None, Some("id".to_string()),  None,None)
    )]
    async fn with_id(&self) -> &'static str {
        "You have access"
    }

    #[graphql(
        directive = authorized::apply(None, Some("id".to_string()),  None,Some(vec![vec!["rusty".to_string()]]))
    )]
    async fn with_id_and_metadata(&self) -> &'static str {
        "You have access"
    }
}

struct AuthorizedEdgeWithNode {
    ids: Vec<String>,
}

#[derive(SimpleObject)]
struct DummyNode {
    id: String,
}

#[Object]
impl AuthorizedEdgeWithNode {
    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn with_id(&self) -> DummyNode {
        DummyNode {
            id: self.ids.first().cloned().unwrap_or_default(),
        }
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), Some(vec![vec!["rusty".to_string()]]))
    )]
    async fn with_id_and_metadata(&self) -> DummyNode {
        DummyNode {
            id: self.ids.first().cloned().unwrap_or_default(),
        }
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn nullable_with_id(&self) -> Option<DummyNode> {
        Some(DummyNode {
            id: self.ids.first().cloned().unwrap_or_default(),
        })
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn list_with_id(&self) -> Vec<DummyNode> {
        self.ids.clone().into_iter().map(|id| DummyNode { id }).collect()
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn list_nullable_with_id(&self) -> Vec<Option<DummyNode>> {
        self.ids.clone().into_iter().map(|id| Some(DummyNode { id })).collect()
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn list_list_with_id(&self) -> Vec<Vec<DummyNode>> {
        self.ids.clone().into_iter().map(|id| vec![DummyNode { id }]).collect()
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn list_nullable_list_with_id(&self) -> Vec<Option<Vec<DummyNode>>> {
        self.ids
            .clone()
            .into_iter()
            .map(|id| Some(vec![DummyNode { id }]))
            .collect()
    }

    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), None)
    )]
    async fn list_list_nullable_with_id(&self) -> Vec<Vec<Option<DummyNode>>> {
        self.ids
            .clone()
            .into_iter()
            .map(|id| vec![Some(DummyNode { id })])
            .collect()
    }
}

pub struct Check;

#[Object]
impl Check {
    async fn authorized_edge_with_fields(&self, prefix: String, id: String) -> AuthorizedEdgeWithFields {
        AuthorizedEdgeWithFields {
            id: format!("{prefix}{id}"),
        }
    }

    async fn authorized_edge_with_node(&self, ids: Vec<String>) -> AuthorizedEdgeWithNode {
        AuthorizedEdgeWithNode { ids }
    }

    // -- @authenticated -- //
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

    // -- @requiresScopes -- //
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

    // -- @authorized -- //
    #[graphql(
        directive = authorized::apply(None, None, None, None)
    )]
    async fn authorized(&self) -> &'static str {
        "You have access"
    }

    #[graphql(
        directive = authorized::apply(None, None, None, Some(vec![vec!["admin".into()]]))
    )]
    async fn authorized_with_metadata(&self) -> &'static str {
        "You have access"
    }

    #[graphql(
        directive = authorized::apply(Some("id".into()), None, None, None)
    )]
    async fn authorized_with_id(&self, id: i64) -> &'static str {
        let _ = id;
        "You have access to the sensistive data"
    }
}

pub struct OtherCheck;

#[Object]
impl OtherCheck {
    #[graphql(
        directive = authorized::apply(None, None, None, None)
    )]
    async fn authorized(&self) -> &'static str {
        "Other: You have access"
    }

    #[graphql(
        directive = authorized::apply(None, None, None, Some(vec![vec!["admin".into()]]))
    )]
    async fn authorized_with_metadata(&self) -> &'static str {
        "You have access"
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
    async fn node(&self) -> Node {
        Node
    }

    async fn nullable_node(&self) -> Option<Node> {
        Some(Node)
    }

    async fn other_check(&self) -> OtherCheck {
        OtherCheck
    }

    async fn nullable_other_check(&self) -> Option<OtherCheck> {
        Some(OtherCheck)
    }

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
