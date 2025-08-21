//! Application layer module.
//! 
//! This module contains the application layer of the CLI clean architecture,
//! including dependency injection container and business logic services.

pub mod container;
pub mod services;

pub use container::CliServiceContainer;
pub use services::*;
