use crate::{QueryPlan, Schema, Translator};
use async_trait::async_trait;

pub struct MockTranslator;

#[async_trait]
impl Translator for MockTranslator {
    async fn translate(&self, prompt: &str, _schema: &Schema) -> anyhow::Result<QueryPlan> {
        Ok(QueryPlan {
            raw_query: format!("-- Mock query for: {}\nSELECT * FROM users LIMIT 10;", prompt),
            explanation: format!("This is a mock translation for '{}'.", prompt),
            cost: Some(0.0),
        })
    }
}
