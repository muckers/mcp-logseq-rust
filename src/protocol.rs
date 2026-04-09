//! # JSON-RPC Protocol Handler
//!
//! This module handles JSON-RPC 2.0 protocol communication for the MCP server.
//! It provides abstractions for request/response handling, error codes, and
//! protocol-specific logic, separating these concerns from business logic.

mod response;
pub use response::HandlerResponse;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    /// Creates a parse error response (id unknown, use 0 per spec)
    pub fn parse_error() -> JsonRpcResponse {
        use serde_json::json;
        Self::error(
            json!(0),
            error_codes::PARSE_ERROR,
            "Parse error".to_string(),
        )
    }
}

/// Parses a JSON-RPC request from a string
pub fn parse_request(input: &str) -> Result<JsonRpcRequest> {
    serde_json::from_str(input)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON-RPC request: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- parse_request tests ---

    #[test]
    fn test_parse_valid_request_with_id() {
        let input = r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#;
        let req = parse_request(input).unwrap();
        assert_eq!(req.method, "tools/list");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_parse_notification_has_no_id() {
        let input = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
        let req = parse_request(input).unwrap();
        assert_eq!(req.method, "initialized");
        assert!(req.id.is_none(), "Notification must have no id");
    }

    #[test]
    fn test_parse_invalid_json_returns_error() {
        let result = parse_request("not json at all");
        assert!(result.is_err());
    }

    // --- ResponseBuilder tests ---

    #[test]
    fn test_success_response_structure() {
        let resp = ResponseBuilder::success(json!(42), json!({"key": "value"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, json!(42));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response_has_correct_code() {
        let resp =
            ResponseBuilder::error(json!(1), error_codes::METHOD_NOT_FOUND, "Not found".into());
        assert!(resp.result.is_none());
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
    }

    #[test]
    fn test_parse_error_uses_id_zero() {
        let resp = ResponseBuilder::parse_error();
        assert_eq!(resp.id, json!(0));
        assert_eq!(resp.error.unwrap().code, error_codes::PARSE_ERROR);
    }

    // --- HandlerResponse notification detection ---

    #[test]
    fn test_notification_ack_is_detected() {
        let ack = HandlerResponse::notification_ack();
        assert!(ack.is_notification_ack());
    }

    #[test]
    fn test_success_response_is_not_notification_ack() {
        let resp = HandlerResponse::success(json!(1), json!({}));
        assert!(!resp.is_notification_ack());
    }

    // TODO(human): Add 2-4 more test cases covering edge cases you think are important.
    // Consider: string ids ("abc"), explicit null id, requests with params object,
    // malformed-but-valid JSON (missing method field), or additional error codes.
    // Pattern: #[test] fn test_your_case() { ... }
}
