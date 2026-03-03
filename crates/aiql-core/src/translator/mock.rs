use crate::{QueryPlan, Schema, Translator, TranslateResult, MigrationPlan, DatabaseDialect, Context, Session};
use async_trait::async_trait;

pub struct MockTranslator;

#[async_trait]
impl Translator for MockTranslator {
    async fn translate(&self, prompt: &str, _schema: &Schema, dialect: DatabaseDialect, _context: &Context, _session: Option<&Session>, _stream: bool) -> anyhow::Result<TranslateResult> {
        Ok(TranslateResult::Plan(QueryPlan {
            dialect,
            raw_query: format!("-- Mock query for: {}\nSELECT * FROM users LIMIT 10;", prompt),
            explanation: format!("This is a mock translation for '{}'.", prompt),
            cost: Some(0.0),
        }))
    }

    async fn translate_migration(&self, prompt: &str, _schema: &Schema, dialect: DatabaseDialect) -> anyhow::Result<MigrationPlan> {
        Ok(MigrationPlan {
            dialect,
            raw_sql: format!("-- Mock migration for: {}\nALTER TABLE users ADD COLUMN mock_field TEXT;", prompt),
            explanation: format!("This is a mock migration for '{}'.", prompt),
        })
    }

    async fn translate_vector(&self, prompt: &str, _schema: &Schema, dialect: DatabaseDialect, _context: &Context, _session: Option<&Session>, _stream: bool) -> anyhow::Result<TranslateResult> {
        Ok(TranslateResult::Plan(QueryPlan {
            dialect,
            raw_query: format!("-- Mock vector query for: {}\nSELECT * FROM products ORDER BY embedding <=> '$VECTOR' LIMIT 5;", prompt),
            explanation: format!("This is a mock vector search for '{}'.", prompt),
            cost: Some(0.0),
        }))
    }
}

#[async_trait]
impl crate::EmbeddingEngine for MockTranslator {
    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0; 1536])
    }
}

#[async_trait]
impl crate::Advisor for MockTranslator {
    async fn advise(&self, _plan: &QueryPlan, _schema: &Schema) -> anyhow::Result<Vec<String>> {
        Ok(vec!["Consider adding an index on frequently filtered columns.".to_string()])
    }
}
