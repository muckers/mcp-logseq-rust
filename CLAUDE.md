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

4. **models.rs**: Data structures for API communication
   - Request/response models
   - JSON serialization/deserialization

5. **tools/** module: MCP tool implementations
   - **query.rs**: Read operations (list_graphs, list_pages, get_page, get_block, search)
   - **mutate.rs**: Write operations (create_page, update_block, insert_block, delete_block, append_to_page)
   - **mod.rs**: Tool definitions and dispatch logic

### Communication Flow

1. MCP client sends JSON-RPC request via stdin
2. Server parses request and identifies the method
3. Request is routed to appropriate tool handler
4. Tool handler calls LogseqClient to interact with Logseq API
5. Response is serialized and sent back via stdout
6. Debug/error logging goes to stderr to avoid polluting JSON-RPC stream

### Key Design Patterns

- **Arc-wrapped client**: LogseqClient is wrapped in Arc for thread-safe sharing
- **Async/await**: Uses Tokio for async runtime
- **Error propagation**: Uses anyhow::Result throughout for consistent error handling
- **Separation of concerns**: Clear boundaries between protocol handling, business logic, and API communication

## Environment Setup

Create a `.env` file in the project root:
```env
LOGSEQ_API_TOKEN=your-actual-token-here
LOGSEQ_API_URL=http://localhost:12315
```

The `.env` file is gitignored by default for security.