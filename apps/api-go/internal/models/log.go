package models

import (
	"gorm.io/gorm"
)

type ExecutionLog struct {
	gorm.Model
	Prompt      string `json:"prompt"`
	SQL         string `json:"sql"`
	ExecutionTime int64  `json:"execution_time"`
	Success     bool   `json:"success"`
	Error       string `json:"error"`
	Dialect     string `json:"dialect"`
}
