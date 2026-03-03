pub mod crawlers;
pub mod translator;
pub mod execution;
pub mod healer;
pub mod client;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub raw_query: String,
    pub explanation: String,
    pub cost: Option<f64>,
}

#[async_trait::async_trait]
pub trait Translator {
    async fn translate(&self, prompt: &str, schema: &Schema) -> anyhow::Result<QueryPlan>;
}

#[async_trait::async_trait]
pub trait QueryHealer {
    async fn heal(&self, query: &str, error: &str, schema: &Schema) -> anyhow::Result<QueryPlan>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[async_trait::async_trait]
pub trait ExecutionEngine {
    async fn execute(&self, query: &str) -> anyhow::Result<ExecutionResult>;
    async fn dry_run(&self, query: &str) -> anyhow::Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub tables: HashMap<String, Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub constraint_name: String,
    pub column_name: String,
    pub foreign_table: String,
    pub foreign_column: String,
}

#[async_trait::async_trait]
pub trait SchemaCrawler {
    async fn crawl(&self) -> anyhow::Result<Schema>;
}
