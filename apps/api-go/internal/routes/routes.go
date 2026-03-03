package routes

import (
	"context"
	"log"
	"net/http"
	"time"

	"github.com/MUKE-coder/gin-docs/gindocs"
	"github.com/MUKE-coder/gorm-studio/studio"
	"github.com/MUKE-coder/pulse/pulse"
	"github.com/MUKE-coder/sentinel"
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

	"webapp/apps/api/internal/ai"
	"webapp/apps/api/internal/cache"
	"webapp/apps/api/internal/config"
	"webapp/apps/api/internal/handlers"
	"webapp/apps/api/internal/mail"
	"webapp/apps/api/internal/middleware"
	"webapp/apps/api/internal/models"
	"webapp/apps/api/internal/jobs"
	"webapp/apps/api/internal/services"
	"webapp/apps/api/internal/storage"
)

// Services holds all Phase 4 services for dependency injection.
type Services struct {
	Cache   *cache.Cache
	Storage *storage.Storage
	Mailer  *mail.Mailer
	AI      *ai.AI
	Jobs    *jobs.Client
}

// Setup configures all routes and returns the Gin engine.
func Setup(db *gorm.DB, cfg *config.Config, svc *Services) *gin.Engine {
	if cfg.AppEnv == "production" {
		gin.SetMode(gin.ReleaseMode)
	}

	r := gin.New()

	// Global middleware
	r.Use(middleware.RequestID())
	r.Use(middleware.Logger())
	r.Use(gin.Recovery())
	r.Use(middleware.CORS(cfg.CORSOrigins))
	r.Use(middleware.Gzip())

	// Mount Sentinel security suite (WAF, rate limiting, auth shield, anomaly detection)
	if cfg.SentinelEnabled {
		sentinel.Mount(r, db, sentinel.Config{
			Dashboard: sentinel.DashboardConfig{
				Username:  cfg.SentinelUsername,
				Password:  cfg.SentinelPassword,
				SecretKey: cfg.SentinelSecretKey,
			},
			WAF: sentinel.WAFConfig{
				Enabled: true,
				Mode:    sentinel.ModeLog, // Switch to sentinel.ModeBlock in production
			},
			RateLimit: sentinel.RateLimitConfig{
				Enabled: true,
				ByIP:    &sentinel.Limit{Requests: 100, Window: 1 * time.Minute},
				ByRoute: map[string]sentinel.Limit{
					"/api/auth/login":    {Requests: 5, Window: 15 * time.Minute},
					"/api/auth/register": {Requests: 3, Window: 15 * time.Minute},
				},
				ExcludeRoutes: []string{"/pulse/*", "/studio/*", "/sentinel/*", "/docs/*"},
			},
			AuthShield: sentinel.AuthShieldConfig{
				Enabled:    true,
				LoginRoute: "/api/auth/login",
			},
			Anomaly: sentinel.AnomalyConfig{
				Enabled: true,
			},
			Geo: sentinel.GeoConfig{
				Enabled: true,
			},
		})
		log.Println("Sentinel security suite mounted at /sentinel")
	}

	// Mount GORM Studio
	if cfg.GORMStudioEnabled {
		studioCfg := studio.Config{
			Prefix: "/studio",
		}
		if cfg.GORMStudioUsername != "" && cfg.GORMStudioPassword != "" {
			studioCfg.AuthMiddleware = gin.BasicAuth(gin.Accounts{
				cfg.GORMStudioUsername: cfg.GORMStudioPassword,
			})
		}
		studio.Mount(r, db, []interface{}{&models.User{}, &models.Upload{}, &models.Blog{}, &models.ExecutionLog{}, &models.Endpoint{}, /* grit:studio */}, studioCfg)
		log.Println("GORM Studio mounted at /studio")
	}

	// API Documentation (gin-docs — auto-generated from routes + models)
	gindocs.Mount(r, db, gindocs.Config{
		Title:       cfg.AppName + " API",
		Description: "REST API built with [Grit](https://gritframework.dev) — Go + React meta-framework.",
		Version:     "1.0.0",
		UI:          gindocs.UIScalar,
		ScalarTheme: "kepler",
		Models:      []interface{}{&models.User{}, &models.Upload{}, &models.Blog{}, &models.ExecutionLog{}, &models.Endpoint{}},
		Auth: gindocs.AuthConfig{
			Type:         gindocs.AuthBearer,
			BearerFormat: "JWT",
		},
	})
	log.Println("API docs available at /docs")

	// Mount Pulse observability (request tracing, DB monitoring, runtime metrics, error tracking)
	if cfg.PulseEnabled {
		p := pulse.Mount(r, db, pulse.Config{
			AppName: cfg.AppName,
			DevMode: cfg.IsDevelopment(),
			Dashboard: pulse.DashboardConfig{
				Username: cfg.PulseUsername,
				Password: cfg.PulsePassword,
			},
			Tracing: pulse.TracingConfig{
				ExcludePaths: []string{"/studio/*", "/sentinel/*", "/docs/*", "/pulse/*"},
			},
			Alerts: pulse.AlertConfig{},
			Prometheus: pulse.PrometheusConfig{
				Enabled: true,
			},
		})

		// Register health checks for connected services
		if svc.Cache != nil {
			p.AddHealthCheck(pulse.HealthCheck{
				Name:     "redis",
				Type:     "redis",
				Critical: false,
				CheckFunc: func(ctx context.Context) error {
					return svc.Cache.Client().Ping(ctx).Err()
				},
			})
		}

		log.Println("Pulse observability mounted at /pulse")
	}

	// Auth service
	authService := &services.AuthService{
		Secret:        cfg.JWTSecret,
		AccessExpiry:  cfg.JWTAccessExpiry,
		RefreshExpiry: cfg.JWTRefreshExpiry,
	}

	// Handlers
	authHandler := &handlers.AuthHandler{
		DB:          db,
		AuthService: authService,
		Config:      cfg,
	}
	userHandler := &handlers.UserHandler{
		DB: db,
	}
	uploadHandler := &handlers.UploadHandler{
		DB:      db,
		Storage: svc.Storage,
		Jobs:    svc.Jobs,
	}
	aiHandler := &handlers.AIHandler{
		AI: svc.AI,
	}
	jobsHandler := &handlers.JobsHandler{
		RedisURL: cfg.RedisURL,
	}
	cronHandler := &handlers.CronHandler{}
	blogHandler := handlers.NewBlogHandler(db)
	endpointHandler := &handlers.EndpointHandler{DB: db}
	// grit:handlers

	// Dynamic Endpoints
	r.GET("/e/:path", endpointHandler.Execute)
	r.POST("/api/endpoints", endpointHandler.Create)

	// Health check
	r.GET("/api/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status":  "ok",
			"version": "0.1.0",
		})
	})

	// Public blog routes (no auth required)
	blogs := r.Group("/api/blogs")
	{
		blogs.GET("", blogHandler.ListPublished)
		blogs.GET("/:slug", blogHandler.GetBySlug)
	}

	// AIQL Routes
	aiql := r.Group("/api/aiql")
	{
		aiql.POST("/crawl", handlers.CrawlSchema)
		aiql.POST("/translate", handlers.Translate)
		aiql.POST("/ask", handlers.Ask)
	}

	// Public auth routes
	auth := r.Group("/api/auth")
	{
		auth.POST("/register", authHandler.Register)
		auth.POST("/login", authHandler.Login)
		auth.POST("/refresh", authHandler.Refresh)
		auth.POST("/forgot-password", authHandler.ForgotPassword)
		auth.POST("/reset-password", authHandler.ResetPassword)
	}

	// OAuth2 social login
	oauth := auth.Group("/oauth")
	{
		oauth.GET("/:provider", authHandler.OAuthBegin)
		oauth.GET("/:provider/callback", authHandler.OAuthCallback)
	}

	// Protected routes
	protected := r.Group("/api")
	protected.Use(middleware.Auth(db, authService))
	{
		protected.GET("/auth/me", authHandler.Me)
		protected.POST("/auth/logout", authHandler.Logout)

		// User routes (authenticated)
		protected.GET("/users/:id", userHandler.GetByID)

		// File uploads
		protected.POST("/uploads", uploadHandler.Create)
		protected.POST("/uploads/presign", uploadHandler.Presign)
		protected.POST("/uploads/complete", uploadHandler.CompleteUpload)
		protected.GET("/uploads", uploadHandler.List)
		protected.GET("/uploads/:id", uploadHandler.GetByID)
		protected.DELETE("/uploads/:id", uploadHandler.Delete)

		// AI
		protected.POST("/ai/complete", aiHandler.Complete)
		protected.POST("/ai/chat", aiHandler.Chat)
		protected.POST("/ai/stream", aiHandler.Stream)

		// grit:routes:protected
	}

	// Profile routes (any authenticated user)
	profile := protected.Group("/profile")
	{
		profile.GET("", userHandler.GetProfile)
		profile.PUT("", userHandler.UpdateProfile)
		profile.DELETE("", userHandler.DeleteProfile)
	}

	// Admin routes
	admin := r.Group("/api")
	admin.Use(middleware.Auth(db, authService))
	admin.Use(middleware.RequireRole("ADMIN"))
	{
		admin.GET("/users", userHandler.List)
		admin.POST("/users", userHandler.Create)
		admin.PUT("/users/:id", userHandler.Update)
		admin.DELETE("/users/:id", userHandler.Delete)

		// Admin system routes
		admin.GET("/admin/jobs/stats", jobsHandler.Stats)
		admin.GET("/admin/jobs/:status", jobsHandler.ListByStatus)
		admin.POST("/admin/jobs/:id/retry", jobsHandler.Retry)
		admin.DELETE("/admin/jobs/queue/:queue", jobsHandler.ClearQueue)
		admin.GET("/admin/cron/tasks", cronHandler.ListTasks)

		// Blog management (admin)
		admin.GET("/admin/blogs", blogHandler.List)
		admin.POST("/admin/blogs", blogHandler.Create)
		admin.PUT("/admin/blogs/:id", blogHandler.Update)
		admin.DELETE("/admin/blogs/:id", blogHandler.Delete)

		// grit:routes:admin
	}

	// Custom role-restricted routes
	// grit:routes:custom

	return r
}
