use wasmtime::component::Resource;

use crate::{AccessLogMessage, AccessLogSender, WasiState, access_log::LogError, extension::wit::HostAccessLog};

impl HostAccessLog for WasiState {
    async fn send(&mut self, data: Vec<u8>) -> wasmtime::Result<Result<(), LogError>> {
        let data = AccessLogMessage::Data(data);

        Ok(self.access_log().send(data).inspect_err(|err| match err {
            LogError::ChannelFull(_) => {
                tracing::error!("access log channel is over capacity");
            }
            LogError::ChannelClosed => {
                tracing::error!("access log channel closed");
            }
        }))
    }

    async fn drop(&mut self, _: Resource<AccessLogSender>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
