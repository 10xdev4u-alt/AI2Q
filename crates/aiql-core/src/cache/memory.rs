use crate::{QueryPlan, SemanticCache};
use async_trait::async_trait;
use std::sync::Mutex;

pub struct InMemorySemanticCache {
    entries: Mutex<Vec<(Vec<f32>, QueryPlan)>>,
    threshold: f32,
}

impl InMemorySemanticCache {
    pub fn new(threshold: f32) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            threshold,
        }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl SemanticCache for InMemorySemanticCache {
    async fn get(&self, embedding: &[f32]) -> anyhow::Result<Option<QueryPlan>> {
        let entries = self.entries.lock().unwrap();
        for (cached_embedding, plan) in entries.iter() {
            if Self::cosine_similarity(embedding, cached_embedding) >= self.threshold {
                return Ok(Some(plan.clone()));
            }
        }
        Ok(None)
    }

    async fn set(&self, embedding: &[f32], plan: QueryPlan) -> anyhow::Result<()> {
        let mut entries = self.entries.lock().unwrap();
        entries.push((embedding.to_vec(), plan));
        Ok(())
    }
}
