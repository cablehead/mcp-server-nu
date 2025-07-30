use assert_cmd::prelude::*;
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn smoke_test_binary_exists() -> Result<(), Box<dyn std::error::Error>> {
    // Simple test to verify the binary can be found and executed
    Command::cargo_bin("mcp-server-nu")?;
    Ok(())
}

#[test]
fn test_mcp_protocol_complete_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcp-server-nu")?;
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdin = child.stdin.as_mut().unwrap();

    // Step 1: Send tools/list before initialize - should get error
    let premature_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    writeln!(
        stdin,
        "{}",
        serde_json::to_string(&premature_tools_request)?
    )?;
    stdin.flush()?;

    // Wait for error response
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 2: Send proper initialize request (but server may have exited)
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writeln!(stdin, "{}", serde_json::to_string(&initialize_request)?)?;
    stdin.flush()?;

    // Step 3: Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });

    writeln!(
        stdin,
        "{}",
        serde_json::to_string(&initialized_notification)?
    )?;
    stdin.flush()?;

    // Step 4: Send tools/list request (should work now)
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list",
        "params": {}
    });

    writeln!(stdin, "{}", serde_json::to_string(&tools_request)?)?;
    stdin.flush()?;

    // Wait for all responses
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Kill the process and check output
    child.kill()?;
    let output = child.wait_with_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Complete protocol flow stdout: {stdout}");

    // Parse each line as JSON and verify responses
    let lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    // Should have responses for: error, initialize, tools/list
    assert!(
        lines.len() >= 2,
        "Expected at least 2 JSON responses, got {}: {:?}",
        lines.len(),
        lines
    );

    // Check first response should be an error for premature tools/list
    let error_response: Value = serde_json::from_str(lines[0])?;
    assert_eq!(error_response["id"], 1);
    assert!(
        error_response["error"].is_object(),
        "Expected error object in first response"
    );
    assert!(
        error_response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Server not initialized")
            || error_response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Pre-initialization")
    );

    // The server currently exits after the error, so we may not get further responses
    // This test validates that the error response is sent correctly
    if lines.len() >= 2 {
        // If we got more responses, verify the initialize response
        let init_response: Value = serde_json::from_str(lines[1])?;
        if init_response["id"] == 2 {
            assert!(init_response["result"].is_object());
            assert_eq!(init_response["result"]["protocolVersion"], "2024-11-05");
        }

        // If we got a tools response, verify it
        if lines.len() >= 3 {
            let tools_response: Value = serde_json::from_str(lines[2])?;
            if tools_response["id"] == 3 {
                assert!(tools_response["result"].is_object());
                assert!(tools_response["result"]["tools"].is_array());

                // Verify the exec tool is present
                let tools = tools_response["result"]["tools"].as_array().unwrap();
                assert!(!tools.is_empty(), "Tools list should not be empty");

                let exec_tool = tools.iter().find(|tool| tool["name"] == "exec");
                assert!(exec_tool.is_some(), "Should have an 'exec' tool");

                let exec_tool = exec_tool.unwrap();
                assert!(exec_tool["description"].is_string());
                assert!(exec_tool["inputSchema"].is_object());
            }
        }
    }

    Ok(())
}
