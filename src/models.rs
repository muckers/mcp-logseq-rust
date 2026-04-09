//! # Data Models
//!
//! Request structure for the Logseq HTTP API.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request structure for calling Logseq HTTP API methods.
///
/// The Logseq API uses a method-based approach where each request specifies
/// a method name and an array of arguments.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogseqApiRequest {
    /// The API method to invoke (e.g., "logseq.Editor.getPage")
    pub method: String,
    /// Arguments to pass to the method as a JSON array
    pub args: Vec<Value>,
}
