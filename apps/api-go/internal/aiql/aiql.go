package aiql

import (
	"encoding/json"
	"fmt"
)

// Schema represents the database schema returned by the Rust core.
type Schema struct {
	Tables map[string]Table `json:"tables"`
}

type Table struct {
	Name        string       `json:"name"`
	Columns     []Column     `json:"columns"`
	Indexes     []Index      `json:"indexes"`
	ForeignKeys []ForeignKey `json:"foreign_keys"`
	Description *string      `json:"description"`
}

type Column struct {
	Name         string  `json:"name"`
	DataType     string  `json:"data_type"`
	IsNullable   bool    `json:"is_nullable"`
	IsPrimaryKey bool    `json:"is_primary_key"`
	DefaultValue *string `json:"default_value"`
	Description  *string `json:"description"`
}

type Index struct {
	Name     string   `json:"name"`
	Columns  []string `json:"columns"`
	IsUnique bool     `json:"is_unique"`
}

type ForeignKey struct {
	ConstraintName string `json:"constraint_name"`
	ColumnName     string `json:"column_name"`
	ForeignTable   string `json:"foreign_table"`
	ForeignColumn  string `json:"foreign_column"`
}

// GetSchema fetches and parses the database schema.
func GetSchema(dbURL string) (*Schema, error) {
	jsonStr, err := CrawlPostgres(dbURL)
	if err != nil {
		return nil, fmt.Errorf("aiql core error: %w", err)
	}

	var schema Schema
	if err := json.Unmarshal([]byte(jsonStr), &schema); err != nil {
		return nil, fmt.Errorf("failed to parse schema JSON: %w", err)
	}

	return &schema, nil
}
