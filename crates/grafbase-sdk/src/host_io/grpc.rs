//! Generic gRPC client.

use crate::wit;

/// A successful response from a unary gRPC call.
#[derive(Debug)]
pub struct GrpcUnaryResponse {
    inner: wit::GrpcUnaryResponse,
}

impl GrpcUnaryResponse {
    /// The response body.
    pub fn message(&self) -> &[u8] {
        &self.inner.message
    }

    /// Return the response body, consuming self.
    pub fn into_message(self) -> Vec<u8> {
        self.inner.message
    }

    /// The response metadata.
    pub fn metadata(&self) -> &[(String, Vec<u8>)] {
        &self.inner.metadata
    }
}

/// A response stream from a server streaming gRPC call.
#[derive(Debug)]
pub struct GrpcStreamingResponse {
    inner: wit::GrpcStreamingResponse,
}

impl GrpcStreamingResponse {
    /// Get the next message in the stream. `None` means the stream has ended and there will no longer be any messages.
    pub fn next_message(&self) -> Result<Option<Vec<u8>>, GrpcStatus> {
        self.inner.get_next_message().map_err(|inner| GrpcStatus { inner })
    }

    /// The response metadata.
    pub fn metadata(&self) -> Vec<(String, Vec<u8>)> {
        self.inner.get_metadata()
    }
}

/// An error response from a unary gRPC call.
#[derive(Debug)]
pub struct GrpcStatus {
    inner: wit::GrpcStatus,
}

impl GrpcStatus {
    /// The grpc status code of the response.
    pub fn code(&self) -> GrpcStatusCode {
        self.inner.code.into()
    }

    /// The error message of the response.
    pub fn message(&self) -> &str {
        &self.inner.message
    }

    /// The metadata of the response.
    pub fn metadata(&self) -> &[(String, Vec<u8>)] {
        &self.inner.metadata
    }
}

/// A gRPC client connected to a single endpoint.
#[derive(Debug)]
pub struct GrpcClient {
    inner: wit::GrpcClient,
}

impl GrpcClient {
    /// Construct a new gRPC client.
    pub fn new(endpoint: &str) -> Result<Self, crate::types::Error> {
        Ok(Self {
            inner: wit::GrpcClient::new(&wit::GrpcClientConfiguration {
                uri: endpoint.to_owned(),
            })?,
        })
    }

    /// Make a unary RPC call. The method can be client streaming, but only the provided message will be sent.
    pub fn unary(
        &self,
        message: &[u8],
        service: &str,
        method: &str,
        metadata: &[(String, Vec<u8>)],
        timeout: Option<std::time::Duration>,
    ) -> Result<GrpcUnaryResponse, GrpcStatus> {
        self.inner
            .unary(
                message,
                service,
                method,
                metadata,
                timeout.map(|duration| duration.as_millis() as u64),
            )
            .map(|response| GrpcUnaryResponse { inner: response })
            .map_err(|error| GrpcStatus { inner: error })
    }

    /// Make a server-streaming RPC call. The method can be client streaming, but only the single message provided will be sent.
    pub fn streaming(
        &self,
        message: &[u8],
        service: &str,
        method: &str,
        metadata: &[(String, Vec<u8>)],
        timeout: Option<std::time::Duration>,
    ) -> Result<GrpcStreamingResponse, GrpcStatus> {
        self.inner
            .streaming(
                message,
                service,
                method,
                metadata,
                timeout.map(|duration| duration.as_millis() as u64),
            )
            .map(|response| GrpcStreamingResponse { inner: response })
            .map_err(|error| GrpcStatus { inner: error })
    }
}

/// Response status of gRPC requests.
///
/// Reference: <https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrpcStatusCode {
    /// 0. Not an error; returned on success.
    Ok,
    /// 1. The operation was cancelled, typically by the caller.
    Cancelled,
    /// 2. Unknown error. For example, this error may be returned when a Status value received from another address space belongs to an error space that is not known in this address space. Also errors raised by APIs that do not return enough error information may be converted to this error.
    Unknown,
    /// 3. The client specified an invalid argument. Note that this differs from FAILED_PRECONDITION. INVALID_ARGUMENT indicates arguments that are problematic regardless of the state of the system (e.g., a malformed file name).
    InvalidArgument,
    /// 4. The deadline expired before the operation could complete. For operations that change the state of the system, this error may be returned even if the operation has completed successfully. For example, a successful response from a server could have been delayed long
    DeadlineExceeded,
    /// 5. Some requested entity (e.g., file or directory) was not found. Note to server developers: if a request is denied for an entire class of users, such as gradual feature rollout or undocumented allowlist, NOT_FOUND may be used. If a request is denied for some users within a class of users, such as user-based access control, PERMISSION_DENIED must be used.
    NotFound,
    /// 6. The entity that a client attempted to create (e.g., file or directory) already exists.
    AlreadyExists,
    /// 7. The caller does not have permission to execute the specified operation. PERMISSION_DENIED must not be used for rejections caused by exhausting some resource (use RESOURCE_EXHAUSTED instead for those errors). PERMISSION_DENIED must not be used if the caller can not be identified (use UNAUTHENTICATED instead for those errors). This error code does not imply the request is valid or the requested entity exists or satisfies other pre-conditions.
    PermissionDenied,
    /// 8. Some resource has been exhausted, perhaps a per-user quota, or perhaps the entire file system is out of space.
    ResourceExhausted,
    /// 9. The operation was rejected because the system is not in a state required for the operation's execution. For example, the directory to be deleted is non-empty, an rmdir operation is applied to a non-directory, etc. Service implementors can use the following guidelines to decide between FAILED_PRECONDITION, ABORTED, and UNAVAILABLE: (a) Use UNAVAILABLE if the client can retry just the failing call. (b) Use ABORTED if the client should retry at a higher level (e.g., when a client-specified test-and-set fails, indicating the client should restart a read-modify-write sequence). (c) Use FAILED_PRECONDITION if the client should not retry until the system state has been explicitly fixed. E.g., if an "rmdir" fails because the directory is non-empty, FAILED_PRECONDITION should be returned since the client should not retry unless the files are deleted from the directory.
    FailedPrecondition,
    /// 10. The operation was aborted, typically due to a concurrency issue such as a sequencer check failure or transaction abort. See the guidelines above for deciding between FAILED_PRECONDITION, ABORTED, and UNAVAILABLE.
    Aborted,
    /// 11. The operation was attempted past the valid range. E.g., seeking or reading past end-of-file. Unlike INVALID_ARGUMENT, this error indicates a problem that may be fixed if the system state changes. For example, a 32-bit file system will generate INVALID_ARGUMENT if asked to read at an offset that is not in the range [0,2^32-1], but it will generate OUT_OF_RANGE if asked to read from an offset past the current file size. There is a fair bit of overlap between FAILED_PRECONDITION and OUT_OF_RANGE. We recommend using OUT_OF_RANGE (the more specific error) when it applies so that callers who are iterating through a space can easily look for an OUT_OF_RANGE error to detect when they are done.
    OutOfRange,
    /// 12. The operation is not implemented or is not supported/enabled in this service.
    Unimplemented,
    /// 13. Internal errors. This means that some invariants expected by the underlying system have been broken. This error code is reserved for serious errors.
    Internal,
    /// 14. The service is currently unavailable. This is most likely a transient condition, which can be corrected by retrying with a backoff. Note that it is not always safe to retry non-idempotent operations.
    Unavailable,
    /// 15. Unrecoverable data loss or corruption.
    DataLoss,
    /// 16. The request does not have valid authentication credentials for the operation.
    Unauthenticated,
}

impl From<wit::GrpcStatusCode> for GrpcStatusCode {
    fn from(value: wit::GrpcStatusCode) -> Self {
        match value {
            wit::GrpcStatusCode::Ok => GrpcStatusCode::Ok,
            wit::GrpcStatusCode::Cancelled => GrpcStatusCode::Cancelled,
            wit::GrpcStatusCode::Unknown => GrpcStatusCode::Unknown,
            wit::GrpcStatusCode::InvalidArgument => GrpcStatusCode::InvalidArgument,
            wit::GrpcStatusCode::DeadlineExceeded => GrpcStatusCode::DeadlineExceeded,
            wit::GrpcStatusCode::NotFound => GrpcStatusCode::NotFound,
            wit::GrpcStatusCode::AlreadyExists => GrpcStatusCode::AlreadyExists,
            wit::GrpcStatusCode::PermissionDenied => GrpcStatusCode::PermissionDenied,
            wit::GrpcStatusCode::ResourceExhausted => GrpcStatusCode::ResourceExhausted,
            wit::GrpcStatusCode::FailedPrecondition => GrpcStatusCode::FailedPrecondition,
            wit::GrpcStatusCode::Aborted => GrpcStatusCode::Aborted,
            wit::GrpcStatusCode::OutOfRange => GrpcStatusCode::OutOfRange,
            wit::GrpcStatusCode::Unimplemented => GrpcStatusCode::Unimplemented,
            wit::GrpcStatusCode::Internal => GrpcStatusCode::Internal,
            wit::GrpcStatusCode::Unavailable => GrpcStatusCode::Unavailable,
            wit::GrpcStatusCode::DataLoss => GrpcStatusCode::DataLoss,
            wit::GrpcStatusCode::Unauthenticated => GrpcStatusCode::Unauthenticated,
        }
    }
}
