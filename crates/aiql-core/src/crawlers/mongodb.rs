use crate::{Column, Schema, SchemaCrawler, Table};
use async_trait::async_trait;
use mongodb::{Client, Database};
use std::collections::HashMap;
use futures_util::StreamExt;

pub struct MongoSchemaCrawler {
    db: Database,
}

impl MongoSchemaCrawler {
    pub fn new(client: Client, db_name: &str) -> Self {
        let db = client.database(db_name);
        Self { db }
    }
}

#[async_trait]
impl SchemaCrawler for MongoSchemaCrawler {
    async fn crawl(&self) -> anyhow::Result<Schema> {
        let mut tables = HashMap::new();
        let collection_names = self.db.list_collection_names().await?;

        for coll_name in collection_names {
            // MongoDB is schemaless, but we can infer schema by sampling documents
            // For now, let's just list collections as tables
            tables.insert(
                coll_name.clone(),
                Table {
                    name: coll_name,
                    columns: Vec::new(), // TODO: Sampling
                    indexes: Vec::new(), // TODO: Index discovery
                    foreign_keys: Vec::new(),
                    description: None,
                },
            );
        }

        Ok(Schema {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            tables,
        })
    }
}
