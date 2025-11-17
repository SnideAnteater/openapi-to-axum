use openapi_parser::OpenApiSpec;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn generate_axum_app(spec: &OpenApiSpec) -> TokenStream {
        let structs = Self::generate_data_structures(spec);
        let routes = Self::generate_routes(spec);

        quote! {
            use axum::{
                routing::{get, post},
                Router, Json, extract::Path,
            };
            use serde::{Deserialize, Serialize};

            #structs

            #routes

            pub fn create_app() -> axum::Router {
                axum::Router::new()
                    #routes
            }
        }
    }

    fn generate_data_structures(spec: &OpenApiSpec) -> TokenStream {
        let mut output = TokenStream::new();

        if let Some(components) = &spec.components {
            for (name, schema) in &components.schemas {
                let struct_tokens = Self::schema_to_struct(name, schema);
                output.extend(struct_tokens);
            }
        }

        output
    }

    fn schema_to_struct(name: &str, schema: &openapi_parser::Schema) -> TokenStream {
        let struct_name = format_ident!("{}", Self::sanitize_identifier(name));

        match schema {
            openapi_parser::Schema::Object {
                properties,
                required,
                ..
            } => {
                if let Some(props) = properties {
                    let fields: Vec<TokenStream> = props
                        .iter()
                        .map(|(field_name, field_schema)| {
                            let field_ident =
                                format_ident!("{}", Self::sanitize_identifier(field_name));
                            let field_type = Self::schema_to_type(field_schema);

                            let is_required = required
                                .as_ref()
                                .map(|r| r.contains(field_name))
                                .unwrap_or(false);

                            if is_required {
                                quote! { pub #field_ident: #field_type }
                            } else {
                                quote! { pub #field_ident: Option<#field_type> }
                            }
                        })
                        .collect();

                    quote! {
                        #[derive(Debug, Deserialize, Serialize)]
                        pub struct #struct_name {
                            #(#fields),*
                        }
                    }
                } else {
                    quote! {
                        #[derive(Debug, Deserialize, Serialize)]
                        pub struct #struct_name {
                            // Empty struct for objects without properties
                        }
                    }
                }
            }
            _ => {
                // For non-object schemas, generate a simple struct
                let field_type = Self::schema_to_type(schema);
                quote! {
                    #[derive(Debug, Deserialize, Serialize)]
                    pub struct #struct_name {
                        value: #field_type
                    }
                }
            }
        }
    }

    fn schema_to_type(schema: &openapi_parser::Schema) -> TokenStream {
        match schema {
            openapi_parser::Schema::Reference { ref_ } => {
                let type_name = ref_.split('/').last().unwrap_or("Value");
                let ident = format_ident!("{}", type_name);
                quote! { #ident }
            }
            openapi_parser::Schema::Object { type_, items, .. } => match type_.as_str() {
                "array" => {
                    if let Some(item_schema) = items {
                        let item_type = Self::schema_to_type(item_schema);
                        quote! { Vec<#item_type> }
                    } else {
                        quote! { Vec<serde_json::Value> }
                    }
                }
                "object" => quote! { serde_json::Value },
                _ => quote! { serde_json::Value },
            },
            openapi_parser::Schema::Simple { type_, format } => match type_.as_str() {
                "string" => {
                    if let Some(format) = format {
                        match format.as_str() {
                            "uuid" => quote! { uuid::Uuid },
                            "date" | "date-time" => quote! { String },
                            _ => quote! { String },
                        }
                    } else {
                        quote! { String }
                    }
                }
                "integer" => {
                    if let Some(format) = format {
                        match format.as_str() {
                            "int32" => quote! { i32 },
                            "int64" => quote! { i64 },
                            _ => quote! { i64 },
                        }
                    } else {
                        quote! { i64 }
                    }
                }
                "number" => quote! { f64 },
                "boolean" => quote! { bool },
                _ => quote! { serde_json::Value },
            },
        }
    }

    fn generate_routes(spec: &OpenApiSpec) -> TokenStream {
        let route_defs: Vec<TokenStream> = spec
            .paths
            .iter()
            .map(|(path, path_item)| Self::generate_route_definition(path, path_item))
            .collect();

        quote! {
            #(#route_defs)*
        }
    }

    fn generate_route_definition(path: &str, path_item: &openapi_parser::PathItem) -> TokenStream {
        let mut routes = TokenStream::new();

        if let Some(op) = &path_item.get {
            let handler = Self::generate_handler("get", path, op);
            routes.extend(handler);
        }
        if let Some(op) = &path_item.post {
            let handler = Self::generate_handler("post", path, op);
            routes.extend(handler);
        }

        routes
    }

    fn generate_handler(
        method: &str,
        path: &str,
        operation: &openapi_parser::Operation,
    ) -> TokenStream {
        let handler_name = if let Some(op_id) = &operation.operation_id {
            format_ident!("{}", Self::sanitize_identifier(op_id))
        } else {
            format_ident!("handle_{}_{}", method, Self::sanitize_path(path))
        };

        quote! {
            .route(#path, axum::routing::#method(#handler_name))

            async fn #handler_name() -> &'static str {
                "Hello, World!"
            }
        }
    }

    fn sanitize_identifier(ident: &str) -> String {
        ident.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    fn sanitize_path(path: &str) -> String {
        path.replace('/', "_")
            .replace(|c: char| !c.is_alphanumeric(), "_")
    }
}
