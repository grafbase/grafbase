use wasmtime::component::Resource;

use crate::{AccessLogMessage, AccessLogSender, WasiState, extension::api::wit as latest};

use super::super::wit::grafbase::sdk::types;

impl types::HostAccessLog for WasiState {
    async fn send(&mut self, data: Vec<u8>) -> wasmtime::Result<Result<(), types::LogError>> {
        let data = AccessLogMessage::Data(data);

        let result = self
            .access_log()
            .send(data)
            .inspect_err(|err| match err {
                latest::access_log::LogError::ChannelFull(_) => {
                    tracing::error!("access log channel is over capacity");
                }
                latest::access_log::LogError::ChannelClosed => {
                    tracing::error!("access log channel closed");
                }
            })
            .map_err(Into::into);

        Ok(result)
    }

    async fn drop(&mut self, _: Resource<AccessLogSender>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
