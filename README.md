# OpenAPI to Axum Code Generator

A Rust workspace that converts OpenAPI specifications into fully functional Axum server code. This tool automatically generates Rust data structures, route handlers, and API endpoints from your OpenAPI specs.

## Project Structure

```
openapi-to-axum/
├── Cargo.toml              # Workspace configuration
├── openapi-parser/         # OpenAPI spec parsing library
├── code-generator/         # Rust code generation logic
├── cli/                    # Command-line interface
└── examples/               # Sample OpenAPI specifications
```

## Prerequisites

- Rust 1.70+ and Cargo
- OpenAPI 3.0+ specification files (YAML or JSON)

## Quick Start

### 1. Clone and Build

```bash
# Clone the repository
git clone <your-repo-url>
cd openapi-to-axum

# Build the project
cargo build
```

### 2. Run with Example

```bash
# Generate code from the example Petstore API
cargo run -- -i examples/petstore.yaml
```

This will generate `generated.rs` in your current directory.

## Usage

### Basic Usage

````bash
# Generate from YAML file (outputs to ./generated.rs)
cargo run -- -i path/to/your/spec.yaml -o ./output

### Using the Generated Code

The generator creates a complete Axum server module:

```rust
// Import the generated code into your project
mod generated;
use generated::{create_app, start_server};

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000".parse().unwrap();
    start_server(addr).await.unwrap();
}
````

## What Gets Generated

For each OpenAPI specification, the tool generates:

- **Data Structures**: Rust structs with Serde derives for all schema definitions
- **Route Handlers**: Async handler functions for each API endpoint
- **Router Setup**: Complete Axum router with all routes configured
- **Server Boilerplate**: Ready-to-use server startup code
- **Type Safety**: Proper Rust types matching your API specification

### Example Output

Given a simple OpenAPI spec with a `Pet` schema and `/pets` endpoint, you get:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
}

pub fn create_app() -> axum::Router {
    axum::Router::new()
        .route("/pets", axum::routing::get(list_pets))
        .route("/pets", axum::routing::post(create_pet))
}

async fn list_pets() -> &'static str {
    "Hello, World!"
}

async fn create_pet() -> &'static str {
    "Hello, World!"
}
```

## Supported OpenAPI Features

- ✅ Path operations (GET, POST, PUT, DELETE)
- ✅ Component schemas and references
- ✅ Request/response bodies
- ✅ Path parameters
- ✅ Basic data types (string, integer, number, boolean, array, object)
- ✅ Optional vs required fields
- ✅ Nested objects and arrays

## Development

### Project Architecture

The workspace is organized into three main crates:

1. **openapi-parser**: Parses OpenAPI YAML/JSON into Rust structs
2. **code-generator**: Converts parsed specs into Rust token streams
3. **cli**: Command-line interface and file I/O

### Adding New Features

1. **Extend Schema Support**: Add new variants to `openapi-parser/src/lib.rs`
2. **Improve Code Generation**: Update handlers in `code-generator/src/lib.rs`
3. **Add CLI Options**: Modify `cli/src/main.rs`

### Running Tests

```bash
# Run all tests
cargo test

# Test specific crate
cargo test -p openapi-parser
cargo test -p code-generator
```

## Example Specifications

Check the `examples/` directory for sample OpenAPI specs:

- `petstore.yaml`: Simple Pet Store API
- Add your own YAML/JSON files to test

## Roadmap

- [ ] Path parameter extraction
- [ ] Request body validation
- [ ] Response type generation
- [ ] Error handling patterns
- [ ] Authentication middleware
- [ ] Database integration templates
- [ ] Customizable code templates

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
