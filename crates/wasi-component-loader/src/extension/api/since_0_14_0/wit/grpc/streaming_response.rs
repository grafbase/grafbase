use wasmtime::component::Resource;

use crate::WasiState;

use super::{GrpcStatus, GrpcStreamingResponse, HostGrpcStreamingResponse, MetadataMap};

impl HostGrpcStreamingResponse for WasiState {
    async fn get_metadata(&mut self, self_: Resource<GrpcStreamingResponse>) -> wasmtime::Result<MetadataMap> {
        let (metadata, _, _) = self.get_mut(&self_)?;

        Ok(super::client::tonic_metadata_to_wasi_metadata(metadata))
    }

    async fn get_next_message(
        &mut self,
        self_: Resource<GrpcStreamingResponse>,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, GrpcStatus>> {
        let (_, stream, _) = self.get_mut(&self_)?;

        match stream.message().await {
            Ok(outcome) => Ok(Ok(outcome)),
            Err(err) => Ok(Err(super::client::tonic_status_to_grpc_status(err))),
        }
    }

    async fn drop(&mut self, rep: Resource<GrpcStreamingResponse>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
