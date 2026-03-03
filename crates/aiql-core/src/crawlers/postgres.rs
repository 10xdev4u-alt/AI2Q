use crate::{Column, ForeignKey, Index, Schema, SchemaCrawler, Table};
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::collections::HashMap;

pub struct PostgresSchemaCrawler {
    pool: PgPool,
}

impl PostgresSchemaCrawler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchemaCrawler for PostgresSchemaCrawler {
    async fn crawl(&self) -> anyhow::Result<Schema> {
        let mut tables = HashMap::new();

        // 1. Fetch tables
        let table_rows = sqlx::query!(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_type = 'BASE TABLE'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for row in table_rows {
            let table_name = row.table_name.unwrap();
            
            // 2. Fetch columns for each table
            let column_rows = sqlx::query!(
                r#"
                SELECT column_name, data_type, is_nullable, column_default
                FROM information_schema.columns
                WHERE table_name = $1
                AND table_schema = 'public'
                ORDER BY ordinal_position
                "#,
                table_name
            )
            .fetch_all(&self.pool)
            .await?;

            let mut columns = Vec::new();
            for col_row in column_rows {
                let column_name = col_row.column_name.unwrap();
                
                // PK Detection
                let pk_check = sqlx::query!(
                    r#"
                    SELECT count(*)
                    FROM information_schema.key_column_usage kcu
                    JOIN information_schema.table_constraints tc ON kcu.constraint_name = tc.constraint_name
                    WHERE kcu.table_name = $1 AND kcu.column_name = $2 AND tc.constraint_type = 'PRIMARY KEY'
                    "#,
                    table_name,
                    column_name
                )
                .fetch_one(&self.pool)
                .await?;

                columns.push(Column {
                    name: column_name,
                    data_type: col_row.data_type.unwrap(),
                    is_nullable: col_row.is_nullable.unwrap() == "YES",
                    is_primary_key: pk_check.count.unwrap_or(0) > 0,
                    default_value: col_row.column_default,
                    description: None,
                });
            }

            // FK Detection
            let fk_rows = sqlx::query!(
                r#"
                SELECT
                    tc.constraint_name, 
                    kcu.column_name, 
                    ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name 
                FROM 
                    information_schema.table_constraints AS tc 
                    JOIN information_schema.key_column_usage AS kcu
                      ON tc.constraint_name = kcu.constraint_name
                      AND tc.table_schema = kcu.table_schema
                    JOIN information_schema.constraint_column_usage AS ccu
                      ON ccu.constraint_name = tc.constraint_name
                      AND ccu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_name=$1
                "#,
                table_name
            )
            .fetch_all(&self.pool)
            .await?;

            let mut foreign_keys = Vec::new();
            for fk_row in fk_rows {
                foreign_keys.push(ForeignKey {
                    constraint_name: fk_row.constraint_name.unwrap(),
                    column_name: fk_row.column_name.unwrap(),
                    foreign_table: fk_row.foreign_table_name.unwrap(),
                    foreign_column: fk_row.foreign_column_name.unwrap(),
                });
            }

            // Index Detection
            let index_rows = sqlx::query!(
                r#"
                SELECT
                    indexname,
                    indexdef
                FROM
                    pg_indexes
                WHERE
                    schemaname = 'public'
                    AND tablename = $1
                "#,
                table_name
            )
            .fetch_all(&self.pool)
            .await?;

            let mut indexes = Vec::new();
            for idx_row in index_rows {
                let index_name = idx_row.indexname.unwrap();
                let index_def = idx_row.indexdef.unwrap();
                let is_unique = index_def.contains("UNIQUE INDEX");
                
                // Extract columns from index definition (simplified)
                let re = regex::Regex::new(r"\((.*)\)").unwrap();
                let columns_str = if let Some(caps) = re.captures(&index_def) {
                    caps.get(1).map_or("", |m| m.as_str())
                } else {
                    ""
                };

                let index_columns: Vec<String> = columns_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                indexes.push(Index {
                    name: index_name,
                    columns: index_columns,
                    is_unique,
                });
            }

            tables.insert(
                table_name.clone(),
                Table {
                    name: table_name,
                    columns,
                    indexes,
                    foreign_keys,
                    description: None,
                },
            );
        }

        Ok(Schema { tables })
    }
}
