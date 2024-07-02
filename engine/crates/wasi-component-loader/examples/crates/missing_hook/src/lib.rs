#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{EdgeDefinition, ErrorResponse, NodeDefinition, SharedContext},
    exports::component::grafbase::authorization,
};

struct Component;

impl authorization::Guest for Component {
    fn authorize_edge_pre_execution(
        _: SharedContext,
        _: EdgeDefinition,
        _: String,
        _: String,
    ) -> Result<(), ErrorResponse> {
        Ok(())
    }

    fn authorize_node_pre_execution(_: SharedContext, _: NodeDefinition, _: String) -> Result<(), ErrorResponse> {
        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
