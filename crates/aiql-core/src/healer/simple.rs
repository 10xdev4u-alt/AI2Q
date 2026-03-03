use crate::{QueryHealer, QueryPlan, Schema};
use async_trait::async_trait;

pub struct SimpleHealer;

#[async_trait]
impl QueryHealer for SimpleHealer {
    async fn heal(&self, query: &str, error: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _context: &crate::Context) -> anyhow::Result<QueryPlan> {
        Ok(QueryPlan {
            dialect,
            raw_query: format!("-- Healed query for: {}\n-- Error was: {}\nSELECT * FROM users LIMIT 5;", query, error),
            explanation: format!("Healed query after error: {}", error),
            cost: Some(0.0),
        })
    }
}
