# MCP Server Nu

A Model Context Protocol (MCP) server that provides Nushell script execution capabilities.

## Features

- Execute Nushell scripts remotely via MCP
- Returns stdout, stderr, and exit codes
- Supports full Nushell syntax and commands

## Usage

The server exposes one tool:

- **exec**: Executes a Nushell script and returns the output

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Testing

Try it out with the MCP inspector:

```bash
npx @modelcontextprotocol/inspector ./target/debug/mcp-server-nu
```

## Requirements

- Rust
- Nushell (`nu`) installed and available in PATH