use crate::{ExecutionEngine, ExecutionResult};
use async_trait::async_trait;
use mongodb::{Database, bson::doc};
use std::time::Instant;
use futures_util::StreamExt;

pub struct MongoExecutionEngine {
    db: Database,
}

impl MongoExecutionEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExecutionEngine for MongoExecutionEngine {
    async fn execute(&self, query: &str) -> anyhow::Result<ExecutionResult> {
        let start = Instant::now();
        
        // MongoDB queries are often JSON-like. We'll assume the LLM generates a JSON string.
        // For simplicity, we'll try to parse it as an aggregation pipeline or a find query.
        let pipeline: Vec<mongodb::bson::Document> = serde_json::from_str(query)?;

        // We'll need a way to know which collection to target. 
        // For now, we'll assume the query includes the collection name or we'll need to parse it.
        // Simplified: targeting the first collection for prototype.
        let coll_name = "sample_collection"; // TODO: Parse from query
        let collection = self.db.collection::<mongodb::bson::Document>(coll_name);
        
        let mut cursor = collection.aggregate(pipeline).await?;
        let mut results = Vec::new();
        while let Some(doc_res) = cursor.next().await {
            let doc = doc_res?;
            results.push(serde_json::to_value(doc)?);
        }

        Ok(ExecutionResult {
            success: true,
            data: Some(serde_json::Value::Array(results)),
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn dry_run(&self, _query: &str) -> anyhow::Result<bool> {
        // MongoDB doesn't have a direct equivalent to SQL EXPLAIN that returns a bool,
        // but we can run an explain command on the aggregation.
        Ok(true) // Simplified
    }
}
