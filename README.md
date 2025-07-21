# MCP Server Nu

A Model Context Protocol (MCP) server that provides Nushell script execution capabilities.

<img width="745" height="687" alt="image" src="https://github.com/user-attachments/assets/7df465b7-cbaf-47a2-9fa1-a2ab9c1f0fb3" />

⚠️ **This is an early sketch with no safety mechanisms. Do not use in production.**

## Features

- Execute Nushell scripts via MCP
- Returns stdout, stderr, and exit codes
- Supports full Nushell syntax and commands

## Usage

The server exposes one tool:

- **exec**: Executes a Nushell script and returns the output

## Testing

Try it out with the MCP inspector:

```bash
npx @modelcontextprotocol/inspector ./target/debug/mcp-server-nu
```
