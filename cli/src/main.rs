use anyhow::Result;
use clap::Parser;
use code_generator::CodeGenerator;
use openapi_parser::OpenApiSpec;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "openapi-to-axum")]
#[command(about = "Generate Rust Axum code from OpenAPI specifications")]
struct Cli {
    /// Input OpenAPI file (JSON or YAML)
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated code
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Generate example server code
    #[arg(short, long)]
    example: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read input file
    let content = std::fs::read_to_string(&cli.input)?;

    // Parse OpenAPI spec
    let spec = if cli.input.extension().map(|e| e == "json").unwrap_or(false) {
        OpenApiSpec::from_json(&content)?
    } else {
        OpenApiSpec::from_yaml(&content)?
    };

    // Generate code
    let generated_tokens = CodeGenerator::generate_axum_app(&spec);

    // Format the generated code properly
    let syntax_tree = syn::parse2(generated_tokens)?;
    let formatted_code = prettyplease::unparse(&syntax_tree);

    // Output the code
    if let Some(output_dir) = cli.output {
        std::fs::create_dir_all(&output_dir)?;
        let output_file = output_dir.join("generated.rs");
        std::fs::write(output_file, formatted_code)?;
        println!("Generated code written to: {}", output_dir.display());
    } else {
        println!("{}", formatted_code);
    }

    Ok(())
}
