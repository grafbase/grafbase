use grafbase_hooks::{
    grafbase_hooks, Context, EdgeDefinition, Error, ErrorResponse, Headers, Hooks, NodeDefinition, SharedContext,
};

struct Component;

#[derive(serde::Deserialize)]
struct Edge {
    value: String,
}

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, context: Context, headers: Headers) -> Result<(), ErrorResponse> {
        if let Some(auth_header) = headers.get("Authorization") {
            context.set("entitlement", &auth_header);
        }

        Ok(())
    }

    fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContext,
        _: EdgeDefinition,
        arguments: String,
        _: String,
    ) -> Result<(), Error> {
        let auth_header = context.get("entitlement");

        if Some(arguments) != auth_header {
            return Err(Error {
                message: String::from("not authorized"),
                extensions: Vec::new(),
            });
        }

        Ok(())
    }

    fn authorize_node_pre_execution(
        &mut self,
        context: SharedContext,
        _: NodeDefinition,
        metadata: String,
    ) -> Result<(), Error> {
        let auth_header = context.get("entitlement");

        if Some(metadata) != auth_header {
            return Err(Error {
                message: String::from("not authorized"),
                extensions: Vec::new(),
            });
        }

        Ok(())
    }

    fn authorize_parent_edge_post_execution(
        &mut self,
        context: SharedContext,
        _: EdgeDefinition,
        parents: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        for parent in parents {
            match serde_json::from_str::<Edge>(&parent) {
                Ok(parent) if Some(&parent.value) == auth_header.as_ref() => {
                    result.push(Ok(()));
                }
                _ => {
                    result.push(Err(Error {
                        message: String::from("not authorized"),
                        extensions: Vec::new(),
                    }));
                }
            }
        }

        result
    }

    fn authorize_edge_node_post_execution(
        &mut self,
        context: SharedContext,
        _: EdgeDefinition,
        nodes: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        for node in nodes {
            match serde_json::from_str::<Edge>(&node) {
                Ok(node) if Some(&node.value) == auth_header.as_ref() => {
                    result.push(Ok(()));
                }
                _ => {
                    result.push(Err(Error {
                        message: String::from("not authorized"),
                        extensions: Vec::new(),
                    }));
                }
            }
        }

        result
    }

    fn authorize_edge_post_execution(
        &mut self,
        context: SharedContext,
        _: EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        for node in edges.into_iter().flat_map(|(_, nodes)| nodes) {
            match serde_json::from_str::<Edge>(&node) {
                Ok(node) if Some(&node.value) == auth_header.as_ref() => {
                    result.push(Ok(()));
                }
                _ => {
                    result.push(Err(Error {
                        message: String::from("not authorized"),
                        extensions: Vec::new(),
                    }));
                }
            }
        }

        result
    }
}

grafbase_hooks::register_hooks!(Component);
