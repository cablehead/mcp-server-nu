use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::Future;
use std::time::Duration;
use tokio::process::Command;
use tracing::{error, info, warn};

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExecRequest {
    /// The nushell script to execute.
    script: String,
}

#[derive(Clone)]
pub struct NuServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl NuServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = r"Executes a nushell script and returns stdout, stderr, and exit code.

EXECUTION PATTERN - CRITICAL:
Execute ONE command at a time that produces meaningful output. For complex tasks, break into sequential single commands like a human operator would. This allows:
- Seeing intermediate results before proceeding
- Easier error diagnosis and recovery
- Step-by-step validation of progress

AVOID: Long multi-step scripts that could fail midway
PREFER: Single commands with clear output, then assess and continue

IMPORTANT NUSHELL SYNTAX DIFFERENCES FROM POSIX:

Line Continuation:
- NO trailing backslashes (\). Use parentheses () for multi-line pipelines
- Example: (ls | where size > 1MB | get name)

Output/Printing:
- DON'T use 'echo' - it doesn't work like POSIX echo
- Use 'print' for side-effect output: print 'hello world'
- Use string literals for return values: 'return value'
- Use pipelines for processing: 'data' | filter | transform

Common Patterns:
- Filter: ls | where size > 1MB
- Transform: ls | get name
- Variables: let var = 'value'
- Return from pipeline: 'result' (not echo 'result')

Data Types:
- Nushell is structured data focused
- Commands return tables/records, not just text
- Use 'to csv' to convert structured data for LLM consumption
- Use 'to text' for plain text output"
    )]
    async fn exec(
        &self,
        Parameters(req): Parameters<ExecRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Executing nushell script: {}",
            req.script.chars().take(100).collect::<String>()
        );

        let timeout_duration = Duration::from_secs(30);
        let command_future = Command::new("nu").arg("-c").arg(&req.script).output();

        let output = match tokio::time::timeout(timeout_duration, command_future).await {
            Ok(result) => result.map_err(|e| {
                error!("Command execution failed: {}", e);
                McpError::internal_error(format!("Command execution failed: {e}"), None)
            })?,
            Err(_) => {
                warn!(
                    "Command timed out after {} seconds",
                    timeout_duration.as_secs()
                );
                return Err(McpError::internal_error(
                    format!("Command timed out after {} seconds. Consider breaking down complex scripts into smaller steps.",
                    timeout_duration.as_secs()),
                    None
                ));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        if exit_code != 0 {
            warn!(
                "Command exited with non-zero code: {}, stderr: {}",
                exit_code, stderr
            );
        } else {
            info!("Command completed successfully");
        }

        let result = json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for NuServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server executes Nushell scripts and returns their output. It provides structured data processing capabilities with proper error handling and timeouts.".to_string()),
        }
    }
}
