use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../api/schemas/zero-latency-api.yaml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let schema_path = Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("api/schemas/zero-latency-api.yaml");

    let generated_dir = Path::new(&out_dir).join("generated");
    fs::create_dir_all(&generated_dir).unwrap();

    // Check if openapi-generator-cli is available
    let generator_available = Command::new("openapi-generator-cli")
        .arg("version")
        .output()
        .is_ok();

    if !generator_available {
        println!("cargo:warning=openapi-generator-cli not found. Install with: npm install -g @openapitools/openapi-generator-cli");
        println!("cargo:warning=Skipping code generation. Using placeholder types.");
        generate_placeholder_types(&generated_dir);
        return;
    }

    // Generate Rust types
    let rust_output = generated_dir.join("rust");
    fs::create_dir_all(&rust_output).unwrap();

    let rust_generation = Command::new("openapi-generator-cli")
        .args([
            "generate",
            "-i", schema_path.to_str().unwrap(),
            "-g", "rust",
            "-o", rust_output.to_str().unwrap(),
            "--additional-properties",
            "packageName=zero_latency_api,supportAsync=true,library=reqwest",
        ])
        .output();

    match rust_generation {
        Ok(output) => {
            if !output.status.success() {
                println!("cargo:warning=Rust code generation failed: {}",
                         String::from_utf8_lossy(&output.stderr));
                generate_placeholder_types(&generated_dir);
            } else {
                println!("cargo:warning=Generated Rust API types successfully");
                copy_generated_rust_files(&rust_output, &generated_dir);
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to run rust code generation: {}", e);
            generate_placeholder_types(&generated_dir);
        }
    }

    // Generate TypeScript client
    let ts_output = generated_dir.join("typescript");
    fs::create_dir_all(&ts_output).unwrap();

    let _ts_generation = Command::new("openapi-generator-cli")
        .args([
            "generate",
            "-i", schema_path.to_str().unwrap(),
            "-g", "typescript-fetch",
            "-o", ts_output.to_str().unwrap(),
            "--additional-properties",
            "npmName=zero-latency-api-client,supportsES6=true",
        ])
        .output();

    // Generate Python client
    let python_output = generated_dir.join("python");
    fs::create_dir_all(&python_output).unwrap();

    let _python_generation = Command::new("openapi-generator-cli")
        .args([
            "generate",
            "-i", schema_path.to_str().unwrap(),
            "-g", "python",
            "-o", python_output.to_str().unwrap(),
            "--additional-properties",
            "packageName=zero_latency_api_client,generateSourceCodeOnly=true",
        ])
        .output();

    // Generate API documentation
    generate_docs(&schema_path, &generated_dir);

    println!("cargo:rustc-env=GENERATED_CODE_DIR={}", generated_dir.display());
}

/// Generate API documentation in multiple formats
fn generate_docs(spec_path: &Path, output_dir: &Path) {
    let docs_dir = output_dir.join("docs");
    let _ = fs::create_dir_all(&docs_dir);

    // Generate Markdown documentation
    let md_output = docs_dir.join("api-reference.md");
    if let Ok(spec_content) = fs::read_to_string(spec_path) {
        if let Ok(spec) = serde_yaml::from_str::<serde_yaml::Value>(&spec_content) {
            if let Ok(markdown_content) = generate_markdown_docs(&spec) {
                let _ = fs::write(md_output, markdown_content);
                println!("cargo:warning=Generated Markdown API documentation");
            }
        }
    }

    // Try to generate HTML documentation with redocly
    let html_output = docs_dir.join("api-reference.html");
    let _html_generation = Command::new("npx")
        .args([
            "@redocly/cli",
            "build-docs",
            spec_path.to_str().unwrap(),
            "--output",
            html_output.to_str().unwrap(),
        ])
        .output();
}

fn generate_placeholder_types(output_dir: &Path) {
    let types_file = output_dir.join("types.rs");
    let placeholder_content = r#"
// Placeholder API types generated when openapi-generator is not available
// Install openapi-generator-cli for full code generation

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub code: String,
    pub trace_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub document_count: Option<i32>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub collection_name: String,
    pub path: Option<String>,
    pub document_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub collection_name: Option<String>,
    pub document_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            collection_name: None,
            document_type: None,
            tags: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filters: Option<Box<SearchFilters>>,
    pub search_type: Option<String>,
    pub include_metadata: Option<bool>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: None,
            offset: None,
            filters: None,
            search_type: None,
            include_metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: i32,
    pub query_time_ms: i32,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f64,
    pub highlights: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRequest {
    pub path: String,
    pub collection_name: Option<String>,
    pub recursive: Option<bool>,
    pub force_reindex: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub success: bool,
    pub processed_count: i32,
    pub failed_count: Option<i32>,
    pub processing_time_ms: Option<i32>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: Option<String>,
    pub uptime_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStatusResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: Option<i32>,
    pub endpoints_count: Option<i32>,
}
"#;

    fs::write(types_file, placeholder_content).unwrap();
}

fn copy_generated_rust_files(rust_output: &Path, target_dir: &Path) {
    // Copy generated Rust files to our target directory
    let src_dir = rust_output.join("src");
    if src_dir.exists() {
        let target_file = target_dir.join("types.rs");

        // Check for models directory (newer openapi-generator versions)
        let models_dir = src_dir.join("models");
        if models_dir.exists() {
            if let Ok(_entries) = fs::read_dir(&models_dir) {
                let mut combined_content = String::from("// Generated API types from OpenAPI specification\n\nuse serde::{Deserialize, Serialize};\nuse uuid::Uuid;\nuse chrono::{DateTime, Utc};\n\n");

                // Priority files to include (search-related types)
                let priority_files = [
                    "search_request.rs",
                    "search_response.rs",
                    "search_result.rs",
                    "search_filters.rs",
                    "api_error.rs",
                    "document.rs",
                    "collection.rs",
                    "index_request.rs",
                    "index_response.rs",
                    "health_check_result.rs",
                    "api_status_response.rs"
                ];

                // First include priority files
                for priority_file in &priority_files {
                    let file_path = models_dir.join(priority_file);
                    if file_path.exists() {
                        if let Ok(content) = fs::read_to_string(&file_path) {
                            let processed_content = process_model_file(&file_path, &content);
                            if !processed_content.trim().is_empty() {
                                combined_content.push_str(&processed_content);
                                combined_content.push_str("\n\n");
                            }
                        }
                    }
                }

                let _ = fs::write(target_file, combined_content);
                return;
            }
        }

        // Fallback: Look for single models.rs file (older openapi-generator versions)
        if let Ok(entries) = fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|n| n.to_str()) == Some("models.rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // Add our custom imports and exports
                        let enhanced_content = format!(
                            "//! Generated API types from OpenAPI specification\n\n{}",
                            content
                        );
                        let _ = fs::write(target_file, enhanced_content);
                    }
                    break;
                }
            }
        }
    }
}

fn process_model_file(file_path: &Path, content: &str) -> String {
    let mut in_comment_block = false;
    let processed = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();

            // Handle multi-line comments
            if trimmed.starts_with("/*") {
                in_comment_block = true;
                return None;
            }
            if in_comment_block {
                if trimmed.ends_with("*/") {
                    in_comment_block = false;
                }
                return None;
            }

            // Filter out unwanted lines but keep struct/enum definitions
            if trimmed.starts_with("use ") ||
               trimmed.starts_with("//") ||
               trimmed.starts_with("*") ||
               (trimmed.is_empty() && !line.starts_with("    ")) {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        // Fix model references - replace crate::models:: and models:: with local references
        .replace("crate::models::", "")
        .replace("models::", "");

    // Rename Status enums to avoid conflicts
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    match file_name {
        "api_status_response.rs" => {
            processed
                .replace("pub enum Status", "pub enum ApiStatus")
                .replace("pub status: Status", "pub status: ApiStatus")
                .replace("status: Status", "status: ApiStatus")
                .replace("-> Status {", "-> ApiStatus {")
                .replace("impl Default for Status", "impl Default for ApiStatus")
                .replace("Self::", "ApiStatus::")
        },
        "collection.rs" => {
            processed
                .replace("pub enum Status", "pub enum CollectionStatus")
                .replace("pub status: Status", "pub status: CollectionStatus")
                .replace("status: Option<Status>", "status: Option<CollectionStatus>")
                .replace("-> Status {", "-> CollectionStatus {")
                .replace("impl Default for Status", "impl Default for CollectionStatus")
                .replace("Self::", "CollectionStatus::")
        },
        "health_check_result.rs" => {
            processed
                .replace("pub enum Status", "pub enum HealthStatus")
                .replace("pub status: Status", "pub status: HealthStatus")
                .replace("status: Status", "status: HealthStatus")
                .replace("-> Status {", "-> HealthStatus {")
                .replace("impl Default for Status", "impl Default for HealthStatus")
                .replace("Self::", "HealthStatus::")
        },
        _ => processed
    }
}

/// Generate basic Markdown documentation from OpenAPI spec
fn generate_markdown_docs(spec: &serde_yaml::Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut content = String::new();

    content.push_str("# Zero-Latency API Documentation\n\n");

    if let Some(info) = spec.get("info") {
        if let Some(title) = info.get("title") {
            content.push_str(&format!("## {}\n\n", title.as_str().unwrap_or("API")));
        }
        if let Some(description) = info.get("description") {
            content.push_str(&format!("{}\n\n", description.as_str().unwrap_or("")));
        }
        if let Some(version) = info.get("version") {
            content.push_str(&format!("**Version:** {}\n\n", version.as_str().unwrap_or("1.0.0")));
        }
    }

    content.push_str("## Endpoints\n\n");

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_mapping()) {
        for (path, path_obj) in paths {
            let path_str = path.as_str().unwrap_or("");
            content.push_str(&format!("### {}\n\n", path_str));

            if let Some(path_mapping) = path_obj.as_mapping() {
                for (method, method_obj) in path_mapping {
                    let method_str = method.as_str().unwrap_or("").to_uppercase();
                    content.push_str(&format!("#### {} {}\n\n", method_str, path_str));

                    if let Some(summary) = method_obj.get("summary") {
                        content.push_str(&format!("{}\n\n", summary.as_str().unwrap_or("")));
                    }

                    if let Some(description) = method_obj.get("description") {
                        content.push_str(&format!("{}\n\n", description.as_str().unwrap_or("")));
                    }

                    // Add request/response schema information
                    if let Some(request_body) = method_obj.get("requestBody") {
                        content.push_str("**Request Body:**\n");
                        if let Some(content_obj) = request_body.get("content").and_then(|c| c.get("application/json")) {
                            if let Some(schema) = content_obj.get("schema") {
                                content.push_str(&format!("```json\n{}\n```\n\n",
                                    serde_yaml::to_string(schema).unwrap_or_default().trim()));
                            }
                        }
                    }
                }
            }
        }
    }

    content.push_str("## Components\n\n");
    if let Some(components) = spec.get("components") {
        if let Some(schemas) = components.get("schemas").and_then(|s| s.as_mapping()) {
            content.push_str("### Schemas\n\n");
            for (schema_name, schema_obj) in schemas {
                let name = schema_name.as_str().unwrap_or("");
                content.push_str(&format!("#### {}\n\n", name));
                if let Some(description) = schema_obj.get("description") {
                    content.push_str(&format!("{}\n\n", description.as_str().unwrap_or("")));
                }
            }
        }
    }

    Ok(content)
}
