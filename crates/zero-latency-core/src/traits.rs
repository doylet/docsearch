use async_trait::async_trait;
use crate::{Result, models::*};
use std::time::Duration;

/// Health checking capability
#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> Result<ComponentHealth>;
}

/// Service lifecycle management
#[async_trait]
pub trait ServiceLifecycle: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn restart(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

/// Configuration provider
pub trait ConfigProvider: Send + Sync {
    type Config;
    
    fn get_config(&self) -> &Self::Config;
    fn reload_config(&mut self) -> Result<()>;
}

/// Event publishing capability
#[async_trait]
pub trait EventPublisher: Send + Sync {
    type Event: Send + Sync;
    
    async fn publish(&self, event: Self::Event) -> Result<()>;
    async fn publish_batch(&self, events: Vec<Self::Event>) -> Result<()>;
}

/// Metrics collection capability
pub trait MetricsCollector: Send + Sync {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn record_duration(&self, name: &str, duration: Duration, labels: &[(&str, &str)]);
}

/// Repository pattern for data access
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn save(&self, entity: T) -> Result<T>;
    async fn delete(&self, id: &ID) -> Result<bool>;
    async fn count(&self) -> Result<usize>;
}

/// Query pattern for read operations
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<R>;
}

/// Command pattern for write operations
#[async_trait]
pub trait CommandHandler<C, R>: Send + Sync {
    async fn handle(&self, command: C) -> Result<R>;
}

/// Caching capability
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Result<Option<V>>;
    async fn set(&self, key: K, value: V, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &K) -> Result<bool>;
    async fn clear(&self) -> Result<()>;
}

/// Serialization capability
pub trait Serializer: Send + Sync {
    type Output;
    type Error;
    
    fn serialize<T: serde::Serialize>(&self, value: &T) -> std::result::Result<Self::Output, Self::Error>;
    fn deserialize<T: serde::de::DeserializeOwned>(&self, data: &Self::Output) -> std::result::Result<T, Self::Error>;
}
