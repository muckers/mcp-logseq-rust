//! # Error Types
//!
//! Domain-specific error types for better error handling and debugging.
//! These errors provide context about what went wrong and where.

use thiserror::Error;

/// Main error type for the MCP Logseq server
#[derive(Error, Debug)]
pub enum McpError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Logseq API errors
    #[error("Logseq API error: {0}")]
    LogseqApi(String),

    /// Protocol-level errors (JSON-RPC)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Tool execution errors
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Parameter validation errors
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Generic errors (for compatibility with anyhow)
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::Other(err.to_string())
    }
}

/// Result type alias using our custom error
pub type McpResult<T> = Result<T, McpError>;

/// Helper functions for creating specific errors
impl McpError {
    pub fn config(msg: impl Into<String>) -> Self {
        McpError::Config(msg.into())
    }

    pub fn logseq_api(msg: impl Into<String>) -> Self {
        McpError::LogseqApi(msg.into())
    }

    pub fn protocol(msg: impl Into<String>) -> Self {
        McpError::Protocol(msg.into())
    }

    pub fn tool_execution(msg: impl Into<String>) -> Self {
        McpError::ToolExecution(msg.into())
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        McpError::InvalidParams(msg.into())
    }
}