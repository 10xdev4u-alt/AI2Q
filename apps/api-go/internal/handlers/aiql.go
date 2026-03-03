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
