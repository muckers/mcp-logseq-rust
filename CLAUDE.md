# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of an MCP (Model Context Protocol) server for Logseq, enabling AI assistants to interact with Logseq knowledge graphs through Logseq's Local HTTP API. The server communicates via JSON-RPC over stdin/stdout and provides both query and mutation operations for Logseq graph data.

## Build and Development Commands

```bash
# Build the project
cargo build --release

# Run the server
cargo run --release
# OR use the compiled binary:
./target/release/mcp-logseq-rust

# Development build and run
cargo build
cargo run

# Code formatting
cargo fmt

# Linting
cargo clippy

# Check for compilation errors without building
cargo check

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests (when available)
cargo test
# Run a specific test
cargo test test_name
```

## Architecture

### Core Components

The server follows a modular architecture with clear separation of concerns:

1. **main.rs**: Entry point and JSON-RPC server loop
   - Handles stdin/stdout communication for MCP protocol
   - Routes requests to appropriate handlers
   - Manages the request-response cycle

2. **config.rs**: Environment-based configuration
   - Loads from `.env` file or environment variables
   - Required: `LOGSEQ_API_TOKEN`
   - Optional: `LOGSEQ_API_URL` (defaults to http://localhost:12315)

3. **logseq_client.rs**: HTTP client wrapper for Logseq API
   - Handles authentication via Bearer token
   - Manages API request formatting and response parsing
   - All API calls go through `call_api` method

4. **protocol/** module: JSON-RPC 2.0 protocol handling
   - Request/response structures
   - Error codes and response builders
   - Protocol-specific logic separated from business logic

5. **models.rs**: Data structures for API communication
   - Logseq API request/response models
   - Entity models (Page, Block, Graph)
   - JSON serialization/deserialization

6. **error.rs**: Custom error types for the application

7. **tools/** module: MCP tool implementations
   - **mod.rs**: Tool definitions using builder pattern
   - **builder.rs**: Helper utilities for defining tool schemas
   - **query.rs**: Read operations (list_graphs, list_pages, get_page, get_block, search)
   - **mutate.rs**: Write operations (create_page, update_block, insert_block, delete_block, append_to_page)

### Communication Flow

1. MCP client sends JSON-RPC 2.0 request via stdin (one request per line)
2. Server parses request in `run_mcp_server` loop (main.rs:84)
3. Request is dispatched to `handle_request` which routes to appropriate handler (main.rs:149)
4. MCP protocol methods (initialize, tools/list, tools/call, etc.) are handled
5. For tool execution, request is routed to appropriate tool handler in query.rs or mutate.rs
6. Tool handler calls LogseqClient to interact with Logseq API via HTTP
7. Response is serialized and sent back via stdout
8. Debug/error logging goes to stderr to avoid polluting JSON-RPC stream

### MCP Protocol Methods

The server implements these standard MCP methods:
- `initialize`: Server capability negotiation and tool listing
- `initialized`/`notifications/initialized`: Initialization confirmation
- `ping`: Health check
- `tools/list`: List available tools with schemas
- `tools/call`: Execute a specific tool by name

### Key Design Patterns

- **Arc-wrapped client**: LogseqClient is wrapped in Arc for thread-safe sharing across handlers
- **Async/await**: Uses Tokio for async runtime throughout
- **Error propagation**: Uses anyhow::Result throughout for consistent error handling
- **Protocol separation**: JSON-RPC protocol logic (protocol/) is cleanly separated from business logic
- **Builder pattern**: Tool definitions use a builder pattern (tools/builder.rs) for clean, declarative schemas
- **Separation of concerns**: Clear boundaries between protocol handling, business logic, and API communication

## Adding New Tools

To add a new tool to the MCP server:

1. Add the tool definition in `tools/mod.rs` using `ToolBuilder`:
   ```rust
   ToolBuilder::new("tool_name")
       .description("What the tool does")
       .string_param("param1", "Description", required: bool)
       .build()
   ```

2. Implement the handler in either `tools/query.rs` (read-only) or `tools/mutate.rs` (write operations):
   ```rust
   pub async fn tool_name(client: &Arc<LogseqClient>, params: Value) -> Result<Value>
   ```

3. Add the dispatch case in `main.rs` in the `handle_tool_call` function (main.rs:330)

4. If needed, add new methods to LogseqClient in `logseq_client.rs`

## Environment Setup

### Prerequisites
Before running the server, ensure Logseq's HTTP API is enabled:
1. Open Logseq → Settings → Features
2. Toggle on "HTTP APIs server"
3. Click the API option in the top menu and set up an authorization token
4. Enable "Auto start server when Logseq launches"
5. Default API URL is http://localhost:12315

### Configuration
Create a `.env` file in the project root:
```env
LOGSEQ_API_TOKEN=your-actual-token-here
LOGSEQ_API_URL=http://localhost:12315
```

The `.env` file is gitignored by default for security.