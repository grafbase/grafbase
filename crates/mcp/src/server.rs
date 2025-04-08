use std::sync::Arc;

use crate::{
    EngineWatcher,
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

#[derive(Clone)]
pub(crate) struct McpServer(Arc<McpServerInner>);

pub(crate) struct McpServerInner {
    info: ServerInfo,
    tools: Vec<Box<dyn RmcpTool>>,
}

impl std::ops::Deref for McpServer {
    type Target = McpServerInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl McpServer {
    pub fn new(engine: EngineWatcher<impl engine::Runtime>, include_mutations: bool) -> anyhow::Result<Self> {
        Ok(Self(Arc::new(McpServerInner {
            info: ServerInfo {
                protocol_version: ProtocolVersion::V_2024_11_05,
                capabilities: ServerCapabilities::builder().enable_tools().build(),
                server_info: Implementation::from_build_env(),
                instructions: None,
            },
            tools: vec![
                Box::new(IntrospectTool::new(&engine, include_mutations)),
                Box::new(SearchTool::new(&engine, include_mutations)?),
                Box::new(ExecuteTool::new(&engine, include_mutations)),
            ],
        })))
    }
}

impl ServerHandler for McpServer {
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
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if let Some(tool) = self.tools.iter().find(|tool| tool.name() == name) {
            return tool.call(arguments).await;
        }

        Err(ErrorData::new(
            ErrorCode::INVALID_PARAMS,
            format!("Unknown tool '{name}'"),
            None,
        ))
    }
}
