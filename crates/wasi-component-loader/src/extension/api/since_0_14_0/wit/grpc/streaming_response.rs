use wasmtime::component::Resource;

use crate::WasiState;

use super::{GrpcStreamingResponse, HostGrpcStreamingResponse, MetadataMap};

impl HostGrpcStreamingResponse for WasiState {
    async fn get_metadata(&mut self, _self_: Resource<GrpcStreamingResponse>) -> wasmtime::Result<MetadataMap> {
        todo!()
    }

    async fn get_next_message(
        &mut self,
        _self_: Resource<GrpcStreamingResponse>,
    ) -> wasmtime::Result<Result<Vec<u8>, String>> {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<GrpcStreamingResponse>) -> wasmtime::Result<()> {
        Ok(())
    }
}
