use crate::{ForeignKey, RelationshipDiscoverer, Schema};
use async_trait::async_trait;

pub struct NameBasedRelationshipDiscoverer;

#[async_trait]
impl RelationshipDiscoverer for NameBasedRelationshipDiscoverer {
    async fn discover(&self, schema: &mut Schema) -> anyhow::Result<()> {
        let table_names: Vec<String> = schema.tables.keys().cloned().collect();
        
        for table_name in &table_names {
            let mut new_fks = Vec::new();
            
            // Look at each table's columns
            if let Some(table) = schema.tables.get(table_name) {
                for col in &table.columns {
                    // Check for patterns like "other_table_id" or "other_table_uuid"
                    for other_table_name in &table_names {
                        if other_table_name == table_name {
                            continue;
                        }

                        let patterns = vec![
                            format!("{}_id", other_table_name),
                            format!("{}id", other_table_name),
                            format!("{}_uuid", other_table_name),
                        ];

                        if patterns.contains(&col.name.to_lowercase()) {
                            // Check if other table has an "id" or "uuid" column
                            if let Some(other_table) = schema.tables.get(other_table_name) {
                                if let Some(target_col) = other_table.columns.iter().find(|c| c.is_primary_key || c.name.to_lowercase() == "id") {
                                    new_fks.push(ForeignKey {
                                        constraint_name: format!("discovered_fk_{}_{}_{}", table_name, col.name, other_table_name),
                                        column_name: col.name.clone(),
                                        foreign_table: other_table_name.clone(),
                                        foreign_column: target_col.name.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if !new_fks.is_empty() {
                if let Some(table) = schema.tables.get_mut(table_name) {
                    // Only add if not already present
                    for new_fk in new_fks {
                        if !table.foreign_keys.iter().any(|fk| fk.column_name == new_fk.column_name && fk.foreign_table == new_fk.foreign_table) {
                            table.foreign_keys.push(new_fk);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
