//! # Tool Builder
//!
//! Provides a fluent builder API for creating tool definitions,
//! reducing boilerplate and improving readability.

use std::collections::HashMap;
use serde_json::{Value, json};
use super::{Tool, ToolInputSchema};

/// Builder for creating tool definitions with a fluent API
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    properties: HashMap<String, Value>,
    required: Vec<String>,
}

impl ToolBuilder {
    /// Creates a new tool builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    /// Sets the tool description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Adds a string parameter to the tool
    pub fn string_param(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let param_name = name.into();
        self.properties.insert(
            param_name.clone(),
            json!({
                "type": "string",
                "description": description.into()
            }),
        );
        if required {
            self.required.push(param_name);
        }
        self
    }

    /// Adds a boolean parameter to the tool
    pub fn bool_param(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        default: Option<bool>,
        required: bool,
    ) -> Self {
        let param_name = name.into();
        let mut param_def = json!({
            "type": "boolean",
            "description": description.into()
        });
        
        if let Some(default_val) = default {
            param_def["default"] = json!(default_val);
        }
        
        self.properties.insert(param_name.clone(), param_def);
        if required {
            self.required.push(param_name);
        }
        self
    }

    /// Adds an integer parameter to the tool
    #[allow(dead_code)]
    pub fn int_param(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let param_name = name.into();
        self.properties.insert(
            param_name.clone(),
            json!({
                "type": "integer",
                "description": description.into()
            }),
        );
        if required {
            self.required.push(param_name);
        }
        self
    }

    /// Builds the final Tool instance
    pub fn build(self) -> Tool {
        Tool {
            name: self.name,
            description: self.description,
            input_schema: ToolInputSchema {
                r#type: "object".to_string(),
                properties: if self.properties.is_empty() {
                    None
                } else {
                    Some(self.properties)
                },
                required: if self.required.is_empty() {
                    None
                } else {
                    Some(self.required)
                },
            },
        }
    }
}

/// Helper function to create a simple tool without parameters
pub fn simple_tool(name: impl Into<String>, description: impl Into<String>) -> Tool {
    ToolBuilder::new(name)
        .description(description)
        .build()
}

/// Helper function to create a tool with a single required string parameter
pub fn single_string_param_tool(
    name: impl Into<String>,
    description: impl Into<String>,
    param_name: impl Into<String>,
    param_desc: impl Into<String>,
) -> Tool {
    ToolBuilder::new(name)
        .description(description)
        .string_param(param_name, param_desc, true)
        .build()
}