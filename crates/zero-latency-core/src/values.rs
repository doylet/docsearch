use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Strongly-typed search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub raw: String,
    pub normalized: String,
    pub enhanced: Option<String>,
}

impl SearchQuery {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let normalized = normalize_query(&raw);
        
        Self {
            raw,
            normalized,
            enhanced: None,
        }
    }

    pub fn with_enhancement(mut self, enhanced: impl Into<String>) -> Self {
        self.enhanced = Some(enhanced.into());
        self
    }

    pub fn effective_query(&self) -> &str {
        self.enhanced.as_ref().unwrap_or(&self.normalized)
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.effective_query())
    }
}

/// Score value with validation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Score(f32);

impl Score {
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err("Score must be between 0.0 and 1.0")
        }
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn one() -> Self {
        Self(1.0)
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn percentage(&self) -> u8 {
        (self.0 * 100.0).round() as u8
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::zero()
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}

/// Response format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    Json,
    JsonCompact,
    Text,
    Csv,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Json
    }
}

/// Service version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVersion {
    pub name: String,
    pub version: String,
    pub build_hash: Option<String>,
    pub build_date: Option<String>,
}

impl ServiceVersion {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            build_hash: None,
            build_date: None,
        }
    }

    pub fn with_build_info(
        mut self, 
        hash: impl Into<String>, 
        date: impl Into<String>
    ) -> Self {
        self.build_hash = Some(hash.into());
        self.build_date = Some(date.into());
        self
    }
}

impl Display for ServiceVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.build_hash, &self.build_date) {
            (Some(hash), Some(date)) => {
                write!(f, "{} v{} ({}@{})", self.name, self.version, &hash[..8], date)
            }
            _ => write!(f, "{} v{}", self.name, self.version),
        }
    }
}

/// Helper function to normalize search queries
fn normalize_query(query: &str) -> String {
    query
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
