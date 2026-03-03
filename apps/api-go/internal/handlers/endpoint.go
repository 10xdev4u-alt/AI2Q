package handlers

import (
	"github.com/gin-gonic/gin"
	"net/http"
	"webapp/apps/api/internal/models"
	"gorm.io/gorm"
)

type EndpointHandler struct {
	DB *gorm.DB
}

func (h *EndpointHandler) Execute(c *gin.Context) {
	path := c.Param("path")
	var endpoint models.Endpoint
	if err := h.DB.Where("path = ?", path).First(&endpoint).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Endpoint not found"})
		return
	}

	var results []map[string]interface{}
	if err := h.DB.Raw(endpoint.SQL).Scan(&results).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, results)
}

func (h *EndpointHandler) Create(c *gin.Context) {
	var endpoint models.Endpoint
	if err := c.ShouldBindJSON(&endpoint); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.DB.Create(&endpoint).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, endpoint)
}
