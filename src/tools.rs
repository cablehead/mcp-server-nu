use rust_mcp_schema::{schema_utils::CallToolError, CallToolResult};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    tool_box,
};
use serde_json::json;
use tokio::process::Command;

#[mcp_tool(
    name = "exec",
    description = r"Executes a nushell script and returns stdout, stderr, and exit code.

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
- Use 'to text' for plain text output",
    idempotent_hint = false,
    destructive_hint = true,
    open_world_hint = true,
    read_only_hint = false
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ExecTool {
    /// The nushell script to execute.
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
