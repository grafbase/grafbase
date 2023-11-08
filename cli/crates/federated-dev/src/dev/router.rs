use super::bus::GraphReceiver;

pub(crate) struct Router {
    bus: GraphReceiver,
}

/// TODO: Benjamin and Graeme!
///
/// Add the router here, route the queries from the Axum to it.
/// When the handler receives a new graph, restart the router.
///
/// P.s. there might be a need for a m-m-mutex...
impl Router {
    pub fn new(bus: GraphReceiver) -> Self {
        Self { bus }
    }

    pub async fn handler(mut self) -> Result<(), crate::Error> {
        while let Some(graph) = self.bus.recv().await {
            // remvove this when adding the actual router
            let rendered = graphql_composition::render_sdl(&graph).expect("must render");
            println!("******** GRAPH ********");
            print!("{rendered}");
        }

        Ok(())
    }
}
