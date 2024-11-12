#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{EdgeDefinition, Error, NodeDefinition, SharedContext},
    exports::component::grafbase::authorization,
};

struct Component;

impl authorization::Guest for Component {
    fn authorize_edge_pre_execution(_: SharedContext, _: EdgeDefinition, _: String, _: String) -> Result<(), Error> {
        Ok(())
    }

    fn authorize_node_pre_execution(_: SharedContext, _: NodeDefinition, _: String) -> Result<(), Error> {
        Ok(())
    }

    fn authorize_parent_edge_post_execution(
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }

    fn authorize_edge_node_post_execution(
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }

    fn authorize_edge_post_execution(
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<(String, Vec<String>)>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }
}

bindings::export!(Component with_types_in bindings);
