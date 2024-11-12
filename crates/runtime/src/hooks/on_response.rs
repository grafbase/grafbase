use std::time::Duration;
use web_time::Instant;

pub use grafbase_telemetry::graphql::GraphqlResponseStatus;

#[derive(Debug, Clone, Copy)]
pub struct ResponseInfo {
    pub connection_time: Duration,
    pub response_time: Duration,
    pub status_code: Option<http::StatusCode>,
}

impl ResponseInfo {
    pub fn builder() -> ResponseInfoBuilder {
        ResponseInfoBuilder {
            start: Instant::now(),
            connection_time: None,
            response_time: None,
        }
    }
}

pub struct ResponseInfoBuilder {
    start: Instant,
    connection_time: Option<Duration>,
    response_time: Option<Duration>,
}

impl ResponseInfoBuilder {
    /// Stops the clock for connection time. This is typically the time the request gets
    /// sent, but no data is fetched back.
    pub fn track_connection(&mut self) {
        self.connection_time = Some(self.start.elapsed())
    }

    /// Stops the clock for response time. This time is the time it takes to initialize
    /// a connection and waiting to get all the data back.
    pub fn track_response(&mut self) {
        self.response_time = Some(self.start.elapsed())
    }

    pub fn build(self, status_code: Option<http::StatusCode>) -> ResponseInfo {
        ResponseInfo {
            connection_time: self.connection_time.unwrap_or_default(),
            response_time: self.response_time.unwrap_or_default(),
            status_code,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CacheStatus {
    Hit,
    PartialHit,
    Miss,
}

#[derive(Debug, Clone, Copy)]
pub enum SubgraphRequestExecutionKind {
    InternalServerError,
    HookError,
    RequestError,
    RateLimited,
    Responsed(ResponseInfo),
}

#[derive(Debug, Clone)]
pub struct ExecutedSubgraphRequest<'a> {
    pub subgraph_name: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub executions: Vec<SubgraphRequestExecutionKind>,
    pub cache_status: CacheStatus,
    pub total_duration: Duration,
    pub has_graphql_errors: bool,
}

impl<'a> ExecutedSubgraphRequest<'a> {
    pub fn builder(subgraph_name: &'a str, method: &'a str, url: &'a str) -> ExecutedSubgraphRequestBuilder<'a> {
        ExecutedSubgraphRequestBuilder {
            subgraph_name,
            method,
            url,
            executions: Vec::new(),
            cache_status: None,
            status: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedSubgraphRequestBuilder<'a> {
    subgraph_name: &'a str,
    method: &'a str,
    url: &'a str,
    executions: Vec<SubgraphRequestExecutionKind>,
    cache_status: Option<CacheStatus>,
    status: Option<GraphqlResponseStatus>,
}

impl<'a> ExecutedSubgraphRequestBuilder<'a> {
    pub fn push_execution(&mut self, kind: SubgraphRequestExecutionKind) {
        self.executions.push(kind);
    }

    pub fn set_cache_status(&mut self, status: CacheStatus) {
        self.cache_status = Some(status);
    }

    pub fn set_graphql_response_status(&mut self, status: GraphqlResponseStatus) {
        self.status = Some(status);
    }

    pub fn build(self, duration: Duration) -> ExecutedSubgraphRequest<'a> {
        ExecutedSubgraphRequest {
            subgraph_name: self.subgraph_name,
            method: self.method,
            url: self.url,
            executions: self.executions,
            cache_status: self.cache_status.unwrap_or(CacheStatus::Miss),
            total_duration: duration,
            has_graphql_errors: self.status.map(|status| !status.is_success()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedOperation<'a, OnSubgraphResponseOutput> {
    pub name: Option<&'a str>,
    pub document: &'a str,
    pub prepare_duration: Duration,
    pub cached_plan: bool,
    pub duration: Duration,
    pub status: GraphqlResponseStatus,
    pub on_subgraph_response_outputs: Vec<OnSubgraphResponseOutput>,
}

impl<'a, OnSubgraphResponseOutput> ExecutedOperation<'a, OnSubgraphResponseOutput> {
    pub fn builder() -> ExecutedOperationBuilder<OnSubgraphResponseOutput> {
        ExecutedOperationBuilder {
            start_time: Instant::now(),
            on_subgraph_response_outputs: Vec::new(),
            prepare_duration: None,
            cached_plan: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedOperationBuilder<OnSubgraphResponseOutput> {
    pub start_time: Instant,
    pub prepare_duration: Option<Duration>,
    pub cached_plan: bool,
    pub on_subgraph_response_outputs: Vec<OnSubgraphResponseOutput>,
}

impl<OnSubgraphResponseOutput> ExecutedOperationBuilder<OnSubgraphResponseOutput> {
    pub fn push_on_subgraph_response_output(&mut self, output: OnSubgraphResponseOutput) {
        self.on_subgraph_response_outputs.push(output);
    }

    pub fn track_prepare(&mut self) -> Duration {
        let prepare_duration = self.start_time.elapsed();
        self.prepare_duration = Some(prepare_duration);
        prepare_duration
    }

    pub fn set_cached_plan(&mut self) {
        self.cached_plan = true;
    }

    pub fn build<'a>(
        self,
        name: Option<&'a str>,
        document: &'a str,
        status: GraphqlResponseStatus,
    ) -> ExecutedOperation<'a, OnSubgraphResponseOutput> {
        ExecutedOperation {
            duration: self.start_time.elapsed(),
            status,
            on_subgraph_response_outputs: self.on_subgraph_response_outputs,
            name,
            document,
            prepare_duration: self.prepare_duration.unwrap_or_default(),
            cached_plan: self.cached_plan,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedHttpRequest<OnOperationResponseOutput> {
    pub method: http::Method,
    pub url: http::Uri,
    pub status_code: http::StatusCode,
    pub on_operation_response_outputs: Vec<OnOperationResponseOutput>,
}
