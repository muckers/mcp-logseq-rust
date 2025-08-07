//! # Response handling utilities
//!
//! Provides a unified response type that can represent either a
//! JSON-RPC response or a notification acknowledgment.

use super::{JsonRpcResponse, ResponseBuilder};
use serde_json::Value;

/// Unified response type for handler functions
pub enum HandlerResponse {
    /// A normal JSON-RPC response to be sent
    Response(JsonRpcResponse),
    /// Acknowledgment that a notification was processed (no response sent)
    NotificationAck,
}

impl HandlerResponse {
    /// Creates a success response
    pub fn success(id: Value, result: Value) -> Self {
        HandlerResponse::Response(ResponseBuilder::success(id, result))
    }

    /// Creates an error response
    pub fn error(id: Value, code: i32, message: String) -> Self {
        HandlerResponse::Response(ResponseBuilder::error(id, code, message))
    }

    /// Creates a notification acknowledgment
    pub fn notification_ack() -> Self {
        HandlerResponse::NotificationAck
    }

    /// Checks if this is a notification acknowledgment
    pub fn is_notification_ack(&self) -> bool {
        matches!(self, HandlerResponse::NotificationAck)
    }

    /// Converts to string for output
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        match self {
            HandlerResponse::Response(resp) => serde_json::to_string(resp),
            HandlerResponse::NotificationAck => Ok(String::new()),
        }
    }
}