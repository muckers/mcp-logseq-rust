//! # Configuration Management
//!
//! This module handles loading and managing configuration for the MCP Logseq server.
//! Configuration is loaded from environment variables, with support for `.env` files
//! for local development.
//!
//! ## Required Configuration
//!
//! - `LOGSEQ_API_TOKEN`: Authentication token for Logseq HTTP API
//! - `LOGSEQ_API_URL`: Base URL for Logseq API (defaults to localhost:12315)
//!
//! ## Environment Setup
//!
//! The server loads configuration from environment variables, with automatic
//! support for `.env` files in the working directory. This allows for easy
//! local development while supporting production deployment patterns.

use anyhow::Result;
use serde::Deserialize;

/// Configuration structure for the MCP Logseq server.
///
/// Contains all the settings needed to connect to and authenticate with
/// a Logseq instance via its HTTP API. Configuration values are loaded
/// from environment variables during server startup.
/// Default request timeout (seconds) for calls to the Logseq HTTP API.
const DEFAULT_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Bearer token for authenticating with the Logseq HTTP API
    pub logseq_api_token: String,
    /// Base URL for the Logseq HTTP API endpoint (no trailing slash)
    pub logseq_api_url: String,
    /// Request timeout, in seconds, for API calls
    pub timeout_secs: u64,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Attempts to load a `.env` file from the current directory first,
    /// then reads configuration from environment variables. This allows
    /// for flexible deployment while supporting local development.
    ///
    /// # Environment Variables
    ///
    /// - `LOGSEQ_API_TOKEN` (required): Bearer token for API authentication
    /// - `LOGSEQ_API_URL` (optional): API base URL, defaults to localhost:12315
    ///
    /// # Returns
    ///
    /// A configured `Config` instance, or an error if required variables
    /// are missing or invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if `LOGSEQ_API_TOKEN` is not set, as this is
    /// required for API authentication.
    pub fn from_env() -> Result<Self> {
        // Load .env file if present (ignore if it doesn't exist)
        dotenvy::dotenv().ok();

        let logseq_api_token = std::env::var("LOGSEQ_API_TOKEN")
            .map_err(|_| anyhow::anyhow!("LOGSEQ_API_TOKEN not set"))?;

        // Default to standard Logseq HTTP API port on localhost.
        // Trim any trailing slash so building "{url}/api" never double-slashes.
        let logseq_api_url = std::env::var("LOGSEQ_API_URL")
            .unwrap_or_else(|_| "http://localhost:12315".to_string())
            .trim_end_matches('/')
            .to_string();

        // Optional timeout override; fall back to the default on missing/invalid values.
        let timeout_secs = std::env::var("LOGSEQ_API_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        Ok(Config {
            logseq_api_token,
            logseq_api_url,
            timeout_secs,
        })
    }
}
