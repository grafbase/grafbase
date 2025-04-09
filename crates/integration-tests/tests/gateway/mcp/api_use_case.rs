use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

const SDL: &str = include_str!("../../../data/federated-api.graphql");

#[test]
fn search_org_analytics() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_federated_sdl(SDL)
            .with_toml_config(
                r#"
            [mcp]
            enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;
        let response = stream
            .call_tool(
                "search",
                json!({"keywords": ["request count", "analytics", "organization"]}),
            )
            .await;
        insta::assert_snapshot!(response, @r##"
        "RFC3339 formatted date in the UTC time zone denoted by letter 'Z'"
        scalar DateTime

        input GraphAnalyticsFilters {
          "Defaults to production branch"
          branchName: String
          "Use this if you really care about having a specific duration like 1 hour, 7 days, etc."
          range: Duration
          """
          Use this if you *at least* the data between `from` and `to` to be provided. You may get
          more, but never less.
          """
          from: DateTime
          "To be used in conjunction with with either `range` or `from`."
          to: DateTime!
          """
          If explicitly to false, specifying both `from` and `to` will be treated as if `range: (to - from)`
          had been specified instead. Meaning only the duration between `from` and `to` matters, not
          necessarily having a data point for `from` itself.
          """
          isCustomRange: Boolean
          """
          If specified, overrides approximateNumberOfPoints. Must be in whole minutes.
          At most 150 points can be returned.
          """
          aggregationStep: Duration
          "Defaults to 100, at most 150 points can be returned."
          approximateNumberOfDataPoints: Int
          """
          Defaults to true
          Example: for an aggregationStep of 15 min:
          - if true, only times with 00, 15, 30 and 45 minutes will appear in the time series
          - if false, times in the time series will be adjusted to start from the periodStart (~from).
          So if from = 15:32:00, times will end in 02, 17, 32 and 47.
          I'll always align to the aggregation step used to store the data though, which
          is currently in minutes. So cannot have 15:10:20, 15:11:20, etc.
          """
          alignPeriodWithAggregationStep: Boolean
          operationName: [String!]
          "Only used if operation name is specified."
          operationNormalizedQueryHash: [OperationNormalizedQueryHash!]
          clientName: [String!]
          "Only used if client name is specified."
          clientVersion: [String!]
        }

        # Incomplete fields
        type Branch {
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
        }

        # Incomplete fields
        type Invite {
          organization: Organization!
        }

        # Incomplete fields
        type Graph {
          request(
            branchName: String,
            "The approximate timestamp of the request, within a few minutes of the actual request."
            approximateTimestamp: DateTime!,
            traceId: ID!
          ): Request
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          operationChecksConfiguration: GraphOperationCheckConfiguration!
        }

        # Incomplete fields
        type GraphOperationCheckConfiguration {
          """
          The request count threshold to consider for operation checks. Operations that have been
          registered less than the specified number of occurrences are ignored.
          """
          requestCountThreshold: Int!
        }

        # Incomplete fields
        type Query {
          invite(id: ID!): Invite
          "Get a graph by account slug and slug of the graph itself."
          graphByAccountSlug(
            "slug of the account"
            accountSlug: String!,
            "slug of the graph"
            graphSlug: String!
          ): Graph
          "Get branch by account slug, graph slug and the name of the branch."
          branch(
            "name of the branch"
            name: String,
            "slug of the account"
            accountSlug: String,
            "slug of the graph"
            graphSlug: String,
            "slug of the project"
            projectSlug: String
          ): Branch
        }

        type GraphAnalytics {
          forField(
            "Schema path defined as: '<parent-type-name>.<name>'"
            schemaPath: String!
          ): FieldAnalytics!
          requestMetrics(
            "Latency percentiles to retrieve. Ex: [50, 99, 99.9]"
            latencyPercentiles: [Float!]
          ): RequestMetricsTimeSeriesV2
          topClients(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the client names/versions"
            searchQuery: String,
            "If not specified, top clients by latency will be empty. Ex: 95"
            latencyPercentile: Float
          ): TopClients
          topOperations(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the opeartion names"
            searchQuery: String,
            "If not specified, top operations by latency will be empty. Ex: 95"
            latencyPercentile: Float
          ): TopOperations
        }

        type RequestMetricsTimeSeriesV2 {
          overall: RequestMetricsV2!
          points: [RequestMetricsTimeSeriesDataPointV2!]!
          previousPeriod: RequestMetricsTimeSeriesV2
        }

        type RequestMetricsV2 {
          cacheHitCount: Int!
          cacheMissCount: Int!
          cachePassCount: Int!
          count: Int!
          error4XxCount: Int!
          error5XxCount: Int!
          errorGraphqlCount: Int!
          latencyMsPercentiles: [Int!]!
        }

        type RequestMetricsTimeSeriesDataPointV2 {
          cacheHitCount: Int!
          cacheMissCount: Int!
          cachePassCount: Int!
          count: Int!
          dateTime: DateTime!
          error4XxCount: Int!
          error5XxCount: Int!
          errorGraphqlCount: Int!
          latencyMsPercentiles: [Int!]!
        }

        type TopOperations {
          byName: TopOperationsByName!
          byNameAndHash: TopOperationsByNameAndHash!
        }

        type TopClients {
          byName: TopClientsByName!
          byNameAndVersion: TopClientsByNameAndVersion!
        }

        type TopOperationsByName {
          orderedByHighestCount: [TopOperationByNameOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopOperationByNameOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopOperationByNameOrderedByHighestLatency!]!
        }

        type TopOperationsByNameAndHash {
          orderedByHighestCount: [TopOperationByNameAndHashOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopOperationByNameAndHashOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopOperationByNameAndHashOrderedByHighestLatency!]!
        }

        type TopClientsByName {
          orderedByHighestCount: [TopClientByNameOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameOrderedByHighestLatency!]!
        }

        type TopClientsByNameAndVersion {
          orderedByHighestCount: [TopClientByNameAndVersionOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameAndVersionOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameAndVersionOrderedByHighestLatency!]!
        }

        type FieldAnalytics {
          metrics: FieldMetricsTimeSeries
          topClients(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the client names/versions"
            searchQuery: String
          ): TopClientsForField
        }

        type FieldMetricsTimeSeries {
          overall: FieldMetrics!
          points: [FieldMetricsTimeSeriesDataPoint!]!
          previousPeriod: FieldMetricsTimeSeries
        }

        type TopClientsForField {
          byName: TopClientsForFieldByName!
          byNameAndVersion: TopClientsForFieldByNameAndVersion!
        }
        "##);
    });
}
#[test]
fn search_analytics() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_federated_sdl(SDL)
            .with_toml_config(
                r#"
            [mcp]
            enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;
        let response = stream
            .call_tool(
                "search",
                json!({"keywords": ["request count", "analytics", "api graph"]}),
            )
            .await;
        insta::assert_snapshot!(response, @r##"
        "RFC3339 formatted date in the UTC time zone denoted by letter 'Z'"
        scalar DateTime

        input GraphAnalyticsFilters {
          "Defaults to production branch"
          branchName: String
          "Use this if you really care about having a specific duration like 1 hour, 7 days, etc."
          range: Duration
          """
          Use this if you *at least* the data between `from` and `to` to be provided. You may get
          more, but never less.
          """
          from: DateTime
          "To be used in conjunction with with either `range` or `from`."
          to: DateTime!
          """
          If explicitly to false, specifying both `from` and `to` will be treated as if `range: (to - from)`
          had been specified instead. Meaning only the duration between `from` and `to` matters, not
          necessarily having a data point for `from` itself.
          """
          isCustomRange: Boolean
          """
          If specified, overrides approximateNumberOfPoints. Must be in whole minutes.
          At most 150 points can be returned.
          """
          aggregationStep: Duration
          "Defaults to 100, at most 150 points can be returned."
          approximateNumberOfDataPoints: Int
          """
          Defaults to true
          Example: for an aggregationStep of 15 min:
          - if true, only times with 00, 15, 30 and 45 minutes will appear in the time series
          - if false, times in the time series will be adjusted to start from the periodStart (~from).
          So if from = 15:32:00, times will end in 02, 17, 32 and 47.
          I'll always align to the aggregation step used to store the data though, which
          is currently in minutes. So cannot have 15:10:20, 15:11:20, etc.
          """
          alignPeriodWithAggregationStep: Boolean
          operationName: [String!]
          "Only used if operation name is specified."
          operationNormalizedQueryHash: [OperationNormalizedQueryHash!]
          clientName: [String!]
          "Only used if client name is specified."
          clientVersion: [String!]
        }

        # Incomplete fields
        type Branch {
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          graph: Graph!
        }

        # Incomplete fields
        type Query {
          "Get a graph by account slug and slug of the graph itself."
          graphByAccountSlug(
            "slug of the account"
            accountSlug: String!,
            "slug of the graph"
            graphSlug: String!
          ): Graph
          "Get branch by account slug, graph slug and the name of the branch."
          branch(
            "name of the branch"
            name: String,
            "slug of the account"
            accountSlug: String,
            "slug of the graph"
            graphSlug: String,
            "slug of the project"
            projectSlug: String
          ): Branch
        }

        # Incomplete fields
        type Graph {
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          request(
            branchName: String,
            "The approximate timestamp of the request, within a few minutes of the actual request."
            approximateTimestamp: DateTime!,
            traceId: ID!
          ): Request
        }

        type GraphAnalytics {
          forField(
            "Schema path defined as: '<parent-type-name>.<name>'"
            schemaPath: String!
          ): FieldAnalytics!
          requestMetrics(
            "Latency percentiles to retrieve. Ex: [50, 99, 99.9]"
            latencyPercentiles: [Float!]
          ): RequestMetricsTimeSeriesV2
          topClients(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the client names/versions"
            searchQuery: String,
            "If not specified, top clients by latency will be empty. Ex: 95"
            latencyPercentile: Float
          ): TopClients
          topOperations(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the opeartion names"
            searchQuery: String,
            "If not specified, top operations by latency will be empty. Ex: 95"
            latencyPercentile: Float
          ): TopOperations
        }

        type RequestMetricsTimeSeriesV2 {
          overall: RequestMetricsV2!
          points: [RequestMetricsTimeSeriesDataPointV2!]!
          previousPeriod: RequestMetricsTimeSeriesV2
        }

        type TopClients {
          byName: TopClientsByName!
          byNameAndVersion: TopClientsByNameAndVersion!
        }

        type TopOperations {
          byName: TopOperationsByName!
          byNameAndHash: TopOperationsByNameAndHash!
        }

        type FieldAnalytics {
          metrics: FieldMetricsTimeSeries
          topClients(
            "Detaults to 10, Max 100"
            limit: Int,
            "Search over the client names/versions"
            searchQuery: String
          ): TopClientsForField
        }

        type RequestMetricsV2 {
          cacheHitCount: Int!
          cacheMissCount: Int!
          cachePassCount: Int!
          count: Int!
          error4XxCount: Int!
          error5XxCount: Int!
          errorGraphqlCount: Int!
          latencyMsPercentiles: [Int!]!
        }

        type RequestMetricsTimeSeriesDataPointV2 {
          cacheHitCount: Int!
          cacheMissCount: Int!
          cachePassCount: Int!
          count: Int!
          dateTime: DateTime!
          error4XxCount: Int!
          error5XxCount: Int!
          errorGraphqlCount: Int!
          latencyMsPercentiles: [Int!]!
        }

        type Request {
          clientName: String!
          clientVersion: String!
          endedAt: DateTime!
          errorCount: Int!
          errorCountByCode: [ErrorCountByCode!]!
          httpRequestMethod: String!
          httpStatusCode: Int!
          id: ID!
          operations: [RequestOperation!]!
          rootSpanId: ID!
          startedAt: DateTime!
          trace: Trace!
          urlPath: String!
          userAgent: String!
        }

        type FieldMetricsTimeSeries {
          overall: FieldMetrics!
          points: [FieldMetricsTimeSeriesDataPoint!]!
          previousPeriod: FieldMetricsTimeSeries
        }

        type TopClientsByName {
          orderedByHighestCount: [TopClientByNameOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameOrderedByHighestLatency!]!
        }

        type TopOperationsByName {
          orderedByHighestCount: [TopOperationByNameOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopOperationByNameOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopOperationByNameOrderedByHighestLatency!]!
        }

        type TopClientsForField {
          byName: TopClientsForFieldByName!
          byNameAndVersion: TopClientsForFieldByNameAndVersion!
        }

        type TopOperationsByNameAndHash {
          orderedByHighestCount: [TopOperationByNameAndHashOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopOperationByNameAndHashOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopOperationByNameAndHashOrderedByHighestLatency!]!
        }

        type TopClientsByNameAndVersion {
          orderedByHighestCount: [TopClientByNameAndVersionOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameAndVersionOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameAndVersionOrderedByHighestLatency!]!
        }
        "##);
    });
}

#[test]
fn verify_analytics() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_federated_sdl(SDL)
            .with_toml_config(
                r#"
            [mcp]
            enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;
        let response = stream
            .call_tool(
                "verify",
                json!(
                    {
                      "query": "query GetRequestMetrics {\n  graphByAccountSlug(accountSlug: \"grafbase\", graphSlug: \"api\") {\n    analytics(filters: {\n      from: \"2025-04-01T00:00:00Z\"\n      to: \"2025-04-05T23:59:59Z\"\n    }) {\n      requestMetrics {\n        overall {\n          requestCount\n        }\n      }\n    }\n  }\n}",
                      "variables": {}
                    }
                )
            )
            .await;
        insta::assert_snapshot!(response, @r#"
        type RequestMetricsV2 {
          cacheHitCount: Int!
          cacheMissCount: Int!
          cachePassCount: Int!
          count: Int!
          error4XxCount: Int!
          error5XxCount: Int!
          errorGraphqlCount: Int!
          latencyMsPercentiles: [Int!]!
        }


        ================================================================================

        {
          "errors": [
            "RequestMetricsV2 does not have a field named 'requestCount'."
          ]
        }
        "#);
    });
}
