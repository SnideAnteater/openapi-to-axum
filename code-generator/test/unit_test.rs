use code_generator::CodeGenerator;
use openapi_parser::OpenApiSpec;

#[test]
fn test_generate_from_taskmanager_spec() {
    let yaml_content = include_str!("../../taskmanager.yaml");

    let spec = OpenApiSpec::from_yaml(yaml_content).expect("Failed to parse taskmanager.yaml");

    let generated = CodeGenerator::generate_axum_app(&spec);
    let output = generated.to_string();

    // Verify key structures are generated
    assert!(output.contains("struct Task"));
    assert!(output.contains("async fn listTasks"));
    assert!(output.contains("Router::new()"));
}
