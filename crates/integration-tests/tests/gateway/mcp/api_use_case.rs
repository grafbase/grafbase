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
        # Incomplete fields
        type Mutation {
          "Request a review for a schema proposal from a user or a team."
          schemaProposalReviewRequestCreate(input: SchemaProposalReviewRequestCreateInput!): SchemaProposalReviewRequestCreatePayload!
          "Create new organization account owned by the current user. Slug must be unique."
          organizationCreate(input: OrganizationCreateInput!): OrganizationCreatePayload!
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

        type GraphOperationCheckConfiguration {
          "The clients to exclude from operation checks."
          excludedClients: [String!]!
          "The operations to exclude from operation checks."
          excludedOperations: [String!]!
          """
          The request count threshold to consider for operation checks. Operations that have been
          registered less than the specified number of occurrences are ignored.
          """
          requestCountThreshold: Int!
          """
          The time range in days to consider for operation checks. Operations older than the specificied
          number of days are ignored.
          """
          timeRangeDays: Int!
        }

        type Graph {
          account: Account!
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          branch(name: String): Branch
          branches(after: String, before: String, first: Int, last: Int): BranchConnection!
          createdAt: DateTime!
          "Webhooks for custom schema checks."
          customCheckWebhooks: [CustomCheckWebhook!]
          id: ID!
          operationChecksConfiguration: GraphOperationCheckConfiguration!
          owners: [Team!]!
          productionBranch: Branch!
          request(
            branchName: String,
            "The approximate timestamp of the request, within a few minutes of the actual request."
            approximateTimestamp: DateTime!,
            traceId: ID!
          ): Request
          requests(after: String, before: String, first: Int, last: Int, filters: RequestFilters!): RequestConnection
          schemaChecks(after: String, before: String, first: Int, last: Int, branch: String): SchemaCheckConnection!
          schemaProposals(after: String, first: Int): SchemaProposalConnection!
          slug: String!
        }

        "RFC3339 formatted date in the UTC time zone denoted by letter 'Z'"
        scalar DateTime

        type Branch {
          activeDeployment: Deployment
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          deployments(after: String, before: String, first: Int, last: Int, filters: DeploymentFilters): DeploymentConnection!
          domains: [String!]!
          endpointConfig: EndpointConfig
          environment: BranchEnvironment!
          federatedSchema: String
          graph: Graph!
          id: ID!
          latestDeployment: Deployment
          name: String!
          operationChecksEnabled: Boolean!
          schema: String
          schemaProposals(after: String, first: Int, filter: SchemaProposalFilter!): SchemaProposalConnection!
          schemaProposalsConfiguration: SchemaProposalsConfiguration!
          subgraphs: [Subgraph!]!
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

        type TopOperations {
          byName: TopOperationsByName!
          byNameAndHash: TopOperationsByNameAndHash!
        }

        type TopClients {
          byName: TopClientsByName!
          byNameAndVersion: TopClientsByNameAndVersion!
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

        type TopOperationsByNameAndHash {
          orderedByHighestCount: [TopOperationByNameAndHashOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopOperationByNameAndHashOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopOperationByNameAndHashOrderedByHighestLatency!]!
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

        type TopClientsByNameAndVersion {
          orderedByHighestCount: [TopClientByNameAndVersionOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameAndVersionOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameAndVersionOrderedByHighestLatency!]!
        }

        type TopClientsByName {
          orderedByHighestCount: [TopClientByNameOrderedByHighestCount!]!
          orderedByHighestErrorRatio: [TopClientByNameOrderedByHighestErrorRatio!]!
          orderedByHighestLatency: [TopClientByNameOrderedByHighestLatency!]!
        }

        type FieldMetricsTimeSeries {
          overall: FieldMetrics!
          points: [FieldMetricsTimeSeriesDataPoint!]!
          previousPeriod: FieldMetricsTimeSeries
        }

        union OrganizationCreatePayload = NameSizeCheckError | OrganizationCreateSuccess | ReservedSlugsCheckError | SlugAlreadyExistsError | SlugError | SlugSizeCheckError | TrialPlanUnavailableError

        type OrganizationCreateSuccess {
          member: Member!
          organization: Organization!
          query: Query!
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

        type GraphOperationCheckConfiguration {
          "The clients to exclude from operation checks."
          excludedClients: [String!]!
          "The operations to exclude from operation checks."
          excludedOperations: [String!]!
          """
          The request count threshold to consider for operation checks. Operations that have been
          registered less than the specified number of occurrences are ignored.
          """
          requestCountThreshold: Int!
          """
          The time range in days to consider for operation checks. Operations older than the specificied
          number of days are ignored.
          """
          timeRangeDays: Int!
        }

        type Graph {
          account: Account!
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          branch(name: String): Branch
          branches(after: String, before: String, first: Int, last: Int): BranchConnection!
          createdAt: DateTime!
          "Webhooks for custom schema checks."
          customCheckWebhooks: [CustomCheckWebhook!]
          id: ID!
          operationChecksConfiguration: GraphOperationCheckConfiguration!
          owners: [Team!]!
          productionBranch: Branch!
          request(
            branchName: String,
            "The approximate timestamp of the request, within a few minutes of the actual request."
            approximateTimestamp: DateTime!,
            traceId: ID!
          ): Request
          requests(after: String, before: String, first: Int, last: Int, filters: RequestFilters!): RequestConnection
          schemaChecks(after: String, before: String, first: Int, last: Int, branch: String): SchemaCheckConnection!
          schemaProposals(after: String, first: Int): SchemaProposalConnection!
          slug: String!
        }

        "RFC3339 formatted date in the UTC time zone denoted by letter 'Z'"
        scalar DateTime

        type Branch {
          activeDeployment: Deployment
          analytics(filters: GraphAnalyticsFilters!): GraphAnalytics
          deployments(after: String, before: String, first: Int, last: Int, filters: DeploymentFilters): DeploymentConnection!
          domains: [String!]!
          endpointConfig: EndpointConfig
          environment: BranchEnvironment!
          federatedSchema: String
          graph: Graph!
          id: ID!
          latestDeployment: Deployment
          name: String!
          operationChecksEnabled: Boolean!
          schema: String
          schemaProposals(after: String, first: Int, filter: SchemaProposalFilter!): SchemaProposalConnection!
          schemaProposalsConfiguration: SchemaProposalsConfiguration!
          subgraphs: [Subgraph!]!
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

        type TopOperations {
          byName: TopOperationsByName!
          byNameAndHash: TopOperationsByNameAndHash!
        }

        type TopClients {
          byName: TopClientsByName!
          byNameAndVersion: TopClientsByNameAndVersion!
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

        "Deployment"
        type Deployment {
          "The schema exposed by the gateway."
          apiSchema: String
          "Diff of the API SDL in this deployment with the last successful deployment. This field only makes sense for successful deployments, so it will be null on failed deployments."
          apiSchemaDiff: [DiffSnippet!]
          branch: Branch!
          changeCounts: DeploymentChangeCounts
          compositionInputs: [DeploymentSubgraph!]!
          createdAt: DateTime!
          "The duration of the deployment in milliseconds."
          duration: Int
          "The federated SDL used to initialize the gateway."
          federatedSdl: String
          finishedAt: DateTime
          id: ID!
          isRedeployable: Boolean!
          startedAt: DateTime
          status: DeploymentStatus!
          steps: [DeploymentStep!]!
          """
          The subgraph that was published or removed, triggering the deployment.
          
          This is nullable in case we introduce back redeployments in the future.
          """
          subgraph: DeploymentSubgraph
        }

        type SchemaProposalConnection {
          "A list of edges."
          edges: [SchemaProposalEdge!]!
          "A list of nodes."
          nodes: [SchemaProposal!]!
          "Information to aid in pagination."
          pageInfo: PageInfo!
        }

        "Information about pagination in a connection"
        type PageInfo {
          "When paginating forwards, the cursor to continue."
          endCursor: String
          "When paginating forwards, are there more items?"
          hasNextPage: Boolean!
          "When paginating backwards, are there more items?"
          hasPreviousPage: Boolean!
          "When paginating backwards, the cursor to continue."
          startCursor: String
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

        type ErrorCountByCode {
          code: String!
          count: Int!
        }

        type Subgraph {
          createdAt: DateTime!
          name: String!
          owners: [Team!]!
          schema: String!
          updatedAt: DateTime!
          url: String
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
                "execute",
                json!(
                    {
                      "query": "query GetRequestMetrics {\n  graphByAccountSlug(accountSlug: \"grafbase\", graphSlug: \"api\") {\n    analytics(filters: {\n      from: \"2025-04-01T00:00:00Z\"\n      to: \"2025-04-05T23:59:59Z\"\n    }) {\n      requestMetrics {\n        overall {\n          requestCount\n        }\n      }\n    }\n  }\n}",
                      "variables": {}
                    }
                )
            )
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "RequestMetricsV2 does not have a field named 'requestCount'.",
              "locations": [
                {
                  "line": 9,
                  "column": 11
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        ================================================================================

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
        "#);
    });
}
