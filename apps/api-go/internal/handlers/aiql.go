package handlers

import (
	"github.com/gin-gonic/gin"
	"net/http"
	"webapp/apps/api/internal/aiql"
)

type CrawlRequest struct {
	URL string `json:"url" binding:"required"`
}

func CrawlSchema(c *gin.Context) {
	var req CrawlRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	schema, err := aiql.CrawlPostgres(req.URL)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.Data(http.StatusOK, "application/json", []byte(schema))
}

type TranslateRequest struct {
	Prompt     string `json:"prompt" binding:"required"`
	SchemaJSON string `json:"schema_json" binding:"required"`
}

func Translate(c *gin.Context) {
	var req TranslateRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	result, err := aiql.Translate(req.Prompt, req.SchemaJSON)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.Data(http.StatusOK, "application/json", []byte(result))
}

type AskRequest struct {
	Prompt     string `json:"prompt" binding:"required"`
	DbURL      string `json:"db_url" binding:"required"`
	SchemaJSON string `json:"schema_json" binding:"required"`
}

func Ask(c *gin.Context) {
	var req AskRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	result, err := aiql.Ask(req.Prompt, req.DbURL, req.SchemaJSON)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.Data(http.StatusOK, "application/json", []byte(result))
}

type MockDataRequest struct {
	Prompt     string `json:"prompt" binding:"required"`
	SchemaJSON string `json:"schema_json" binding:"required"`
}

func GenerateMockData(c *gin.Context) {
	var req MockDataRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	result, err := aiql.GenerateMockData(req.Prompt, req.SchemaJSON)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.Data(http.StatusOK, "application/json", []byte(result))
}

func GetStats(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"version": "1.0.0-ALPHA",
		"status": "Healthy",
		"uptime": "99.9%",
		"average_latency_ms": 120,
		"cached_queries_count": 450,
		"active_sessions": 12,
	})
}
