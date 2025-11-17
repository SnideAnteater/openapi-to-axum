use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenApiError {
    #[error("Failed to parse OpenAPI spec: {0}")]
    ParseError(String),
    #[error("Unsupported OpenAPI version: {0}")]
    UnsupportedVersion(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String,
    pub required: bool,
    pub schema: Option<Schema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestBody {
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Schema {
    Reference {
        #[serde(rename = "$ref")]
        ref_: String,
    },
    Object {
        #[serde(rename = "type")]
        type_: String,
        properties: Option<HashMap<String, Schema>>,
        required: Option<Vec<String>>,
        items: Option<Box<Schema>>,
        format: Option<String>,
    },
    Simple {
        #[serde(rename = "type")]
        type_: String,
        format: Option<String>,
    },
}

// Helper implementation to make schema handling easier
impl Schema {
    pub fn get_type(&self) -> Option<&str> {
        match self {
            Schema::Reference { .. } => Some("reference"),
            Schema::Object { type_, .. } => Some(type_),
            Schema::Simple { type_, .. } => Some(type_),
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Schema::Reference { .. })
    }

    pub fn get_reference(&self) -> Option<&str> {
        match self {
            Schema::Reference { ref_ } => Some(ref_),
            _ => None,
        }
    }
}

impl OpenApiSpec {
    pub fn from_yaml(content: &str) -> Result<Self, OpenApiError> {
        serde_yaml::from_str(content).map_err(|e| OpenApiError::ParseError(e.to_string()))
    }

    pub fn from_json(content: &str) -> Result<Self, OpenApiError> {
        serde_json::from_str(content).map_err(|e| OpenApiError::ParseError(e.to_string()))
    }
}
