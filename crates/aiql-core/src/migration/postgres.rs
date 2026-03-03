use crate::{MigrationEngine, MigrationPlan};
use async_trait::async_trait;
use sqlx::postgres::PgPool;

pub struct PostgresMigrationEngine {
    pool: PgPool,
}

impl PostgresMigrationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MigrationEngine for PostgresMigrationEngine {
    async fn migrate(&self, plan: &MigrationPlan) -> anyhow::Result<()> {
        log::info!("AIQL: Executing migration: {}", plan.explanation);
        sqlx::query(&plan.raw_sql).execute(&self.pool).await?;
        Ok(())
    }
}
