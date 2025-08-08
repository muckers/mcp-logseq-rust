//! # JSON-RPC Protocol Handler
//!
//! This module handles JSON-RPC 2.0 protocol communication for the MCP server.
//! It provides abstractions for request/response handling, error codes, and
//! protocol-specific logic, separating these concerns from business logic.

mod response;
pub use response::HandlerResponse;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// JSON-RPC 2.0 Request structure
#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    #[allow(dead_code)]
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Response builder for creating JSON-RPC responses
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Creates a successful response with the given result
    pub fn success(id: Value, result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Creates an error response
    pub fn error(id: Value, code: i32, message: String) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }

    /// Creates an error response with additional data
    #[allow(dead_code)]
    pub fn error_with_data(id: Value, code: i32, message: String, data: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: Some(data),
            }),
            id,
        }
    }

    /// Creates a method not found error
    #[allow(dead_code)]
    pub fn method_not_found(id: Value, method: &str) -> JsonRpcResponse {
        Self::error(
            id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method '{}' not found", method),
        )
    }

    /// Creates an internal error response
    #[allow(dead_code)]
    pub fn internal_error(id: Value, message: String) -> JsonRpcResponse {
        Self::error(id, error_codes::INTERNAL_ERROR, message)
    }

    /// Creates an invalid params error
    #[allow(dead_code)]
    pub fn invalid_params(id: Value, message: String) -> JsonRpcResponse {
        Self::error(id, error_codes::INVALID_PARAMS, message)
    }

    /// Creates a parse error response
    pub fn parse_error() -> JsonRpcResponse {
        Self::error(
            json!(0),
            error_codes::PARSE_ERROR,
            "Parse error".to_string(),
        )
    }

    /// Creates a notification acknowledgment (for internal use)
    /// This returns a special marker that indicates no response should be sent
    #[allow(dead_code)]
    pub fn notification_ack() -> Value {
        json!({"_skip_response": true})
    }
}

/// Parses a JSON-RPC request from a string
pub fn parse_request(input: &str) -> Result<JsonRpcRequest> {
    serde_json::from_str(input)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON-RPC request: {}", e))
}

/// Serializes a JSON-RPC response to a string
#[allow(dead_code)]
pub fn serialize_response(response: &JsonRpcResponse) -> Result<String> {
    serde_json::to_string(response)
        .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))
}

/// Checks if a request is a notification (no ID means notification)
#[allow(dead_code)]
pub fn is_notification(request: &JsonRpcRequest) -> bool {
    request.id.is_none()
}

/// Ensures we have a valid ID for responses (never null per JSON-RPC spec)
#[allow(dead_code)]
pub fn ensure_valid_id(id: Option<Value>) -> Value {
    id.unwrap_or(json!(0))
}