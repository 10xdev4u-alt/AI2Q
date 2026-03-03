pub mod crawlers;
pub mod translator;
pub mod execution;
pub mod healer;
pub mod client;
pub mod utils;
pub mod migration;
pub mod vector;
pub mod cache;
pub mod privacy;
pub mod discovery;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait PrivacyGuard {
    /// Checks a prompt for PII and returns a masked version or an error if unsafe.
    async fn scrub_prompt(&self, prompt: &str) -> anyhow::Result<String>;
    /// Masks PII in result data.
    async fn mask_results(&self, data: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

#[async_trait::async_trait]
pub trait SemanticCache {
    /// Tries to find a cached query plan based on prompt embedding.
    async fn get(&self, embedding: &[f32]) -> anyhow::Result<Option<QueryPlan>>;
    /// Stores a query plan in the cache with its associated embedding.
    async fn set(&self, embedding: &[f32], plan: QueryPlan) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait EmbeddingEngine {
    /// Generates a vector embedding for the given text.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseDialect {
    Postgres,
    MySQL,
    SQLite,
    MongoDB,
    Supabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub dialect: DatabaseDialect,
    pub raw_query: String,
    pub explanation: String,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub dialect: DatabaseDialect,
    pub raw_sql: String,
    pub explanation: String,
}

#[async_trait::async_trait]
pub trait MigrationEngine {
    /// Executes a migration plan against the database.
    async fn migrate(&self, plan: &MigrationPlan) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub id: String,
    pub history: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AskResult {
    Success(ExecutionResult),
    ClarificationNeeded {
        reason: String,
        suggestions: Vec<String>,
    },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslateResult {
    Plan(QueryPlan),
    ClarificationNeeded {
        reason: String,
        suggestions: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub now: chrono::DateTime<chrono::Utc>,
}

/// Translator is responsible for converting natural language prompts into executable database queries.
#[async_trait::async_trait]
pub trait Translator {
    /// Translates a prompt into a query plan or clarification request given the database schema, dialect, context, and session history.
    async fn translate(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect, context: &Context, session: Option<&Session>) -> anyhow::Result<TranslateResult>;

    /// Translates a natural language migration prompt into a migration plan.
    async fn translate_migration(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect) -> anyhow::Result<MigrationPlan>;

    /// Translates a natural language prompt into a query plan that includes vector search placeholders.
    async fn translate_vector(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect, context: &Context, session: Option<&Session>) -> anyhow::Result<TranslateResult>;
}

/// QueryHealer is responsible for fixing broken or inefficient queries.
#[async_trait::async_trait]
pub trait QueryHealer {
    /// Heals a broken query by analyzing the error message and schema context.
    async fn heal(&self, query: &str, error: &str, schema: &Schema) -> anyhow::Result<QueryPlan>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Budget {
    pub max_execution_time_ms: Option<u64>,
    pub max_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// ExecutionEngine runs queries against a database and returns structured results.
#[async_trait::async_trait]
pub trait ExecutionEngine {
    /// Executes a query and returns result data or an error message.
    async fn execute(&self, query: &str) -> anyhow::Result<ExecutionResult>;
    /// Runs an EXPLAIN plan to verify query safety and correctness.
    async fn dry_run(&self, query: &str) -> anyhow::Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

/// SchemaCrawler is responsible for auto-ingesting database structure.
#[async_trait::async_trait]
pub trait SchemaCrawler {
    /// Crawls the database to extract tables, columns, indexes, and relationships.
    async fn crawl(&self) -> anyhow::Result<Schema>;
}

/// RelationshipDiscoverer finds hidden relationships between tables.
#[async_trait::async_trait]
pub trait RelationshipDiscoverer {
    /// Discovers relationships in the schema that are not explicitly defined as foreign keys.
    async fn discover(&self, schema: &mut Schema) -> anyhow::Result<()>;
}

/// MockDataGenerator generates synthetic data for testing.
#[async_trait::async_trait]
pub trait MockDataGenerator {
    /// Generates mock data queries based on a prompt and schema.
    async fn generate_mock_data(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect) -> anyhow::Result<Vec<String>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_serialization() {
        let mut tables = HashMap::new();
        tables.insert(
            "users".to_string(),
            Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: false,
                    is_primary_key: true,
                    default_value: None,
                    description: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
                description: Some("Test table".to_string()),
            },
        );

        let schema = Schema { 
            version: "1.0".into(), 
            created_at: chrono::Utc::now(), 
            tables 
        };
        let json = serde_json::to_string(&schema).unwrap();
        let deserialized: Schema = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tables.len(), 1);
        assert_eq!(deserialized.tables["users"].name, "users");
    }
}
