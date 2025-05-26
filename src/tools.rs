use rust_mcp_schema::{schema_utils::CallToolError, CallToolResult};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    tool_box,
};
use serde_json::json;
use tokio::process::Command;

#[mcp_tool(
    name = "exec",
    description = "Executes a nushell script and returns stdout, stderr, and exit code",
    idempotent_hint = false,
    destructive_hint = true,
    open_world_hint = true,
    read_only_hint = false
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ExecTool {
    /// The nushell script to execute
    script: String,
}

impl ExecTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let output = Command::new("nu")
            .arg("-c")
            .arg(&self.script)
            .output()
            .await
            .map_err(|e| CallToolError::new(e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let result = json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code
        });

        Ok(CallToolResult::text_content(
            serde_json::to_string_pretty(&result).unwrap(),
            None,
        ))
    }
}

tool_box!(NuTools, [ExecTool]);