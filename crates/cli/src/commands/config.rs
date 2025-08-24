use clap::{Args, Subcommand};
use colored::*;
use zero_latency_core::Result as ZeroLatencyResult;
use crate::application::CliServiceContainer;
use crate::config::CliConfig;

#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration from file
    Set(SetConfig),
    /// Get current configuration and save to file
    Export(ExportConfig),
    /// Reset configuration to defaults
    Reset,
}

#[derive(Args)]
struct SetConfig {
    /// Path to configuration file to load
    path: std::path::PathBuf,
}

#[derive(Args)]
struct ExportConfig {
    /// Path where to save current configuration
    path: std::path::PathBuf,
}

impl ConfigCommand {
    /// Execute the config command with proper dependency injection
    pub async fn execute(&self, container: &CliServiceContainer) -> ZeroLatencyResult<()> {
        match &self.action {
            ConfigAction::Show => self.show_config(Some(container)).await,
            ConfigAction::Set(args) => self.set_config(&args.path).await,
            ConfigAction::Export(args) => self.export_config(&args.path).await,
            ConfigAction::Reset => self.reset_config().await,
        }
    }

    async fn show_config(&self, container: Option<&CliServiceContainer>) -> ZeroLatencyResult<()> {
        println!("{}", "Current Configuration".blue().bold());
        println!();

        let config = if let Some(container) = container {
            // Use config from container (respects --config file loading)
            (*container.config()).clone()
        } else {
            // Fallback to loading from default location
            CliConfig::load().map_err(|e| {
                zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to load config: {}", e))
            })?
        };

        // Display config in a nice table format
        println!("┌─────────────────────┬─────────────────────────────────────────┐");
        println!("│ {}              │ {}                                │", "Setting".cyan().bold(), "Value".cyan().bold());
        println!("├─────────────────────┼─────────────────────────────────────────┤");
        println!("│ Server URL          │ {:39} │", config.server_url);
        println!("│ Collection Name     │ {:39} │", config.collection_name);
        println!("│ Default Limit       │ {:39} │", config.default_limit);
        println!("│ Output Format       │ {:39} │", config.output_format);
        println!("│ Verbose             │ {:39} │", config.verbose);
        println!("└─────────────────────┴─────────────────────────────────────────┘");
        println!();

        let config_file = CliConfig::config_file().map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to get config file path: {}", e))
        })?;
        
        println!("Config file: {}", config_file.display().to_string().dimmed());
        
        Ok(())
    }

    async fn set_config(&self, path: &std::path::Path) -> ZeroLatencyResult<()> {
        println!("{} {}", "Loading configuration from".blue().bold(), path.display().to_string().cyan());

        // Check if file exists
        if !path.exists() {
            return Err(zero_latency_core::ZeroLatencyError::not_found(&format!("Config file not found: {}", path.display())));
        }

        // Load config from file
        let content = std::fs::read_to_string(path).map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to read config file: {}", e))
        })?;

        let new_config: CliConfig = toml::from_str(&content).map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Invalid config format: {}", e))
        })?;

        // Save the new config
        new_config.save().map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to save config: {}", e))
        })?;

        println!("Configuration applied successfully!");
        println!();

        // Show the new config
        self.show_config(None).await?;

        Ok(())
    }

    async fn export_config(&self, path: &std::path::Path) -> ZeroLatencyResult<()> {
        println!("{} {}", "Exporting configuration to".blue().bold(), path.display().to_string().cyan());

        let config = CliConfig::load().map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to load config: {}", e))
        })?;

        let content = toml::to_string_pretty(&config).map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to write config file: {}", e))
        })?;

        println!("Configuration exported successfully!");
        println!("Saved to: {}", path.display().to_string().cyan());

        Ok(())
    }

    async fn reset_config(&self) -> ZeroLatencyResult<()> {
        println!("{}", "Resetting configuration to defaults".yellow().bold());

        let default_config = CliConfig::default();
        default_config.save().map_err(|e| {
            zero_latency_core::ZeroLatencyError::configuration(&format!("Failed to save config: {}", e))
        })?;

        println!("Configuration reset to defaults!");
        println!();

        // Show the reset config
        self.show_config(None).await?;

        Ok(())
    }
}
