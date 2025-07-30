use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn smoke_test_binary_exists() -> Result<(), Box<dyn std::error::Error>> {
    // Simple test to verify the binary can be found and executed
    Command::cargo_bin("mcp-server-nu")?;
    Ok(())
}
