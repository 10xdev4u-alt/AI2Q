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
            let collection = self.db.collection::<mongodb::bson::Document>(&coll_name);
            
            // Sample up to 5 documents to infer fields
            let mut cursor = collection.find(None).limit(5).await?;
            let mut inferred_columns = HashMap::new();

            while let Some(doc_res) = cursor.next().await {
                let doc = doc_res?;
                for (key, value) in doc {
                    inferred_columns.insert(key, Column {
                        name: key.clone(),
                        data_type: format!("{:?}", value.element_type()),
                        is_nullable: true,
                        is_primary_key: key == "_id",
                        default_value: None,
                        description: None,
                    });
                }
            }

            tables.insert(
                coll_name.clone(),
                Table {
                    name: coll_name,
                    columns: inferred_columns.into_values().collect(),
                    indexes: Vec::new(),
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
