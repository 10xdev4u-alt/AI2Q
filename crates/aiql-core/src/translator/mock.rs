use crate::{QueryPlan, Schema, Translator};
use async_trait::async_trait;

pub struct MockTranslator;

#[async_trait]
impl Translator for MockTranslator {
    async fn translate(&self, prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<QueryPlan> {
        Ok(QueryPlan {
            dialect,
            raw_query: format!("-- Mock query for: {}\nSELECT * FROM users LIMIT 10;", prompt),
            explanation: format!("This is a mock translation for '{}'.", prompt),
            cost: Some(0.0),
        })
    }

    async fn translate_migration(&self, prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<MigrationPlan> {
        Ok(MigrationPlan {
            dialect,
            raw_sql: format!("-- Mock migration for: {}\nALTER TABLE users ADD COLUMN mock_field TEXT;", prompt),
            explanation: format!("This is a mock migration for '{}'.", prompt),
        })
    }
}
