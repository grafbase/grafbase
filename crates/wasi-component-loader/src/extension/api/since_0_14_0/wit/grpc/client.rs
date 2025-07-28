use super::{
    GrpcClient, GrpcClientConfiguration, GrpcStatus, GrpcStreamingResponse, GrpcUnaryResponse, HostGrpcClient,
    MetadataMap,
};
use crate::InstanceState;
use bytes::BufMut as _;
use dashmap::Entry;
use tonic::metadata::{MetadataKey, MetadataValue};
use wasmtime::component::Resource;

impl HostGrpcClient for InstanceState {
    async fn new(
        &mut self,
        configuration: GrpcClientConfiguration,
    ) -> wasmtime::Result<Result<Resource<GrpcClient>, String>> {
        tracing::debug!("Creating new gRPC client for URI: {}", configuration.uri);

        let client = match self.grpc_clients.entry(configuration.uri.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let endpoint = match tonic::transport::Endpoint::new(configuration.uri) {
                    Ok(endpoint) => endpoint,
                    Err(err) => return Ok(Err(err.to_string())),
                };

                let transport = match endpoint.connect().await {
                    Ok(transport) => transport,
                    Err(err) => return Ok(Err(err.to_string())),
                };

                let client = tonic::client::Grpc::new(transport);

                entry.insert(client.clone());

                client
            }
        };

        let resource = self.resources.push(client)?;

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
        let client = self.resources.get_mut(&self_)?;

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

        tracing::debug!("Sending unary request to {path_and_query}");

        match client.unary(request, path_and_query, IdentityCodec).await {
            Ok(response) => Ok(Ok(GrpcUnaryResponse {
                metadata: tonic_metadata_to_wasi_metadata(response.metadata()),
                message: response.into_inner(),
            })),
            Err(err) => Ok(Err(tonic_status_to_grpc_status(err))),
        }
    }

    async fn streaming(
        &mut self,
        self_: Resource<GrpcClient>,
        message: Vec<u8>,
        service: String,
        method: String,
        metadata: MetadataMap,
        timeout: Option<u64>,
    ) -> wasmtime::Result<Result<wasmtime::component::Resource<GrpcStreamingResponse>, GrpcStatus>> {
        let client = self.resources.get_mut(&self_)?;

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

        tracing::debug!("Sending server streaming request to {path_and_query}");

        match client.server_streaming(request, path_and_query, IdentityCodec).await {
            Ok(stream) => Ok(Ok(self.resources.push(stream.into_parts())?)),
            Err(err) => Ok(Err(tonic_status_to_grpc_status(err))),
        }
    }

    async fn drop(&mut self, rep: Resource<GrpcClient>) -> wasmtime::Result<()> {
        self.resources.delete(rep)?;
        Ok(())
    }
}

struct IdentityCodec;

impl tonic::codec::Codec for IdentityCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = Self;
    type Decoder = Self;

    fn encoder(&mut self) -> Self::Encoder {
        IdentityCodec
    }

    fn decoder(&mut self) -> Self::Decoder {
        IdentityCodec
    }
}

impl tonic::codec::Encoder for IdentityCodec {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, dst: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put_slice(item.as_slice());
        Ok(())
    }
}

impl tonic::codec::Decoder for IdentityCodec {
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

impl From<tonic::Code> for super::GrpcStatusCode {
    fn from(code: tonic::Code) -> Self {
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
}

pub(super) fn tonic_metadata_to_wasi_metadata(metadata: &tonic::metadata::MetadataMap) -> MetadataMap {
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

pub(super) fn tonic_status_to_grpc_status(err: tonic::Status) -> GrpcStatus {
    GrpcStatus {
        code: err.code().into(),
        message: err.message().to_owned(),
        metadata: tonic_metadata_to_wasi_metadata(err.metadata()),
    }
}
