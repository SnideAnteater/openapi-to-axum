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
    // #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    // #[serde(rename = "requestBody")]
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
    AllOf {
        #[serde(rename = "allOf")]
        all_of: Vec<Schema>,
    },
    OneOf {
        #[serde(rename = "oneOf")]
        one_of: Vec<Schema>,
        discriminator: Option<Discriminator>,
    },
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Vec<Schema>,
    },
    Not {
        not: Box<Schema>,
    },
    Object {
        #[serde(rename = "type")]
        type_: Option<String>,
        properties: Option<HashMap<String, Schema>>,
        required: Option<Vec<String>>,
        items: Option<Box<Schema>>,
        format: Option<String>,
        #[serde(rename = "enum")]
        enum_values: Option<Vec<serde_json::Value>>,
    },
    SimpleType {
        #[serde(rename = "type")]
        type_: String,
        format: Option<String>,
        #[serde(rename = "enum")]
        enum_values: Option<Vec<serde_json::Value>>,
    },
    ArrayType {
        #[serde(rename = "type")]
        type_: String,
        items: Box<Schema>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Discriminator {
    #[serde(rename = "propertyName")]
    pub property_name: String,
    pub mapping: Option<HashMap<String, String>>,
}

impl Schema {
    pub fn get_type(&self) -> Option<&str> {
        match self {
            Schema::Reference { .. } => Some("reference"),
            Schema::AllOf { .. } => Some("allOf"),
            Schema::OneOf { .. } => Some("oneOf"),
            Schema::AnyOf { .. } => Some("anyOf"),
            Schema::Not { .. } => Some("not"),
            Schema::Object { type_, .. } => type_.as_deref(),
            Schema::SimpleType { type_, .. } => Some(type_),
            Schema::ArrayType { type_, .. } => Some(type_),
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

    pub fn is_composition(&self) -> bool {
        matches!(
            self,
            Schema::AllOf { .. } | Schema::OneOf { .. } | Schema::AnyOf { .. } | Schema::Not { .. }
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml() {
        let yaml = r#"
        openapi: "3.0.0"
        info:
        title: "Test API"
        version: "1.0.0"
        paths: {}
        components:
        schemas:
            Task:
            type: object
            properties:
                id:
                type: string
        "#;
        let result = OpenApiSpec::from_yaml(yaml);
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert_eq!(spec.info.title, "Test API");
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid_yaml = "{ invalid yaml";
        let result = OpenApiSpec::from_yaml(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
  "openapi": "3.0.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {},
  "components": {
    "schemas": {}
  }
}"#;
        let result = OpenApiSpec::from_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_schema_reference() {
        let schema = Schema::Reference {
            ref_: "#/components/schemas/Task".to_string(),
        };
        assert!(schema.is_reference());
        assert_eq!(schema.get_reference(), Some("#/components/schemas/Task"));
    }

    #[test]
    fn test_schema_composition() {
        let schema = Schema::AllOf { all_of: vec![] };
        assert!(schema.is_composition());
    }
}
