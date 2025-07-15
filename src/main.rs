mod config;
mod logseq_client;
mod models;
mod tools;

use anyhow::Result;
use mcpr::{
    server::{Server, ServerConfig},
    transport::stdio::StdioTransport,
    error::MCPError,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::{
    config::Config,
    logseq_client::LogseqClient,
    tools::{query, mutate},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .init();

    info!("Starting MCP Logseq Server");

    // Load configuration
    let config = Config::from_env()?;
    let client = Arc::new(LogseqClient::new(config)?);
    
    // Configure the server
    let server_config = ServerConfig::new()
        .with_name("Logseq MCP Server")
        .with_version("0.1.0");
    
    // Add all tools to the server config
    let mut server_config = server_config;
    for tool in tools::get_all_tools() {
        server_config = server_config.with_tool(tool);
    }
    
    // Create the server
    let mut server = Server::new(server_config);
    
    // Register tool handlers
    register_tool_handlers(&mut server, client)?;
    
    // Start the server with stdio transport
    info!("Server configured, starting stdio transport");
    let transport = StdioTransport::new();
    
    if let Err(e) = server.start(transport) {
        error!("Server error: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}

// Helper macro to register async handlers
macro_rules! register_async_handler {
    ($server:expr, $client:expr, $name:expr, $handler:path) => {
        {
            let client_clone = $client.clone();
            $server.register_tool_handler($name, move |params| {
                let client = client_clone.clone();
                let rt = tokio::runtime::Runtime::new().map_err(|e| MCPError::Protocol(e.to_string()))?;
                rt.block_on(async {
                    $handler(&client, params).await
                        .map_err(|e| MCPError::Protocol(e.to_string()))
                })
            })
        }
    };
}

fn register_tool_handlers(server: &mut Server<StdioTransport>, client: Arc<LogseqClient>) -> Result<()> {
    // Query handlers
    register_async_handler!(server, client, "list_graphs", query::list_graphs)?;
    register_async_handler!(server, client, "list_pages", query::list_pages)?;
    register_async_handler!(server, client, "get_page", query::get_page)?;
    register_async_handler!(server, client, "get_block", query::get_block)?;
    register_async_handler!(server, client, "search", query::search)?;
    
    // Write handlers
    register_async_handler!(server, client, "create_page", mutate::create_page)?;
    register_async_handler!(server, client, "update_block", mutate::update_block)?;
    register_async_handler!(server, client, "insert_block", mutate::insert_block)?;
    register_async_handler!(server, client, "delete_block", mutate::delete_block)?;
    register_async_handler!(server, client, "append_to_page", mutate::append_to_page)?;
    
    Ok(())
}