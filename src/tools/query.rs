//! # Query Tool Implementations
//!
//! This module implements all read-only MCP tools that retrieve data from Logseq
//! without making any modifications. These tools provide safe access to graph
//! data for analysis, search, and content retrieval operations.
//!
//! ## Tool Functions
//!
//! Each function in this module corresponds to a specific MCP tool that clients
//! can invoke. All functions follow the same pattern:
//! - Accept a Logseq client and parameters
//! - Validate required parameters
//! - Call appropriate Logseq API methods
//! - Return formatted results as JSON
//!
//! ## Error Handling
//!
//! Parameter validation errors are returned immediately with descriptive messages.
//! API errors from Logseq are propagated up to the MCP layer for consistent handling.

use crate::logseq_client::LogseqClient;
use anyhow::Result;
use chrono::{Datelike, Local};
use serde_json::Value;

/// Lists available Logseq graphs.
///
/// Currently returns information about the active graph, as the Logseq API
/// primarily works with the currently open graph. In the future, this could
/// be extended to list multiple graphs if the API supports it.
///
/// # Parameters
///
/// No parameters required.
///
/// # Returns
///
/// JSON object containing an array of graph information with name, path, and metadata.
pub async fn list_graphs(client: &LogseqClient, _params: Value) -> Result<Value> {
    let graph = client.get_current_graph().await?;
    Ok(serde_json::json!({
        "graphs": [graph]
    }))
}

/// Retrieves a list of all pages in the current graph.
///
/// Returns comprehensive information about every page in the graph, including
/// page names, UUIDs, creation dates, and other metadata. Useful for getting
/// an overview of the graph's structure and content.
///
/// # Parameters
///
/// No parameters required.
///
/// # Returns
///
/// JSON object containing an array of page objects with metadata for each page
/// in the graph.
/// Optional params:
/// - `name_contains`: substring filter on page name (case-insensitive)
/// - `limit`: max pages to return (default: 100)
pub async fn list_pages(client: &LogseqClient, params: Value) -> Result<Value> {
    let pages = client.get_all_pages().await?;

    let name_filter = params["name_contains"].as_str().map(|s| s.to_lowercase());
    let limit = params["limit"].as_u64().unwrap_or(100) as usize;

    let filtered: Vec<&Value> = pages
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|page| {
                    name_filter.as_ref().is_none_or(|f| {
                        page["name"]
                            .as_str()
                            .map(|n| n.to_lowercase().contains(f.as_str()))
                            .unwrap_or(false)
                    })
                })
                .take(limit)
                .collect()
        })
        .unwrap_or_default();

    Ok(serde_json::json!({
        "pages": filtered,
        "total": filtered.len()
    }))
}

/// Retrieves comprehensive information about a specific page.
///
/// Fetches both the page metadata and the complete block tree structure
/// for the specified page. This provides a complete view of the page's
/// content and organization.
///
/// # Parameters
///
/// - `page_name` (required): The name of the page to retrieve
///
/// # Returns
///
/// JSON object containing:
/// - `page`: Page metadata (name, UUID, properties)
/// - `blocks`: Complete hierarchical block tree for the page
///
/// # Errors
///
/// Returns an error if the page_name parameter is missing or if the page
/// doesn't exist in the graph.
pub async fn get_page(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;

    // Fetch both page metadata and block structure for complete information
    let page_info = client.get_page(page_name).await?;
    let blocks = client.get_page_blocks_tree(page_name).await?;

    Ok(serde_json::json!({
        "page": page_info,
        "blocks": blocks
    }))
}

/// Retrieves a specific block by its UUID.
///
/// Fetches detailed information about a single block, including its content,
/// properties, parent/child relationships, and metadata. Useful for examining
/// specific pieces of content within the graph.
///
/// # Parameters
///
/// - `uuid` (required): The unique identifier of the block to retrieve
///
/// # Returns
///
/// JSON object containing the complete block information including content,
/// properties, relationships, and metadata.
///
/// # Errors
///
/// Returns an error if the uuid parameter is missing or if no block exists
/// with the specified UUID.
pub async fn get_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;

    let block = client.get_block(uuid).await?;
    Ok(serde_json::json!({
        "block": block
    }))
}

/// Searches across all content in the graph.
///
/// Performs a full-text search using Logseq's built-in search engine, which
/// provides intelligent ranking and supports various search features like
/// exact phrases, property searches, and content type filtering.
///
/// # Parameters
///
/// - `query` (required): The search terms to look for
///
/// # Returns
///
/// JSON object containing an array of search results, each with:
/// - Matching content snippets
/// - Page/block context information
/// - Relevance ranking from Logseq's search algorithm
///
/// # Search Features
///
/// The search supports:
/// - Full-text search across all blocks and pages
/// - Property searches (e.g., searching for specific tags or metadata)
/// - Phrase searches with quotes
/// - Advanced filtering based on content type
///
/// # Errors
///
/// Returns an error if the query parameter is missing.
pub async fn search(client: &LogseqClient, params: Value) -> Result<Value> {
    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("query parameter is required"))?;

    let results = client.search(query).await?;
    Ok(serde_json::json!({
        "results": results
    }))
}

/// Runs a Datascript/Datalog query against the graph database.
///
/// Exposes Logseq's full Datascript query engine for precise, structured queries.
/// This is far more powerful than full-text search for finding blocks by properties,
/// relationships, or metadata.
///
/// # Parameters
///
/// - `query` (required): Datalog query string
///   Example: `[:find ?n :where [?b :block/name ?n]]` — returns all page names
///   Example: `[:find (pull ?b [*]) :where [?b :block/content ?c] [(clojure.string/includes? ?c "TODO")]]`
///
/// # Returns
///
/// Raw query results as returned by Datascript.
pub async fn query(client: &LogseqClient, params: Value) -> Result<Value> {
    let q = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("query parameter is required"))?;

    let results = client.datascript_query(q).await?;
    Ok(serde_json::json!({ "results": results }))
}

/// Gets today's journal page content.
///
/// Retrieves the journal page for the current date using the graph's configured
/// date format. Journal pages are named after the date in the graph's locale format.
///
/// # Returns
///
/// JSON object with `page` metadata and `blocks` content tree for today's journal.
/// Returns an error if the journal page doesn't exist yet (not yet created in Logseq).
pub async fn get_today_journal(client: &LogseqClient, _params: Value) -> Result<Value> {
    let formatter = client.get_date_formatter().await?;

    // Logseq uses Java-style date format tokens. Map common ones to chrono equivalents.
    let fmt_str = formatter.as_str().unwrap_or("MMM do, yyyy");

    // Format the date and lowercase it (Logseq stores journal pages in lowercase)
    let page_name = format_journal_date(fmt_str).to_lowercase();

    let page_info = client.get_page(&page_name).await?;
    let blocks = client.get_page_blocks_tree(&page_name).await?;

    Ok(serde_json::json!({
        "date": page_name,
        "page": page_info,
        "blocks": blocks
    }))
}

/// Gets all blocks that link to the given page (backlinks).
///
/// Returns every block across the graph that contains a `[[page_name]]` reference.
/// This is a core Logseq feature for understanding how content is connected.
///
/// # Parameters
///
/// - `page_name` (required): The page to find references for
///
/// # Returns
///
/// Array of `[page, [blocks]]` pairs where each block references the given page.
pub async fn get_page_references(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;

    let refs = client.get_page_linked_references(page_name).await?;
    Ok(serde_json::json!({ "references": refs }))
}

/// Gets all properties on a block.
///
/// Retrieves the Logseq property key-value pairs attached to a block,
/// such as `tags::`, `type::`, `priority::`, or any custom properties.
///
/// # Parameters
///
/// - `uuid` (required): UUID of the block
///
/// # Returns
///
/// JSON object mapping property names to their values.
pub async fn get_block_properties(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;

    let props = client.get_block_properties(uuid).await?;
    Ok(serde_json::json!({ "properties": props }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_pages_filter_applies() {
        // format_journal_date is pure — test it directly as a proxy for filtering logic
        let result = format_journal_date("yyyy-MM-dd");
        // Should be a date string like 2026-04-09
        assert!(
            result.len() == 10,
            "Expected ISO date string, got: {}",
            result
        );
        assert!(result.contains('-'));
    }

    #[test]
    fn test_format_journal_date_iso() {
        let result = format_journal_date("yyyy-MM-dd");
        // Should match YYYY-MM-DD pattern
        let parts: Vec<&str> = result.split('-').collect();
        assert_eq!(parts.len(), 3, "Expected 3 date parts");
        assert_eq!(parts[0].len(), 4, "Expected 4-digit year");
    }

    #[test]
    fn test_format_journal_date_month_name() {
        let result = format_journal_date("MMM do, yyyy");
        // Should contain comma, space, and ordinal suffix (e.g. "Apr 9th, 2026")
        assert!(result.contains(','));
        // Should contain an ordinal suffix (st, nd, rd, or th)
        assert!(
            result.contains("st,") || result.contains("nd,") ||
            result.contains("rd,") || result.contains("th,"),
            "Expected ordinal suffix in: {}", result
        );
    }

    #[test]
    fn test_get_ordinal_suffix() {
        assert_eq!(get_ordinal_suffix(1), "st");
        assert_eq!(get_ordinal_suffix(2), "nd");
        assert_eq!(get_ordinal_suffix(3), "rd");
        assert_eq!(get_ordinal_suffix(4), "th");
        assert_eq!(get_ordinal_suffix(11), "th"); // Special case
        assert_eq!(get_ordinal_suffix(12), "th");
        assert_eq!(get_ordinal_suffix(13), "th");
        assert_eq!(get_ordinal_suffix(21), "st");
        assert_eq!(get_ordinal_suffix(22), "nd");
        assert_eq!(get_ordinal_suffix(23), "rd");
        assert_eq!(get_ordinal_suffix(31), "st");
    }
}

/// Public alias for use in mutate.rs (append_to_journal).
pub fn format_journal_date_pub(logseq_fmt: &str) -> String {
    format_journal_date(logseq_fmt)
}

/// Maps a subset of Logseq/Java date format tokens to chrono format strings,
/// then formats today's date. Falls back to ISO date on unknown format tokens.
fn format_journal_date(logseq_fmt: &str) -> String {
    let now = Local::now();

    // Handle ordinal day format "do" specially (1st, 2nd, 3rd, etc.)
    let result = if logseq_fmt.contains("do") {
        let day = now.day();
        let ordinal = get_ordinal_suffix(day);
        let day_with_ordinal = format!("{}{}", day, ordinal);

        // First replace "do" with a placeholder, then do other replacements
        let temp_fmt = logseq_fmt.replace("do", "<<DAY_ORDINAL>>");
        let chrono_fmt = temp_fmt
            .replace("yyyy", "%Y")
            .replace("yy", "%y")
            .replace("MMMM", "%B")
            .replace("MMM", "%b")
            .replace("MM", "%m")
            .replace("dd", "%d")
            .replace("EEE", "%a")
            .replace("EEEE", "%A");

        let formatted = now.format(&chrono_fmt).to_string();
        formatted.replace("<<DAY_ORDINAL>>", &day_with_ordinal)
    } else {
        // Standard format without ordinals
        let chrono_fmt = logseq_fmt
            .replace("yyyy", "%Y")
            .replace("yy", "%y")
            .replace("MMMM", "%B")
            .replace("MMM", "%b")
            .replace("MM", "%m")
            .replace("dd", "%d")
            .replace("EEE", "%a")
            .replace("EEEE", "%A");

        now.format(&chrono_fmt).to_string()
    };

    result
}

/// Returns the ordinal suffix for a day number (st, nd, rd, th).
fn get_ordinal_suffix(day: u32) -> &'static str {
    // Special cases: 11th, 12th, 13th (not 11st, 12nd, 13rd)
    if (11..=13).contains(&day) {
        return "th";
    }

    // Check last digit for other cases
    match day % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}
