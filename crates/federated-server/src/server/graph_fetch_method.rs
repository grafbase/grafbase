use super::gateway::{self, GatewayRuntime, GraphDefinition};

use engine_v2::Engine;
use gateway_config::Config;
use graph_ref::GraphRef;
use runtime_local::HooksWasi;
use std::{future::Future, path::PathBuf, sync::Arc};
use tokio::sync::watch;

pub struct FetchGraphFromGraphRef {
    /// The access token for accessing the the API.
    pub access_token: ascii::AsciiString,
    pub graph_ref: GraphRef,
}

pub struct FetchGraphFromSchema {
    /// Static federated graph from a file
    pub federated_sdl: String,
}

pub type GraphFetchMethodSender = watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>;

/// The method of running the gateway.
pub trait GraphFetchMethod: Sync + Sized {
    fn start(
        self,
        config: &Config,
        hot_reload_config_path: Option<PathBuf>,
        sender: GraphFetchMethodSender,
        hooks: HooksWasi,
    ) -> impl Future<Output = crate::Result<()>>;
}

impl GraphFetchMethod for FetchGraphFromGraphRef {
    /// Converts the fetch method into an eventually existing gateway.
    /// The gateway becomes available once the GDN responds with a working graph.
    #[cfg_attr(feature = "lambda", allow(unreachable_code, unused))]
    async fn start(
        self,
        config: &Config,
        _hot_reload_config_path: Option<PathBuf>,
        sender: GraphFetchMethodSender,
        hooks: HooksWasi,
    ) -> crate::Result<()> {
        #[cfg(feature = "lambda")]
        return Err(crate::Error::InternalError(
            "Cannot fetch schema with graph in lambda mode.".to_string(),
        ));

        let config = config.clone();
        tokio::spawn(async move {
            let config = config.clone();
            use super::graph_updater::GraphUpdater;

            GraphUpdater::new(self.graph_ref, self.access_token, sender, config, hooks)?
                .poll()
                .await;

            Ok::<_, crate::Error>(())
        });

        Ok(())
    }
}

impl GraphFetchMethod for FetchGraphFromSchema {
    /// Converts the fetch method into a gateway
    async fn start(
        self,
        config: &Config,
        hot_reload_config_path: Option<PathBuf>,
        sender: GraphFetchMethodSender,
        hooks: HooksWasi,
    ) -> crate::Result<()> {
        let gateway = gateway::generate(
            GraphDefinition::Sdl(self.federated_sdl),
            config,
            hot_reload_config_path,
            hooks,
        )
        .await?;

        sender.send(Some(Arc::new(gateway)))?;

        Ok(())
    }
}
