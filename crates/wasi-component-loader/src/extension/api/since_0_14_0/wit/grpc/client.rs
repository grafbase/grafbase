use super::{
    GrpcClient, GrpcClientConfiguration, GrpcStatus, GrpcStreamingResponse, GrpcUnaryResponse, HostGrpcClient,
    MetadataMap,
};
use crate::{
    WasiState,
    tonic::{
        self, GrpcMethod,
        metadata::{MetadataKey, MetadataValue},
    },
};
use bytes::BufMut as _;
use dashmap::Entry;
use wasmtime::component::Resource;

impl HostGrpcClient for WasiState {
    async fn new(
        &mut self,
        configuration: GrpcClientConfiguration,
    ) -> wasmtime::Result<Result<Resource<GrpcClient>, String>> {
        tracing::debug!("Creating new gRPC client for URI: {}", configuration.uri);

        let client = match self.grpc_clients().entry(configuration.uri.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let transport = match tonic::transport::Endpoint::new(configuration.uri)?.connect().await {
                    Ok(transport) => transport,
                    Err(err) => return Ok(Err(err.to_string())),
                };

                let client = tonic::client::Grpc::new(transport);

                entry.insert(client.clone());

                client
            }
        };

        let resource = self.push_resource(client)?;

        Ok(Ok(resource))
    }

    async fn unary(
        &mut self,
        self_: Resource<GrpcClient>,
        message: Vec<u8>,
        service: String,
        method: String,
        metadata: MetadataMap,
        timeout: Option<u64>,
    ) -> wasmtime::Result<Result<GrpcUnaryResponse, GrpcStatus>> {
        let client = self.get_mut(&self_)?;

        client
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {e}")))?;

        let path_and_query: http::uri::PathAndQuery = format!("/{service}/{method}").parse()?;
        let mut request = tonic::Request::new(message);

        for (key, value) in metadata {
            request.metadata_mut().insert_bin(
                MetadataKey::from_bytes(key.as_bytes()).unwrap(),
                MetadataValue::from_bytes(&value),
            );
        }

        if let Some(timeout) = timeout {
            request.set_timeout(std::time::Duration::from_millis(timeout));
        }

        let service: &'static str = Box::leak(service.into_boxed_str());
        let method: &'static str = Box::leak(method.into_boxed_str());

        request.extensions_mut().insert(GrpcMethod::new(service, method));

        tracing::debug!("Sending unary request to {path_and_query}");

        match client.unary(request, path_and_query, TrivialCodec).await {
            Ok(response) => Ok(Ok(GrpcUnaryResponse {
                metadata: tonic_metadata_to_wasi_metadata(response.metadata()),
                message: response.into_inner(),
            })),
            Err(err) => Ok(Err(GrpcStatus {
                code: convert_grpc_status_code(err.code()),
                message: err.message().to_owned(),
                metadata: tonic_metadata_to_wasi_metadata(err.metadata()),
            })),
        }
    }

    async fn streaming(
        &mut self,
        _self_: wasmtime::component::Resource<GrpcClient>,
        _message: Vec<u8>,
        _method: String,
        _metadata: MetadataMap,
        _timeout: Option<u64>,
    ) -> wasmtime::Result<Result<wasmtime::component::Resource<GrpcStreamingResponse>, GrpcStatus>> {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<GrpcClient>) -> wasmtime::Result<()> {
        Ok(())
    }
}

struct TrivialCodec;

impl tonic::codec::Codec for TrivialCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = Self;
    type Decoder = Self;

    fn encoder(&mut self) -> Self::Encoder {
        TrivialCodec
    }

    fn decoder(&mut self) -> Self::Decoder {
        TrivialCodec
    }
}

impl tonic::codec::Encoder for TrivialCodec {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, dst: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put_slice(item.as_slice());
        Ok(())
    }
}

impl tonic::codec::Decoder for TrivialCodec {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn decode(&mut self, src: &mut tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        use bytes::Buf;
        use std::io::Read;

        let mut out = Vec::with_capacity(src.remaining());
        src.reader().read_to_end(&mut out)?;
        Ok(Some(out))
    }
}

fn convert_grpc_status_code(code: tonic::Code) -> super::GrpcStatusCode {
    match code {
        tonic::Code::Ok => super::GrpcStatusCode::Ok,
        tonic::Code::Cancelled => super::GrpcStatusCode::Cancelled,
        tonic::Code::Unknown => super::GrpcStatusCode::Unknown,
        tonic::Code::InvalidArgument => super::GrpcStatusCode::InvalidArgument,
        tonic::Code::DeadlineExceeded => super::GrpcStatusCode::DeadlineExceeded,
        tonic::Code::NotFound => super::GrpcStatusCode::NotFound,
        tonic::Code::AlreadyExists => super::GrpcStatusCode::AlreadyExists,
        tonic::Code::PermissionDenied => super::GrpcStatusCode::PermissionDenied,
        tonic::Code::ResourceExhausted => super::GrpcStatusCode::ResourceExhausted,
        tonic::Code::FailedPrecondition => super::GrpcStatusCode::FailedPrecondition,
        tonic::Code::Aborted => super::GrpcStatusCode::Aborted,
        tonic::Code::OutOfRange => super::GrpcStatusCode::OutOfRange,
        tonic::Code::Unimplemented => super::GrpcStatusCode::Unimplemented,
        tonic::Code::Internal => super::GrpcStatusCode::Internal,
        tonic::Code::Unavailable => super::GrpcStatusCode::Unavailable,
        tonic::Code::DataLoss => super::GrpcStatusCode::DataLoss,
        tonic::Code::Unauthenticated => super::GrpcStatusCode::Unauthenticated,
    }
}

fn tonic_metadata_to_wasi_metadata(metadata: &tonic::metadata::MetadataMap) -> MetadataMap {
    metadata
        .iter()
        .map(|kv| match kv {
            tonic::metadata::KeyAndValueRef::Ascii(metadata_key, metadata_value) => {
                (metadata_key.as_str().to_owned(), metadata_value.as_bytes().to_owned())
            }
            tonic::metadata::KeyAndValueRef::Binary(metadata_key, metadata_value) => {
                (metadata_key.as_str().to_owned(), metadata_value.as_ref().to_owned())
            }
        })
        .collect()
}
