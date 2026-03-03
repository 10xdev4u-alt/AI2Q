package models

import (
	"gorm.io/gorm"
)

type Endpoint struct {
	gorm.Model
	Name        string `gorm:"uniqueIndex;not null" json:"name"`
	Path        string `gorm:"uniqueIndex;not null" json:"path"`
	Prompt      string `gorm:"not null" json:"prompt"`
	SQL         string `gorm:"not null" json:"sql"`
	Description string `json:"description"`
	Method      string `gorm:"default:GET" json:"method"`
}
