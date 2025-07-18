//! # MCP Logseq Server
//!
//! A Model Context Protocol (MCP) server implementation for Logseq integration.
//! This server provides an interface for external tools to interact with Logseq
//! through its HTTP API, enabling operations like reading pages, searching content,
//! creating and modifying blocks, and managing graph data.
//!
//! ## Protocol
//! 
//! The server implements the MCP protocol over JSON-RPC via stdin/stdout,
//! allowing it to be used by MCP-compatible clients like Claude Desktop,
//! IDEs, and other development tools.
//!
//! ## Features
//!
//! - Query operations: list graphs, pages, get blocks, search
//! - Mutation operations: create pages, update/insert/delete blocks
//! - Real-time communication via stdin/stdout JSON-RPC
//! - Error handling with graceful degradation
//! - Configurable via environment variables

mod config;
mod logseq_client;
mod models;
mod tools;

use anyhow::Result;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use crate::{
    config::Config,
    logseq_client::LogseqClient,
    tools::{query, mutate},
};

/// Main entry point for the MCP Logseq server.
/// 
/// Sets up logging, loads configuration from environment variables,
/// initializes the Logseq client, and starts the MCP server loop
/// that handles JSON-RPC requests from stdin.
#[tokio::main]
async fn main() -> Result<()> {
    // Set up stderr logging for debugging (won't pollute stdout)
    // This ensures debug output doesn't interfere with JSON-RPC communication
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .with_target(false)
        .with_level(true)
        .with_writer(std::io::stderr)
        .init();

    // Load configuration from environment variables
    let config = Config::from_env()?;
    let client = Arc::new(LogseqClient::new(config)?);
    
    eprintln!("[INFO] MCP Logseq Server starting...");
    
    // Start the MCP server loop
    run_mcp_server(client).await?;
    
    Ok(())
}

/// Main MCP server loop that handles JSON-RPC communication.
///
/// Reads JSON-RPC requests from stdin line by line, processes each request
/// through the request handler, and writes responses to stdout. This follows
/// the MCP protocol specification for server communication.
///
/// ## Protocol Details
///
/// - Each request is a single line of JSON
/// - Empty lines are ignored
/// - Responses are written immediately after processing
/// - Notifications (requests without IDs) may not generate responses
/// - All errors are logged to stderr to avoid polluting the JSON-RPC stream
async fn run_mcp_server(client: Arc<LogseqClient>) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // Process each line from stdin as a separate JSON-RPC request
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        let request: Value = match serde_json::from_str(&line) {
            Ok(req) => {
                eprintln!("[DEBUG] Received request: {}", serde_json::to_string(&req).unwrap_or_else(|_| "invalid".to_string()));
                req
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to parse JSON: {} - Line: {}", e, line);
                continue;
            }
        };
        
        // Handle the request and generate a response
        let response = handle_request(request, &client).await?;
        
        // Check if we should skip the response (for notifications)
        // Notifications don't require responses per JSON-RPC spec
        if let Some(skip) = response.get("_skip_response") {
            if skip.as_bool().unwrap_or(false) {
                eprintln!("[DEBUG] Skipping response for notification");
                continue;
            }
        }
        
        // Send response back to client via stdout
        let response_str = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }
    
    Ok(())
}

/// Central request handler that routes JSON-RPC requests to appropriate handlers.
///
/// Extracts the method name from the request and dispatches to the corresponding
/// handler function. Implements the MCP protocol's core methods including
/// initialization, tool listing, and tool execution.
///
/// ## Supported Methods
///
/// - `initialize`: Server capability negotiation
/// - `initialized`: Initialization confirmation
/// - `notifications/initialized`: Alternative initialization notification
/// - `ping`: Health check
/// - `tools/list`: List available tools
/// - `tools/call`: Execute a specific tool
///
/// ## Error Handling
///
/// Unknown methods return a JSON-RPC error with code -32601 (Method not found).
/// The ID is preserved from the request, or defaults to 0 for malformed requests.
async fn handle_request(request: Value, client: &Arc<LogseqClient>) -> Result<Value> {
    // Ensure we always have a valid ID - never use null per JSON-RPC spec
    let id = request.get("id").cloned().unwrap_or(json!(0));
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    
    match method {
        "initialize" => handle_initialize(id),
        "initialized" => handle_initialized(id),
        "notifications/initialized" => handle_notifications_initialized(id),
        "ping" => handle_ping(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tool_call(id, request, client).await,
        _ => {
            eprintln!("[DEBUG] Unknown method: {}", method);
            Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method '{}' not found", method)
                }
            }))
        }
    }
}

/// Handles the MCP `initialize` request.
///
/// This is the first method called during the MCP handshake. It returns
/// server capabilities, protocol version, and the list of available tools.
/// The client uses this information to understand what the server can do.
///
/// ## Response Format
///
/// Returns server info including:
/// - Protocol version (2024-11-05)
/// - Server capabilities (tools support)
/// - List of all available tools with their schemas
fn handle_initialize(id: Value) -> Result<Value> {
    let tools = tools::get_all_tools();
    
    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "logseq": true
            },
            "serverInfo": {
                "name": "mcp-logseq-rust",
                "version": "1.0.0"
            },
            "tools": tools.iter().map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description.as_ref().unwrap_or(&"".to_string()),
                    "inputSchema": {
                        "type": tool.input_schema.r#type,
                        "properties": tool.input_schema.properties.as_ref().unwrap_or(&std::collections::HashMap::new()),
                        "required": tool.input_schema.required.as_ref().unwrap_or(&Vec::new())
                    }
                })
            }).collect::<Vec<_>>()
        }
    });
    
    Ok(response)
}

/// Handles the MCP `initialized` notification.
///
/// This method is called after the client has processed the `initialize` response
/// and is ready to begin normal operation. Per the MCP specification, this is
/// typically a notification (no response expected), but some clients may send
/// it as a request requiring acknowledgment.
///
/// ## Behavior
///
/// - If sent as notification (id is null): returns skip marker
/// - If sent as request (with id): returns empty result object
fn handle_initialized(id: Value) -> Result<Value> {
    eprintln!("[DEBUG] Received 'initialized' notification");
    
    // If it's a notification (no id), don't send a response per MCP spec
    if id.is_null() {
        eprintln!("[DEBUG] Skipping response for notification");
        // Return a special marker that we'll check for in the main loop
        return Ok(json!({"_skip_response": true}));
    }
    
    // Some clients send this as a request, so acknowledge with empty result
    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {}
    }))
}

/// Handles ping requests for server health checking.
///
/// Returns an empty result object to indicate the server is alive and responsive.
/// This can be used by clients to verify the server is still operational.
fn handle_ping(id: Value) -> Result<Value> {
    eprintln!("[DEBUG] Received 'ping' request");
    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {}
    }))
}

/// Handles alternative initialization notification format.
///
/// Some MCP clients may use this method name instead of the standard
/// `initialized` notification. Returns an empty result for compatibility.
fn handle_notifications_initialized(id: Value) -> Result<Value> {
    eprintln!("[DEBUG] Received 'notifications/initialized' request");
    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {}
    }))
}

/// Handles the MCP `tools/list` request.
///
/// Returns a list of all available tools that clients can invoke.
/// Each tool includes its name, description, and input schema definition
/// which describes the expected parameters and their types.
///
/// ## Response Format
///
/// Returns a `tools` array where each tool contains:
/// - `name`: Unique identifier for the tool
/// - `description`: Human-readable description of what the tool does
/// - `inputSchema`: JSON Schema defining expected parameters
fn handle_tools_list(id: Value) -> Result<Value> {
    let tools = tools::get_all_tools();
    
    eprintln!("[DEBUG] Handling tools/list request");
    
    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tools.iter().map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description.as_ref().unwrap_or(&"".to_string()),
                    "inputSchema": {
                        "type": tool.input_schema.r#type,
                        "properties": tool.input_schema.properties.as_ref().unwrap_or(&std::collections::HashMap::new()),
                        "required": tool.input_schema.required.as_ref().unwrap_or(&Vec::new())
                    }
                })
            }).collect::<Vec<_>>()
        }
    }))
}

/// Handles the MCP `tools/call` request to execute a specific tool.
///
/// Extracts the tool name and parameters from the request, dispatches to the
/// appropriate tool handler function, and returns the formatted result.
/// All tool results are wrapped in MCP's standard content format.
///
/// ## Request Format
///
/// Expects:
/// - `params.name`: The name of the tool to execute
/// - `params.arguments`: Object containing tool-specific parameters
///
/// ## Response Format
///
/// Success responses contain:
/// - `result.content`: Array with tool output as formatted text
///
/// Error responses contain:
/// - `error.code`: -32603 (Internal error)
/// - `error.message`: Description of what went wrong
///
/// ## Supported Tools
///
/// Query tools: list_graphs, list_pages, get_page, get_block, search
/// Mutation tools: create_page, update_block, insert_block, delete_block, append_to_page
async fn handle_tool_call(id: Value, request: Value, client: &Arc<LogseqClient>) -> Result<Value> {
    // Extract tool name and parameters from the MCP request format
    let params = request.get("params").ok_or_else(|| anyhow::anyhow!("Missing params"))?;
    let tool_name = params.get("name").and_then(|n| n.as_str()).ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let default_params = json!({});
    let tool_params = params.get("arguments").unwrap_or(&default_params);
    
    // Dispatch to the appropriate tool handler based on tool name
    let result = match tool_name {
        "list_graphs" => query::list_graphs(client, tool_params.clone()).await,
        "list_pages" => query::list_pages(client, tool_params.clone()).await,
        "get_page" => query::get_page(client, tool_params.clone()).await,
        "get_block" => query::get_block(client, tool_params.clone()).await,
        "search" => query::search(client, tool_params.clone()).await,
        "create_page" => mutate::create_page(client, tool_params.clone()).await,
        "update_block" => mutate::update_block(client, tool_params.clone()).await,
        "insert_block" => mutate::insert_block(client, tool_params.clone()).await,
        "delete_block" => mutate::delete_block(client, tool_params.clone()).await,
        "append_to_page" => mutate::append_to_page(client, tool_params.clone()).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name))
    };
    
    // Format the response according to MCP protocol
    match result {
        Ok(tool_result) => Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&tool_result)?
                }]
            }
        })),
        Err(e) => Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("Tool execution failed: {}", e)
            }
        }))
    }
}

