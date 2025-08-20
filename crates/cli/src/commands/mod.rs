pub mod search;
pub mod index;
pub mod status;
pub mod server;
pub mod reindex;

use anyhow::Result;
use crate::client::ApiClient;

/// Trait for all CLI commands
pub trait Command {
    async fn execute(&self, client: &ApiClient) -> Result<()>;
}
