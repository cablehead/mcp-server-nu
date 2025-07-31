# MCP Server Nu

MCP server for executing Nushell scripts.

<img width="745" height="687" alt="image" src="https://github.com/user-attachments/assets/7df465b7-cbaf-47a2-9fa1-a2ab9c1f0fb3" />

⚠️ **No safety mechanisms. Do not use in production.**

## Tool: exec

Executes Nushell scripts and returns stdout, stderr, exit code.

**Parameters:**

- `script` (required): Nushell script to execute
- `timeout_seconds` (optional): Timeout in seconds (default: 30)

**Example:**

```json
{
  "script": "ls | where size > 1MB | get name",
  "timeout_seconds": 10
}
```

## Install & Test

```bash
cargo install --locked mcp-server-nu
npx @modelcontextprotocol/inspector mcp-server-nu
```
