# MCP Server Nu

An MCP (Model Context Protocol) server that allows AI assistants to execute
Nushell scripts with full system access.

<img width="745" height="687" alt="image" src="https://github.com/user-attachments/assets/7df465b7-cbaf-47a2-9fa1-a2ab9c1f0fb3" />

## How it works

This server spawns `nu -c "<script>"` processes to execute commands. The nu
process runs with the same permissions as the server process, giving it access
to:

- File system (read/write/delete)
- Network (make requests, start servers)
- Environment variables
- System commands via nu's shell integration
- Any custom nu configurations you provide

**Security implications**: The AI can execute arbitrary code on your system.
Only use with trusted AI assistants and in environments where this level of
access is acceptable.

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

## Configuration

The server accepts optional CLI arguments to use custom nu config files:

```bash
mcp-server-nu --nu-config /path/to/config.nu --nu-env-config /path/to/env.nu
```

**Options:**

- `--nu-config <path>`: Custom config.nu file (sets up commands, aliases, etc.)
- `--nu-env-config <path>`: Custom env.nu file (sets up environment variables)

When provided, these configs are loaded for every script execution via
`nu --config <path> --env-config <path> -c "<script>"`.

## Install & Test

```bash
cargo install --locked mcp-server-nu
npx @modelcontextprotocol/inspector mcp-server-nu
```

**With custom config:**

```bash
npx @modelcontextprotocol/inspector -- mcp-server-nu --nu-config /path/to/config.nu
```
