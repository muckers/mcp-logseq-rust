use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogseqApiRequest {
    pub method: String,
    pub args: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogseqApiResponse {
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub name: String,
    pub uuid: String,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub uuid: String,
    pub content: String,
    pub page: Option<String>,
    pub children: Option<Vec<Block>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    pub name: String,
    pub path: String,
}