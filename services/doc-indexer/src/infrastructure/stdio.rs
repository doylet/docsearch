/// Stdio transport for JSON-RPC communication
/// 
/// This module provides stdin/stdout JSON-RPC transport, enabling
/// doc-indexer to be used as a subprocess for process-to-process communication.

use std::io::{self, BufRead, BufReader, Write};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::infrastructure::{
    http::handlers::AppState,
    jsonrpc::{handlers::route_method, JsonRpcRequest, JsonRpcResponse, JsonRpcError},
};

/// Stdio JSON-RPC server
pub struct StdioServer {
    app_state: AppState,
}

impl StdioServer {
    /// Create a new stdio server
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Start the stdio JSON-RPC server
    /// Reads JSON-RPC requests from stdin and writes responses to stdout
    pub async fn start(&self) -> io::Result<()> {
        info!("Starting stdio JSON-RPC server");
        
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let response = self.handle_line(&line).await;
                    self.write_response(&response)?;
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
        
        info!("Stdio JSON-RPC server stopped");
        Ok(())
    }

    /// Start the stdio server in async mode using channels
    /// This version allows for non-blocking operation
    pub async fn start_async(&self) -> io::Result<()> {
        info!("Starting async stdio JSON-RPC server");
        
        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        // Spawn stdin reader task
        let stdin_tx = tx.clone();
        tokio::spawn(async move {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if !line.trim().is_empty() && stdin_tx.send(line).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Process requests from channel
        while let Some(line) = rx.recv().await {
            let response = self.handle_line(&line).await;
            if let Err(e) = self.write_response(&response) {
                error!("Error writing response: {}", e);
                break;
            }
        }
        
        info!("Async stdio JSON-RPC server stopped");
        Ok(())
    }

    /// Handle a single line of input (JSON-RPC request)
    async fn handle_line(&self, line: &str) -> JsonRpcResponse {
        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => {
                info!("Processing JSON-RPC request: {}", request.method);
                
                // Route the request through our existing handler
                route_method(
                    &request.method,
                    request.params,
                    request.id,
                    &self.app_state,
                ).await
            }
            Err(e) => {
                warn!("Invalid JSON-RPC request: {}", e);
                JsonRpcResponse::error(None, JsonRpcError::parse_error())
            }
        }
    }

    /// Write a JSON-RPC response to stdout
    fn write_response(&self, response: &JsonRpcResponse) -> io::Result<()> {
        let json = serde_json::to_string(response)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        println!("{}", json);
        io::stdout().flush()?;
        
        Ok(())
    }
}

/// Batch processor for multiple stdin requests
pub struct StdioBatchProcessor {
    app_state: AppState,
}

impl StdioBatchProcessor {
    /// Create a new batch processor
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Process multiple JSON-RPC requests from stdin
    /// Each line should contain a JSON-RPC request
    pub async fn process_batch(&self) -> io::Result<()> {
        info!("Starting stdio batch processing");
        
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut requests = Vec::new();
        
        // Read all requests from stdin
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                break; // Empty line signals end of batch
            }
            
            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => requests.push(request),
                Err(e) => {
                    warn!("Skipping invalid JSON-RPC request: {}", e);
                }
            }
        }
        
        info!("Processing {} requests in batch", requests.len());
        
        // Process all requests and collect responses
        let mut responses = Vec::new();
        for request in requests {
            let response = route_method(
                &request.method,
                request.params,
                request.id,
                &self.app_state,
            ).await;
            responses.push(response);
        }
        
        // Output all responses
        for response in responses {
            let json = serde_json::to_string(&response)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            println!("{}", json);
        }
        
        info!("Batch processing completed");
        Ok(())
    }
}

/// Utility functions for stdio transport
pub mod utils {
    /// Check if we should use stdio mode based on command line arguments
    pub fn should_use_stdio(args: &[String]) -> bool {
        args.iter().any(|arg| arg == "--stdio" || arg == "-s")
    }

    /// Check if we should use batch mode
    pub fn should_use_batch_mode(args: &[String]) -> bool {
        args.iter().any(|arg| arg == "--batch" || arg == "-b")
    }

    /// Print usage information for stdio mode
    pub fn print_stdio_usage() {
        println!("Stdio JSON-RPC Transport Usage:");
        println!("  --stdio, -s     Enable stdio JSON-RPC mode");
        println!("  --batch, -b     Enable batch processing mode");
        println!();
        println!("In stdio mode, send JSON-RPC requests via stdin:");
        println!("  {{\"jsonrpc\": \"2.0\", \"method\": \"service.info\", \"id\": 1}}");
        println!();
        println!("In batch mode, send multiple requests separated by newlines,");
        println!("then send an empty line to signal end of batch.");
    }
}
