/// Document discovery operations (list, get)
/// 
/// Read-only commands for exploring documents in the search index.
/// To add documents, use 'mdx index <path>' instead.

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use zero_latency_core::Result as ZeroLatencyResult;
use crate::application::CliServiceContainer;

#[derive(Args)]
pub struct DocumentCommand {
    #[command(subcommand)]
    pub action: DocumentAction,
}

#[derive(Subcommand)]
pub enum DocumentAction {
    /// List all documents in the index
    List(ListArgs),
    
    /// Get a specific document by ID
    Get(GetArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Number of documents per page
    #[arg(short, long, default_value = "50")]
    pub limit: u64,
    
    /// Page number (1-based)
    #[arg(short, long, default_value = "1")]
    pub page: u64,
    
    /// Output format (table, json, simple)
    #[arg(short, long, default_value = "table")]
    pub format: String,
}

#[derive(Args)]
pub struct GetArgs {
    /// Document ID to retrieve
    pub id: String,
    
    /// Output format (json, content, metadata)
    #[arg(short, long, default_value = "content")]
    pub format: String,
}

// Response types for HTTP API integration
#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentSummary>,
    pub total_count: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub index_size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub size: u64,
    pub last_modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDocumentResponse {
    pub id: String,
    pub found: bool,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDocumentResponse {
    pub success: bool,
    pub message: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
    pub path: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub success: bool,
    pub message: String,
    pub id: String,
}

impl DocumentCommand {
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        match &self.action {
            DocumentAction::List(args) => self.list_documents(container, args).await,
            DocumentAction::Get(args) => self.get_document(container, args).await,
        }
    }
    
    async fn list_documents(&self, container: &CliServiceContainer, args: &ListArgs) -> ZeroLatencyResult<()> {
        let response = container
            .document_client()
            .list_documents(args.page, args.limit)
            .await?;
        
        container.output_formatter().format_document_list(&response, &args.format).await
    }
    
    async fn get_document(&self, container: &CliServiceContainer, args: &GetArgs) -> ZeroLatencyResult<()> {
        let response = container
            .document_client()
            .get_document(&args.id)
            .await?;
        
        container.output_formatter().format_document_detail(&response, &args.format).await
    }
}
