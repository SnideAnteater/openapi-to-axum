use openapi_parser::OpenApiSpec;

#[test]
fn test_full_openapi_spec_parsing() {
    let yaml_content = r#"
openapi: "3.0.0"
info:
  title: "Task API"
  version: "1.0.0"
paths:
  /tasks:
    get:
      operationId: "listTasks"
      responses:
        "200":
          description: "Success"
components:
  schemas:
    Task:
      type: object
      required:
        - id
        - title
      properties:
        id:
          type: string
        title:
          type: string
"#;

    let spec = OpenApiSpec::from_yaml(yaml_content).expect("Failed to parse");
    assert_eq!(spec.info.title, "Task API");
    assert!(spec.paths.contains_key("/tasks"));
}
