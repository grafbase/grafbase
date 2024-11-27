use grafbase_hooks::{grafbase_hooks, EdgeDefinition, Error, Hooks, NodeDefinition, SharedContext};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn authorize_edge_pre_execution(
        &mut self,
        _: SharedContext,
        _: EdgeDefinition,
        _: String,
        _: String,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn authorize_node_pre_execution(&mut self, _: SharedContext, _: NodeDefinition, _: String) -> Result<(), Error> {
        Ok(())
    }

    fn authorize_parent_edge_post_execution(
        &mut self,
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }

    fn authorize_edge_node_post_execution(
        &mut self,
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<String>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }

    fn authorize_edge_post_execution(
        &mut self,
        _: SharedContext,
        _: EdgeDefinition,
        _: Vec<(String, Vec<String>)>,
        _: String,
    ) -> Vec<Result<(), Error>> {
        vec![Ok(())]
    }
}

grafbase_hooks::register_hooks!(Component);
