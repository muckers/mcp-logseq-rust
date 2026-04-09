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
mod protocol;
mod tools;

use anyhow::Result;
use serde_json::{Value, json};
use std::io::Write;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing_subscriber::EnvFilter;

use crate::{
    config::Config,
    logseq_client::LogseqClient,
    protocol::{HandlerResponse, JsonRpcRequest, ResponseBuilder, error_codes, parse_request},
    tools::{mutate, query},
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

    tracing::info!("MCP Logseq Server starting...");

    // Verify Logseq is reachable before entering the server loop
    if let Err(e) = client.get_current_graph().await {
        eprintln!(
            "[ERROR] Cannot connect to Logseq: {}. Is Logseq running with HTTP API enabled?",
            e
        );
        std::process::exit(1);
    }
    tracing::info!("Connected to Logseq successfully");

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
    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = std::io::stdout();
    let mut lines = stdin.lines();

    // Process each line from stdin as a separate JSON-RPC request
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        let request = match parse_request(&line) {
            Ok(req) => {
                tracing::debug!("Received request: method={}, id={:?}", req.method, req.id);
                req
            }
            Err(e) => {
                tracing::error!("Failed to parse JSON: {}", e);
                let error_response = ResponseBuilder::parse_error();
                let error_str = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", error_str)?;
                stdout.flush()?;
                continue;
            }
        };

        // Handle the request and generate a response
        let response = handle_request(request, &client).await;

        // Check if this is a notification (no response needed)
        if response.is_notification_ack() {
            tracing::debug!("Skipping response for notification");
            continue;
        }

        // Send response back to client via stdout
        let response_str = response
            .serialize()
            .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;
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
async fn handle_request(request: JsonRpcRequest, client: &Arc<LogseqClient>) -> HandlerResponse {
    // Resolve id for handlers that always respond. Notification handlers receive
    // the raw Option<Value> so they can detect and correctly silence notifications.
    let id = request.id.clone().unwrap_or(json!(0));
    let method = &request.method;

    match method.as_str() {
        "initialize" => handle_initialize(id),
        "initialized" => handle_initialized(request.id.clone()),
        "notifications/initialized" => handle_notifications_initialized(request.id.clone()),
        "ping" => handle_ping(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tool_call(id, request, client).await,
        _ => {
            tracing::debug!("Unknown method: {}", method);
            HandlerResponse::error(
                id,
                error_codes::METHOD_NOT_FOUND,
                format!("Method '{}' not found", method),
            )
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
/// - Server name and version
///
/// Note: Tools are NOT included here per MCP spec - they're returned via tools/list
fn handle_initialize(id: Value) -> HandlerResponse {
    let result = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "mcp-logseq-rust",
            "version": "1.0.0"
        }
    });

    HandlerResponse::success(id, result)
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
fn handle_initialized(id: Option<Value>) -> HandlerResponse {
    tracing::debug!("Received 'initialized' notification");

    // Per MCP spec: no id means this is a notification — do not respond
    match id {
        None => HandlerResponse::notification_ack(),
        Some(id) => HandlerResponse::success(id, json!({})),
    }
}

/// Handles ping requests for server health checking.
///
/// Returns an empty result object to indicate the server is alive and responsive.
/// This can be used by clients to verify the server is still operational.
fn handle_ping(id: Value) -> HandlerResponse {
    tracing::debug!("Received 'ping' request");
    HandlerResponse::success(id, json!({}))
}

/// Handles alternative initialization notification format.
///
/// Some MCP clients may use this method name instead of the standard
/// `initialized` notification. Returns an empty result for compatibility.
fn handle_notifications_initialized(_id: Option<Value>) -> HandlerResponse {
    tracing::debug!("Received 'notifications/initialized' notification");
    HandlerResponse::notification_ack()
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
fn handle_tools_list(id: Value) -> HandlerResponse {
    let tools = tools::get_all_tools();

    tracing::debug!("Handling tools/list request");

    let result = json!({
        "tools": tools.iter().map(|t| t.to_json()).collect::<Vec<_>>()
    });

    HandlerResponse::success(id, result)
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
async fn handle_tool_call(
    id: Value,
    request: JsonRpcRequest,
    client: &Arc<LogseqClient>,
) -> HandlerResponse {
    // Extract tool name and parameters from the MCP request format
    let params = match request.params {
        Some(ref p) => p,
        None => {
            return HandlerResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                "Missing params".to_string(),
            );
        }
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(name) => name,
        None => {
            return HandlerResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                "Missing tool name".to_string(),
            );
        }
    };

    let default_params = json!({});
    let tool_params = params.get("arguments").unwrap_or(&default_params);

    // Dispatch to the appropriate tool handler based on tool name
    let result = match tool_name {
        // Query tools
        "list_graphs" => query::list_graphs(client, tool_params.clone()).await,
        "list_pages" => query::list_pages(client, tool_params.clone()).await,
        "get_page" => query::get_page(client, tool_params.clone()).await,
        "get_block" => query::get_block(client, tool_params.clone()).await,
        "search" => query::search(client, tool_params.clone()).await,
        "query" => query::query(client, tool_params.clone()).await,
        "get_today_journal" => query::get_today_journal(client, tool_params.clone()).await,
        "get_page_references" => query::get_page_references(client, tool_params.clone()).await,
        "get_block_properties" => query::get_block_properties(client, tool_params.clone()).await,
        // Mutation tools
        "create_page" => mutate::create_page(client, tool_params.clone()).await,
        "update_block" => mutate::update_block(client, tool_params.clone()).await,
        "insert_block" => mutate::insert_block(client, tool_params.clone()).await,
        "delete_block" => mutate::delete_block(client, tool_params.clone()).await,
        "delete_page" => mutate::delete_page(client, tool_params.clone()).await,
        "append_to_page" => mutate::append_to_page(client, tool_params.clone()).await,
        "append_to_journal" => mutate::append_to_journal(client, tool_params.clone()).await,
        "set_block_property" => mutate::set_block_property(client, tool_params.clone()).await,
        "remove_block_property" => mutate::remove_block_property(client, tool_params.clone()).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    };

    // Format the response according to MCP protocol
    match result {
        Ok(tool_result) => {
            let text = match serde_json::to_string_pretty(&tool_result) {
                Ok(t) => t,
                Err(e) => {
                    return HandlerResponse::error(
                        id,
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to serialize result: {}", e),
                    );
                }
            };
            HandlerResponse::success(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }),
            )
        }
        Err(e) => HandlerResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            format!("Tool execution failed: {}", e),
        ),
    }
}
