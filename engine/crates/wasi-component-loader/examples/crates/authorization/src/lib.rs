#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, EdgeDefinition, Error, Headers, NodeDefinition, SharedContext},
    exports::component::grafbase::{authorization, gateway_request},
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(context: Context, headers: Headers) -> Result<(), Error> {
        if let Ok(Some(auth_header)) = headers.get("Authorization") {
            context.set("entitlement", &auth_header);
        }

        Ok(())
    }
}

impl authorization::Guest for Component {
    fn authorize_edge_pre_execution(
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

    fn authorize_node_pre_execution(context: SharedContext, _: NodeDefinition, metadata: String) -> Result<(), Error> {
        let auth_header = context.get("entitlement");

        if Some(metadata) != auth_header {
            return Err(Error {
                message: String::from("not authorized"),
                extensions: Vec::new(),
            });
        }

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
