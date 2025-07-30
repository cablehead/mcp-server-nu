use rust_mcp_schema::{schema_utils::CallToolError, CallToolResult};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    tool_box,
};
use serde_json::json;
use std::time::Duration;
use tokio::process::Command;
use tracing::{error, info, warn};

#[mcp_tool(
    name = "exec",
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
        info!(
            "Executing nushell script: {}",
            self.script.chars().take(100).collect::<String>()
        );

        let timeout_duration = Duration::from_secs(30);
        let command_future = Command::new("nu").arg("-c").arg(&self.script).output();

        let output = match tokio::time::timeout(timeout_duration, command_future).await {
            Ok(result) => result.map_err(|e| {
                error!("Command execution failed: {}", e);
                CallToolError::new(e)
            })?,
            Err(_) => {
                warn!(
                    "Command timed out after {} seconds",
                    timeout_duration.as_secs()
                );
                return Err(CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "Command timed out after {} seconds. Consider breaking down complex scripts into smaller steps.",
                        timeout_duration.as_secs()
                    )
                )));
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

        Ok(CallToolResult::text_content(
            serde_json::to_string_pretty(&result).unwrap(),
            None,
        ))
    }
}

tool_box!(NuTools, [ExecTool]);
