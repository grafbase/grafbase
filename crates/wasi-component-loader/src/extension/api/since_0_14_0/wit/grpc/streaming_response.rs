use wasmtime::component::Resource;

use crate::InstanceState;

use super::{GrpcStatus, GrpcStreamingResponse, HostGrpcStreamingResponse, MetadataMap};

impl HostGrpcStreamingResponse for InstanceState {
    async fn get_metadata(&mut self, self_: Resource<GrpcStreamingResponse>) -> wasmtime::Result<MetadataMap> {
        let (metadata, _, _) = self.resources.get_mut(&self_)?;

        Ok(super::client::tonic_metadata_to_wasi_metadata(metadata))
    }

    async fn get_next_message(
        &mut self,
        self_: Resource<GrpcStreamingResponse>,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, GrpcStatus>> {
        let (_, stream, _) = self.resources.get_mut(&self_)?;

        match stream.message().await {
            Ok(outcome) => Ok(Ok(outcome)),
            Err(err) => Ok(Err(super::client::tonic_status_to_grpc_status(err))),
        }
    }

    async fn drop(&mut self, rep: Resource<GrpcStreamingResponse>) -> wasmtime::Result<()> {
        self.resources.delete(rep)?;
        Ok(())
    }
}
