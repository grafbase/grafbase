use grafbase_hooks::{
    grafbase_hooks, Context, EdgeNodePostExecutionArguments, EdgePostExecutionArguments, EdgePreExecutionArguments,
    Error, ErrorResponse, Headers, Hooks, NodePreExecutionArguments, ParentEdgePostExecutionArguments, SharedContext,
};

struct Component;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct Parent {
    id: u64,
}

#[derive(serde::Deserialize)]
struct Edge<'a> {
    value: &'a str,
}

#[derive(serde::Deserialize)]
struct Metadata<'a> {
    role: &'a str,
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
        arguments: EdgePreExecutionArguments,
    ) -> Result<(), Error> {
        let auth_header = context.get("entitlement");
        let argument = arguments.arguments::<Edge>().unwrap();

        if Some(argument.value) != auth_header.as_deref() {
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
        arguments: NodePreExecutionArguments,
    ) -> Result<(), Error> {
        let auth_header = context.get("entitlement");
        let metadata: Metadata = arguments.metadata().unwrap();

        if Some(metadata.role) != auth_header.as_deref() {
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
        arguments: ParentEdgePostExecutionArguments,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        let parents: Vec<Edge> = match arguments.parents() {
            Ok(parents) => parents,
            Err(_) => {
                return vec![Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                })]
            }
        };

        for parent in parents {
            if Some(parent.value) == auth_header.as_deref() {
                result.push(Ok(()));
            } else {
                result.push(Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                }));
            }
        }

        result
    }

    fn authorize_edge_node_post_execution(
        &mut self,
        context: SharedContext,
        arguments: EdgeNodePostExecutionArguments,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        let nodes: Vec<Edge> = match arguments.nodes() {
            Ok(nodes) => nodes,
            Err(_) => {
                return vec![Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                })]
            }
        };

        for node in nodes {
            if Some(node.value) == auth_header.as_deref() {
                result.push(Ok(()));
            } else {
                result.push(Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                }));
            }
        }

        result
    }

    fn authorize_edge_post_execution(
        &mut self,
        context: SharedContext,
        arguments: EdgePostExecutionArguments,
    ) -> Vec<Result<(), Error>> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::new();

        let edges: Vec<(Parent, Vec<Edge>)> = match arguments.edges() {
            Ok(edges) => edges,
            Err(_) => {
                return vec![Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                })]
            }
        };

        for node in edges.into_iter().flat_map(|(_, nodes)| nodes) {
            if Some(node.value) == auth_header.as_deref() {
                result.push(Ok(()));
            } else {
                result.push(Err(Error {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                }));
            }
        }

        result
    }
}

grafbase_hooks::register_hooks!(Component);
