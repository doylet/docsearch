/// Embedded vector store adapter using SQLite
/// 
/// This adapter provides a self-contained, persistent vector storage solution
/// that doesn't require external databases. It uses SQLite with binary blob
/// storage for vectors and provides efficient similarity search.

use std::path::{PathBuf};
use std::sync::Arc;
use rusqlite::{Connection, params, OpenFlags};
use async_trait::async_trait;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use zero_latency_core::{Result, ZeroLatencyError, models::HealthStatus, values::Score, Uuid};
use zero_latency_vector::{VectorRepository, VectorDocument, SimilarityResult, SimilarityCalculator, SimilarityMetric, VectorMetadata};
use lru::LruCache;

/// Configuration for embedded vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    /// Database file path
    pub db_path: PathBuf,
    /// Vector dimension
    pub dimension: usize,
    /// Maximum documents to cache in memory
    pub cache_size: usize,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            db_path: home_dir.join(".zero-latency").join("vectors.db"),
            dimension: 384, // gte-small default
            cache_size: 10000,
        }
    }
}

/// Embedded vector store using SQLite
pub struct EmbeddedVectorStore {
    db_path: PathBuf,
    connection: Arc<Mutex<Connection>>,
    config: EmbeddedConfig,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
}impl EmbeddedVectorStore {
    /// Create a new embedded vector store
    pub async fn new(config: EmbeddedConfig) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ZeroLatencyError::database(format!("Failed to create database directory: {}", e)))?;
        }

        // Open SQLite connection
        let connection = Connection::open_with_flags(
            &config.db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        ).map_err(|e| ZeroLatencyError::database(format!("Failed to open database: {}", e)))?;

        let store = Self {
            db_path: config.db_path.clone(),
            connection: Arc::new(Mutex::new(connection)),
            config: config.clone(),
            cache: Arc::new(Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(config.cache_size).unwrap()
            ))),
        };

        // Initialize database schema
        store.initialize_schema().await?;

        Ok(store)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS vectors (
                id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| ZeroLatencyError::database(format!("Failed to create vectors table: {}", e)))?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vectors_created_at ON vectors(created_at)",
            [],
        ).map_err(|e| ZeroLatencyError::database(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Serialize vector to binary format
    fn serialize_vector(&self, vector: &[f32]) -> Result<Vec<u8>> {
        bincode::serialize(vector)
            .map_err(|e| ZeroLatencyError::database(format!("Failed to serialize vector: {}", e)))
    }

    /// Deserialize vector from binary format
    fn deserialize_vector(&self, data: &[u8]) -> Result<Vec<f32>> {
        bincode::deserialize(data)
            .map_err(|e| ZeroLatencyError::database(format!("Failed to deserialize vector: {}", e)))
    }

    /// Get vector from cache or database
    async fn get_vector(&self, document_id: &str) -> Result<Option<Vec<f32>>> {
        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(vector) = cache.get(document_id) {
                return Ok(Some(vector.clone()));
            }
        }

        // Load from database
        let conn = self.connection.lock().await;
        let mut stmt = conn.prepare("SELECT embedding FROM vectors WHERE id = ?")
            .map_err(|e| ZeroLatencyError::database(format!("Failed to prepare query: {}", e)))?;

        let result: std::result::Result<Vec<u8>, _> = stmt.query_row(params![document_id], |row| {
            row.get(0)
        });

        match result {
            Ok(blob) => {
                let vector = self.deserialize_vector(&blob)?;
                
                // Cache the vector
                {
                    let mut cache = self.cache.lock().await;
                    cache.put(document_id.to_string(), vector.clone());
                }
                
                Ok(Some(vector))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ZeroLatencyError::database(format!("Database query failed: {}", e))),
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<EmbeddedStats> {
        let conn = self.connection.lock().await;
        
        let document_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vectors",
            [],
            |row| row.get(0)
        ).map_err(|e| ZeroLatencyError::database(format!("Failed to count documents: {}", e)))?;

        let db_size = std::fs::metadata(&self.config.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(EmbeddedStats {
            document_count: document_count as usize,
            db_size_bytes: db_size,
            cache_size: self.cache.lock().await.len(),
            db_path: self.config.db_path.clone(),
        })
    }

    /// Compact database (VACUUM)
    pub async fn compact(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.execute("VACUUM", [])
            .map_err(|e| ZeroLatencyError::database(format!("Failed to compact database: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl VectorRepository for EmbeddedVectorStore {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()> {
        for document in vectors {
            let embedding_blob = self.serialize_vector(&document.embedding)?;
            let metadata_json = serde_json::to_string(&document.metadata)
                .map_err(|e| ZeroLatencyError::database(format!("Failed to serialize metadata: {}", e)))?;

            {
                let conn = self.connection.lock().await;
                let mut stmt = conn.prepare(
                    "INSERT OR REPLACE INTO vectors (id, embedding, metadata) VALUES (?, ?, ?)"
                ).map_err(|e| ZeroLatencyError::database(format!("Failed to prepare insert: {}", e)))?;

                stmt.execute(params![
                    document.id.to_string(),
                    embedding_blob,
                    metadata_json
                ]).map_err(|e| ZeroLatencyError::database(format!("Failed to insert document: {}", e)))?;
            }

            // Update cache
            {
                let mut cache = self.cache.lock().await;
                cache.put(document.id.to_string(), document.embedding);
            }
        }

        Ok(())
    }

    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>> {
        let conn = self.connection.lock().await;
        let mut stmt = conn.prepare("SELECT id, embedding, metadata FROM vectors")
            .map_err(|e| ZeroLatencyError::database(format!("Failed to prepare search: {}", e)))?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let embedding_blob: Vec<u8> = row.get(1)?;
            let metadata_json: String = row.get(2)?;
            Ok((id, embedding_blob, metadata_json))
        }).map_err(|e| ZeroLatencyError::database(format!("Failed to execute search: {}", e)))?;

        let mut results = Vec::new();

        for row_result in rows {
            let (id, embedding_blob, metadata_json) = row_result
                .map_err(|e| ZeroLatencyError::database(format!("Failed to read row: {}", e)))?;

            let embedding = self.deserialize_vector(&embedding_blob)?;
            let metadata: VectorMetadata = serde_json::from_str(&metadata_json)
                .map_err(|e| ZeroLatencyError::database(format!("Failed to parse metadata: {}", e)))?;

            let calculator = CosineCalculator;
            let similarity = calculator.calculate_similarity(&query_vector, &embedding);

            let document_id = Uuid::parse_str(&id)
                .map_err(|e| ZeroLatencyError::database(format!("Invalid UUID: {}", e)))?;

            results.push(SimilarityResult {
                document_id,
                similarity: Score::new(similarity).unwrap_or_else(|_| {
                    Score::new(0.0).unwrap()
                }),
                metadata,
            });
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| {
            b.similarity.value().partial_cmp(&a.similarity.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(k);

        Ok(results)
    }

    async fn delete(&self, document_id: &str) -> Result<bool> {
        let conn = self.connection.lock().await;
        let changes = conn.execute("DELETE FROM vectors WHERE id = ?", params![document_id])
            .map_err(|e| ZeroLatencyError::database(format!("Failed to delete document: {}", e)))?;

        // Remove from cache
        {
            let mut cache = self.cache.lock().await;
            cache.pop(document_id);
        }

        Ok(changes > 0)
    }

    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool> {
        let embedding_blob = self.serialize_vector(&vector)?;
        
        let conn = self.connection.lock().await;
        let changes = conn.execute(
            "UPDATE vectors SET embedding = ? WHERE id = ?",
            params![embedding_blob, document_id]
        ).map_err(|e| ZeroLatencyError::database(format!("Failed to update document: {}", e)))?;

        if changes > 0 {
            // Update cache
            let mut cache = self.cache.lock().await;
            cache.put(document_id.to_string(), vector);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn count(&self) -> Result<usize> {
        let conn = self.connection.lock().await;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |row| row.get(0))
            .map_err(|e| ZeroLatencyError::database(format!("Failed to count documents: {}", e)))?;
        Ok(count as usize)
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // Test database connectivity
        let conn = self.connection.lock().await;
        conn.query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| ZeroLatencyError::database(format!("Database health check failed: {}", e)))?;

        Ok(HealthStatus::Healthy)
    }
}

/// Statistics for embedded vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedStats {
    pub document_count: usize,
    pub db_size_bytes: u64,
    pub cache_size: usize,
    pub db_path: PathBuf,
}

/// Cosine similarity calculator
struct CosineCalculator;

impl SimilarityCalculator for CosineCalculator {
    fn calculate_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    fn batch_similarities(&self, query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32> {
        candidates.iter()
            .map(|candidate| self.calculate_similarity(query, candidate))
            .collect()
    }

    fn metric(&self) -> SimilarityMetric {
        SimilarityMetric::Cosine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_embedded_store_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let config = EmbeddedConfig {
            db_path: temp_dir.path().join("test_vectors.db"),
            dimension: 3,
            cache_size: 100,
        };

        let store = EmbeddedVectorStore::new(config).await.unwrap();

        // Test insert
        let doc = VectorDocument {
            id: Uuid::new_v4(),
            embedding: vec![1.0, 0.0, 0.0],
            metadata: VectorMetadata {
                document_id: Uuid::new_v4(),
                chunk_index: 0,
                content: "test content".to_string(),
                title: "test1".to_string(),
                heading_path: vec![],
                url: None,
                custom: std::collections::HashMap::new(),
            },
        };

        store.insert(vec![doc]).await.unwrap();

        // Test count
        assert_eq!(store.count().await.unwrap(), 1);

        // Test search
        let results = store.search(vec![1.0, 0.0, 0.0], 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.title, "test1");

        // Test delete
        assert!(store.delete("test1").await.unwrap());
        assert_eq!(store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_embedded_store_persistence() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("persist_test.db");
        
        let config = EmbeddedConfig {
            db_path: db_path.clone(),
            dimension: 3,
            cache_size: 100,
        };

        // Create store and insert data
        {
            let store = EmbeddedVectorStore::new(config.clone()).await.unwrap();
            let doc = VectorDocument {
                id: Uuid::new_v4(),
                embedding: vec![0.5, 0.5, 0.0],
                metadata: VectorMetadata {
                    document_id: Uuid::new_v4(),
                    chunk_index: 0,
                    content: "persist content".to_string(),
                    title: "persist1".to_string(),
                    heading_path: vec![],
                    url: None,
                    custom: std::collections::HashMap::new(),
                },
            };
            store.insert(vec![doc]).await.unwrap();
        }

        // Create new store instance and verify data persists
        {
            let store = EmbeddedVectorStore::new(config).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);
            
            let results = store.search(vec![0.5, 0.5, 0.0], 10).await.unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].metadata.title, "persist1");
        }
    }
}
