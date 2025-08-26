pub mod cli_service;

pub use cli_service::{
    CliServiceImpl, IndexCommand, ReindexCommand, SearchCommand, ServerCommand, StatusCommand,
};
