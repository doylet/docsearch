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
            
            // Truncate snippet for table display (UTF-8 safe)
            let snippet = if result.snippet.len() > 80 {
                let truncated = result.snippet.chars().take(77).collect::<String>();
                format!("{}...", truncated)
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
        println!("📚 Collection: {}", response.collection.name.cyan());
        println!("   📄 Documents: {}", response.collection.documents.to_string().yellow());
        println!("   🔢 Chunks: {}", response.collection.chunks.to_string().yellow());
        println!("   📍 Dimensions: {}", response.collection.vector_dimensions.to_string().yellow());
        if let Some(last_updated) = &response.collection.last_updated {
            println!("   🕒 Last Updated: {}", last_updated.cyan());
        }
        
        // Configuration info
        println!("⚙️ Configuration:");
        println!("   🧠 Model: {}", response.configuration.embedding_model.cyan());
        println!("   🗄️ Database: {}", response.configuration.vector_database.cyan());
        
        // Performance metrics
        println!("📈 Performance:");
        println!("   ⏱️ Uptime: {}s", response.performance.uptime_seconds.to_string().yellow());
        println!("   🔍 Total Searches: {}", response.performance.total_searches.to_string().yellow());
        println!("   ⚡ Avg Search Time: {:.2}ms", response.performance.avg_search_time_ms.to_string().parse::<f64>().unwrap_or(0.0).to_string().yellow());
        
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
