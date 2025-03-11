use wasmtime::component::Resource;

use crate::{AccessLogMessage, AccessLogSender, WasiState};

use super::super::wit::access_log;

impl access_log::HostAccessLog for WasiState {
    async fn send(&mut self, data: Vec<u8>) -> wasmtime::Result<Result<(), access_log::LogError>> {
        let data = AccessLogMessage::Data(data);

        Ok(self.access_log().send(data).inspect_err(|err| match err {
            access_log::LogError::ChannelFull(_) => {
                tracing::error!("access log channel is over capacity");
            }
            access_log::LogError::ChannelClosed => {
                tracing::error!("access log channel closed");
            }
        }))
    }

    async fn drop(&mut self, _: Resource<AccessLogSender>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
