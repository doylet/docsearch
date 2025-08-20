use anyhow::Result;
use colored::*;
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
use crate::client::{SearchResponse, StatusResponse};

pub struct OutputFormatter {
    format: String,
}

impl OutputFormatter {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
        }
    }
    
    pub fn display_search_results(&self, response: &SearchResponse) -> Result<()> {
        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Score").add_attribute(Attribute::Bold),
                Cell::new("Document").add_attribute(Attribute::Bold),
                Cell::new("Snippet").add_attribute(Attribute::Bold),
                Cell::new("Section").add_attribute(Attribute::Bold),
                Cell::new("Type").add_attribute(Attribute::Bold),
            ]);
        
        for result in &response.results {
            // Color-code based on score
            let score_color = if result.score > 0.8 {
                Color::Green
            } else if result.score > 0.6 {
                Color::Yellow
            } else {
                Color::Red
            };
            
            // Truncate snippet for table display
            let snippet = if result.snippet.len() > 80 {
                format!("{}...", &result.snippet[..77])
            } else {
                result.snippet.clone()
            };
            
            table.add_row(vec![
                Cell::new(format!("{:.3}", result.score)).fg(score_color),
                Cell::new(&result.document_title).add_attribute(Attribute::Bold),
                Cell::new(snippet),
                Cell::new(&result.section),
                Cell::new(&result.doc_type).fg(Color::Cyan),
            ]);
        }
        
        println!("{}", table);
        Ok(())
    }
    
    pub fn display_status(&self, response: &StatusResponse) -> Result<()> {
        println!("{}", "📊 System Status".blue().bold());
        println!();
        
        // Server status
        let status_icon = if response.status == "healthy" { "✅" } else { "⚠️" };
        println!("{} Server: {}", status_icon, response.status.green().bold());
        
        // Collection info
        println!("📚 Collection: {}", response.collection_info.name.cyan());
        println!("   📄 Documents: {}", response.collection_info.indexed_documents.to_string().yellow());
        println!("   🔢 Vectors: {}", response.collection_info.vectors_count.to_string().yellow());
        println!("   📍 Points: {}", response.collection_info.points_count.to_string().yellow());
        
        // Server info
        println!("🚀 Server: v{}", response.server_info.version.green());
        println!("   ⏱️ Uptime: {}s", response.server_info.uptime_seconds.to_string().yellow());
        println!("   🧠 Model: {}", response.server_info.embedding_model.cyan());
        
        Ok(())
    }
    
    pub fn display_success_message(&self, message: &str) -> Result<()> {
        println!("{} {}", "✅".green(), message.green().bold());
        Ok(())
    }
    
    pub fn display_info_message(&self, message: &str) -> Result<()> {
        println!("{} {}", "ℹ️".blue(), message);
        Ok(())
    }
    
    pub fn display_warning_message(&self, message: &str) -> Result<()> {
        println!("{} {}", "⚠️".yellow(), message.yellow());
        Ok(())
    }
    
    pub fn display_error_message(&self, message: &str) -> Result<()> {
        println!("{} {}", "❌".red(), message.red());
        Ok(())
    }
    
    pub fn display_progress_start(&self, message: &str) -> Result<()> {
        println!("{} {}...", "🔄".blue(), message);
        Ok(())
    }
}
