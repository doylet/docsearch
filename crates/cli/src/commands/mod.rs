pub mod search;
pub mod index;
pub mod status;
pub mod server;
pub mod reindex;

use zero_latency_core::Result as ZeroLatencyResult;
use crate::application::CliServiceContainer;

/// Trait for all CLI commands
pub trait Command {
    async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()>;
}
