use async_trait::async_trait;
use rust_mcp_schema::{
    schema_utils::CallToolError, CallToolRequest, CallToolResult, ListToolsRequest,
    ListToolsResult, RpcError,
};
use rust_mcp_sdk::{mcp_server::ServerHandler, McpServer};
use tracing::{error, info};

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
        info!(
            "Received tool call request for tool: {}",
            request.params.name
        );

        let tool_params: NuTools = NuTools::try_from(request.params).map_err(|e| {
            error!("Failed to parse tool parameters: {}", e);
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid tool parameters: {e}"),
            ))
        })?;

        let result = match tool_params {
            NuTools::ExecTool(exec_tool) => match exec_tool.call_tool().await {
                Ok(result) => {
                    info!("Tool execution completed successfully");
                    Ok(result)
                }
                Err(e) => {
                    error!("Tool execution failed: {}", e);
                    Err(e)
                }
            },
        };

        result
    }
}
