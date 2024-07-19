use bindings::{
    component::grafbase::types::{Context, EdgeDefinition, Error, Headers, NodeDefinition, SharedContext},
    exports::component::grafbase::{authorization, gateway_request, subgraph_request},
};

#[allow(warnings)]
mod bindings;

/// This example will only have numeric IDs as arguments. A real life example would utilize
/// something like Serde enums to allow multiple different forms to be passed.
///
/// https://serde.rs/enum-representations.html#untagged
#[derive(serde::Deserialize)]
struct Arguments {
    id: u64,
}

struct Component;

impl gateway_request::Guest for Component {
    /// The context written in this hook will be available in all subsequent hooks throughout
    /// the request lifespan.
    fn on_gateway_request(context: Context, headers: Headers) -> Result<(), Error> {
        if let Some(id) = headers.get("x-current-user-id") {
            context.set("current-user-id", &id);
        }

        Ok(())
    }
}
impl subgraph_request::Guest for Component {
    /// Called just before sending a request to the subgraph. Headers can be modified
    fn on_subgraph_request(
        _context: SharedContext,
        _subgraph_name: String,
        _method: String,
        _url: String,
        _headers: Headers,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl authorization::Guest for Component {
    /// The hook gets called if a field/edge defines an @authorized directive. If the field
    /// has any arguments, they can be sent to this hook from the directive's arguments argument.
    ///
    /// Example from federated-schema.graphql:
    ///
    /// ```ignore
    /// type Query {
    ///   getUser(id: Int!): User @join__field(graph: USERS) @authorized(arguments: "id")
    /// }
    /// ```
    ///
    /// The directive defines the id arguments to be passed to the hook, so the hook arguments is
    /// json `{ id: VALUE }`. If the value is the same as the header x-current-user-id, the edge
    /// query will execute and the data returned back to the user. Otherwise the edge doesn't execute
    /// and an error is added to the response errors.
    fn authorize_edge_pre_execution(
        context: SharedContext,
        _definition: EdgeDefinition,
        arguments: String,
        _metadata: String,
    ) -> Result<(), Error> {
        let current_user_id = parse_user_id(&context)?;

        let args: Arguments = serde_json::from_str(&arguments).unwrap();

        if args.id != current_user_id {
            return Err(Error {
                extensions: vec![(
                    String::from("authorization"),
                    String::from("current user is not authorized to fetch this record"),
                )],
                message: String::from("authorization failed"),
            });
        }

        Ok(())
    }

    /// The hook gets called if a node/type defines an @authorized directive. The directive
    /// can define custom metadata, but this implementation just prevents returning the data
    /// if the x-current-user-id is not 1.
    ///
    /// Example from federated-schema.graphql:
    ///
    /// ```ignore
    /// type Secret @authorized {
    ///    id: Int! @join__field(graph: USERS)
    ///    socialSecurityNumber: String! @join__field(graph: USERS)
    /// }
    /// ```
    ///
    /// Every query that tries to access data from Secret will error and return null, if the
    /// x-current-user-id is not 1.
    fn authorize_node_pre_execution(
        context: SharedContext,
        _definition: NodeDefinition,
        _metadata: String,
    ) -> Result<(), Error> {
        let current_user_id = parse_user_id(&context)?;

        if current_user_id != 1 {
            return Err(Error {
                extensions: vec![(
                    String::from("authorization"),
                    String::from("current user is not authorized to fetch this record"),
                )],
                message: String::from("authorization failed"),
            });
        }

        Ok(())
    }
}

fn parse_user_id(context: &SharedContext) -> Result<u64, Error> {
    match context.get("current-user-id").and_then(|id| id.parse().ok()) {
        Some(id) => Ok(id),
        None => Err(Error {
            extensions: vec![(
                String::from("authorization"),
                String::from("current-user-id is not set"),
            )],
            message: String::from("authorization failed"),
        }),
    }
}

bindings::export!(Component with_types_in bindings);
