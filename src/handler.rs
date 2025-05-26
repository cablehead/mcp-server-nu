use async_trait::async_trait;
use rust_mcp_schema::{
    schema_utils::CallToolError, CallToolRequest, CallToolResult, ListToolsRequest,
    ListToolsResult, RpcError,
};
use rust_mcp_sdk::{mcp_server::ServerHandler, McpServer};

use crate::tools::NuTools;

pub struct NuServerHandler;

#[async_trait]
#[allow(unused)]
impl ServerHandler for NuServerHandler {
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: NuTools::tools(),
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_params: NuTools =
            NuTools::try_from(request.params).map_err(CallToolError::new)?;

        match tool_params {
            NuTools::ExecTool(exec_tool) => exec_tool.call_tool().await,
        }
    }
}