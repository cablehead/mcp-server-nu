use assert_cmd::prelude::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

struct McpTestHarness {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout_reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpTestHarness {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("mcp-server-nu")?;
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdout_reader = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            stdout_reader,
            next_id: 1,
        })
    }

    fn send_request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let id = self.next_id;
        self.next_id += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        writeln!(self.stdin, "{}", serde_json::to_string(&request)?)?;
        self.stdin.flush()?;
        Ok(id)
    }

    fn send_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        writeln!(self.stdin, "{}", serde_json::to_string(&request)?)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut line = String::new();
        self.stdout_reader.read_line(&mut line)?;
        let response: Value = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    fn send_initialize(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )
    }

    fn send_initialized_notification(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_notification("notifications/initialized", json!({}))
    }

    fn send_tools_list(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        self.send_request("tools/list", json!({}))
    }

    fn send_tool_call(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )
    }

    fn assert_response_success(
        &mut self,
        expected_id: u64,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let response = self.read_response()?;
        assert_eq!(response["id"], expected_id, "Response ID mismatch");
        assert!(response["result"].is_object(), "Expected result object");
        Ok(response)
    }

    fn assert_response_error(
        &mut self,
        expected_id: u64,
        contains_text: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let response = self.read_response()?;
        assert_eq!(response["id"], expected_id, "Response ID mismatch");
        assert!(response["error"].is_object(), "Expected error object");
        let error_message = response["error"]["message"].as_str().unwrap_or("");
        assert!(
            error_message.contains(contains_text),
            "Error message '{error_message}' should contain '{contains_text}'"
        );
        Ok(response)
    }
}

impl Drop for McpTestHarness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn smoke_test_binary_exists() -> Result<(), Box<dyn std::error::Error>> {
    // Simple test to verify the binary can be found and executed
    Command::cargo_bin("mcp-server-nu")?;
    Ok(())
}

#[test]
fn test_mcp_protocol_complete_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = McpTestHarness::new()?;

    // Step 1: Send tools/list before initialize - should get error
    let premature_id = harness.send_tools_list()?;
    harness.assert_response_error(premature_id, "Server not initialized")?;

    // Step 2: Send proper initialize request
    let init_id = harness.send_initialize()?;
    let init_response = harness.assert_response_success(init_id)?;
    assert_eq!(init_response["result"]["protocolVersion"], "2024-11-05");

    // Step 3: Send initialized notification (no response expected)
    harness.send_initialized_notification()?;

    // Step 4: Send tools/list request (should work now)
    let tools_id = harness.send_tools_list()?;
    let tools_response = harness.assert_response_success(tools_id)?;

    // Verify the exec tool is present
    let tools = tools_response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Tools list should not be empty");

    let exec_tool = tools.iter().find(|tool| tool["name"] == "exec");
    assert!(exec_tool.is_some(), "Should have an 'exec' tool");

    let exec_tool = exec_tool.unwrap();
    assert!(exec_tool["description"].is_string());
    assert!(exec_tool["inputSchema"].is_object());

    // Verify timeout_seconds parameter is present
    let input_schema = &exec_tool["inputSchema"];
    assert!(
        input_schema["properties"]["timeout_seconds"].is_object(),
        "Should have timeout_seconds parameter"
    );

    println!("✓ Complete MCP protocol flow works correctly");
    Ok(())
}

#[test]
fn test_server_continues_after_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = McpTestHarness::new()?;

    // Step 1: Initialize the server
    let init_id = harness.send_initialize()?;
    harness.assert_response_success(init_id)?;

    // Step 2: Send initialized notification
    harness.send_initialized_notification()?;

    // Step 3: Call exec with a sleep command that will timeout (sleep 1s with 1s timeout)
    let exec_id = harness.send_tool_call(
        "exec",
        json!({
            "script": "sleep 1sec",
            "timeout_seconds": 1
        }),
    )?;

    // Step 4: Assert we get a timeout error response immediately
    harness.assert_response_error(exec_id, "timed out")?;

    // Step 5: Send 3 "ping" requests (tools/list) to verify server is still responsive
    for i in 1..=3 {
        let ping_id = harness.send_tools_list()?;
        let ping_response = harness.assert_response_success(ping_id)?;

        // Verify the response contains tools
        assert!(
            ping_response["result"]["tools"].is_array(),
            "Ping {i} should have tools array"
        );
        println!("✓ Ping {i} responded successfully");
    }

    println!("✓ Server continued processing after timeout - all 3 pings responded");
    Ok(())
}
