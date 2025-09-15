use http::request::Parts;
use std::sync::Arc;

use crate::{
    GraphQLServer,
    tools::{ExecuteTool, IntrospectTool, RmcpTool, SearchTool},
};
use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, ErrorCode, ErrorData, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
};

pub(crate) struct McpServer<G: GraphQLServer>(Arc<McpServerInner<G>>);

impl<G: GraphQLServer> Clone for McpServer<G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub(crate) struct McpServerInner<G: GraphQLServer> {
    info: ServerInfo,
    tools: Vec<Box<dyn RmcpTool>>,
    gql: G,
}

impl<G: GraphQLServer> std::ops::Deref for McpServer<G> {
    type Target = McpServerInner<G>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G: GraphQLServer> McpServer<G> {
    pub(crate) async fn new(gql: G, can_mutate: bool) -> anyhow::Result<Self> {
        let default_schema = gql.default_schema().await?;
        Ok(Self(Arc::new(McpServerInner {
            info: ServerInfo {
                protocol_version: ProtocolVersion::LATEST,
                capabilities: ServerCapabilities::builder().enable_tools().build(),
                server_info: Implementation::from_build_env(),
                instructions: None,
            },
            tools: vec![
                Box::new(IntrospectTool::new()),
                Box::new(SearchTool::new(default_schema, can_mutate)?),
                Box::new(ExecuteTool::new(gql.clone(), can_mutate)),
            ],
            gql,
        })))
    }
}

impl<G: GraphQLServer> ServerHandler for McpServer<G> {
    fn get_info(&self) -> ServerInfo {
        self.info.clone()
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: self.tools.iter().map(|tool| tool.to_tool()).collect(),
        })
    }

    async fn call_tool(
        &self,
        CallToolRequestParam { name, arguments }: CallToolRequestParam,
        mut ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Extract parts and retrieve schema once
        let parts = ctx
            .extensions
            .remove::<Parts>()
            .unwrap_or_else(|| http::Request::builder().body(Vec::<u8>::new()).unwrap().into_parts().0);

        let schema = self
            .gql
            .get_schema_for_request(&parts)
            .await
            .map_err(|err| ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None))?;

        if let Some(tool) = self.tools.iter().find(|tool| tool.name() == name) {
            return tool.call(parts, arguments, schema).await;
        }

        Err(ErrorData::new(
            ErrorCode::INVALID_PARAMS,
            format!("Unknown tool '{name}'"),
            None,
        ))
    }
}
