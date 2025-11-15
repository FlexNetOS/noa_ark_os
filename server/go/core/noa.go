package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// NOACore represents the central orchestrating intelligence
type NOACore struct {
	logger         *logrus.Logger
	redis          *redis.Client
	db             *sql.DB
	router         *gin.Engine
	config         *Config
	agentManager   *AgentManager
	taskManager    *TaskManager
	eventBus       *EventBus
	systemMonitor  *SystemMonitor
}

// Config holds the NOA core configuration
type Config struct {
	ServerPort     string `yaml:"server_port"`
	RedisURL       string `yaml:"redis_url"`
	DatabaseURL    string `yaml:"database_url"`
	LogLevel       string `yaml:"log_level"`
	SystemName     string `yaml:"system_name"`
	MaxAgents      int    `yaml:"max_agents"`
	EnableMetrics  bool   `yaml:"enable_metrics"`
}

// AgentManager manages the hierarchical agent system
type AgentManager struct {
	noaCore *NOACore
}

// TaskManager manages task execution and coordination
type TaskManager struct {
	noaCore *NOACore
}

// EventBus handles inter-service communication
type EventBus struct {
	noaCore *NOACore
	redis   *redis.Client
}

// SystemMonitor monitors overall system health
type SystemMonitor struct {
	noaCore *NOACore
}

func main() {
	// Initialize logger
	logger := logrus.New()
	logger.SetFormatter(&logrus.JSONFormatter{})
	logger.SetLevel(logrus.InfoLevel)

	// Load configuration
	config := &Config{
		ServerPort:    getEnv("SERVER_PORT", "8080"),
		RedisURL:      getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:   getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:      getEnv("LOG_LEVEL", "info"),
		SystemName:    "Ark-OS-NOA",
		MaxAgents:     50,
		EnableMetrics: true,
	}

	// Set log level
	if level, err := logrus.ParseLevel(config.LogLevel); err == nil {
		logger.SetLevel(level)
	}

	logger.Info("🚀 Starting Ark-OS-NOA Core Service")

	// Initialize NOA Core
	noaCore, err := NewNOACore(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize NOA Core:", err)
	}

	// Start the service
	if err := noaCore.Start(); err != nil {
		logger.Fatal("Failed to start NOA Core:", err)
	}
}

// NewNOACore creates a new NOA core instance
func NewNOACore(config *Config, logger *logrus.Logger) (*NOACore, error) {
	// Initialize Redis client
	rdb := redis.NewClient(&redis.Options{
		Addr: config.RedisURL,
	})

	// Test Redis connection
	if err := rdb.Ping(context.Background()).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	// Initialize database connection
	db, err := sql.Open("postgres", config.DatabaseURL)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	// Test database connection
	if err := db.Ping(); err != nil {
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	// Initialize Gin router
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()

	// Add middleware
	router.Use(gin.Logger())
	router.Use(gin.Recovery())
	router.Use(corsMiddleware())

	noaCore := &NOACore{
		logger:        logger,
		redis:         rdb,
		db:            db,
		router:        router,
		config:        config,
		agentManager:  &AgentManager{},
		taskManager:   &TaskManager{},
		eventBus:      &EventBus{redis: rdb},
		systemMonitor: &SystemMonitor{},
	}

	// Set self-reference for managers
	noaCore.agentManager.noaCore = noaCore
	noaCore.taskManager.noaCore = noaCore
	noaCore.eventBus.noaCore = noaCore
	noaCore.systemMonitor.noaCore = noaCore

	// Setup routes
	noaCore.setupRoutes()

	return noaCore, nil
}

// Start begins the NOA core service
func (n *NOACore) Start() error {
	// Start background services
	go n.startBackgroundServices()

	// Start HTTP server
	srv := &http.Server{
		Addr:    ":" + n.config.ServerPort,
		Handler: n.router,
	}

	// Start server in a goroutine
	go func() {
		n.logger.Infof("🌐 NOA Core listening on port %s", n.config.ServerPort)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			n.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal to gracefully shutdown the server
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	n.logger.Info("🛑 Shutting down NOA Core...")

	// Graceful shutdown
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		return fmt.Errorf("server forced to shutdown: %w", err)
	}

	n.logger.Info("✅ NOA Core stopped")
	return nil
}

// setupRoutes configures all HTTP routes
func (n *NOACore) setupRoutes() {
	// Health check endpoint
	n.router.GET("/health", n.healthCheck)

	// Metrics endpoint
	if n.config.EnableMetrics {
		n.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
	}

	// API v1 routes
	v1 := n.router.Group("/api/v1")
	{
		// System endpoints
		v1.GET("/status", n.getSystemStatus)
		v1.POST("/shutdown", n.initiateShutdown)

		// Agent management
		v1.GET("/agents", n.agentManager.listAgents)
		v1.POST("/agents", n.agentManager.createAgent)
		v1.GET("/agents/:id", n.agentManager.getAgent)
		v1.PUT("/agents/:id", n.agentManager.updateAgent)
		v1.DELETE("/agents/:id", n.agentManager.deleteAgent)

		// Task management
		v1.GET("/tasks", n.taskManager.listTasks)
		v1.POST("/tasks", n.taskManager.createTask)
		v1.GET("/tasks/:id", n.taskManager.getTask)
		v1.PUT("/tasks/:id", n.taskManager.updateTask)
		v1.DELETE("/tasks/:id", n.taskManager.cancelTask)

		// Event bus
		v1.POST("/events", n.eventBus.publishEvent)
		v1.GET("/events", n.eventBus.getEvents)

		// System monitoring
		v1.GET("/system/metrics", n.systemMonitor.getMetrics)
		v1.GET("/system/health", n.systemMonitor.getHealth)
	}
}

// startBackgroundServices starts all background services
func (n *NOACore) startBackgroundServices() {
	n.logger.Info("🔄 Starting background services...")

	// Start agent heartbeat monitoring
	go n.agentManager.monitorAgentHealth()

	// Start task scheduler
	go n.taskManager.processTaskQueue()

	// Start system monitoring
	go n.systemMonitor.startMonitoring()

	// Start event bus processor
	go n.eventBus.processEvents()

	n.logger.Info("✅ Background services started")
}

// healthCheck provides a simple health check endpoint
func (n *NOACore) healthCheck(c *gin.Context) {
	health := map[string]interface{}{
		"status":    "healthy",
		"service":   "noa-core",
		"timestamp": time.Now().UTC(),
		"version":   "5.0.0",
	}

	// Check Redis connectivity
	if err := n.redis.Ping(context.Background()).Err(); err != nil {
		health["status"] = "unhealthy"
		health["redis_error"] = err.Error()
		c.JSON(http.StatusServiceUnavailable, health)
		return
	}

	// Check database connectivity
	if err := n.db.Ping(); err != nil {
		health["status"] = "unhealthy"
		health["database_error"] = err.Error()
		c.JSON(http.StatusServiceUnavailable, health)
		return
	}

	c.JSON(http.StatusOK, health)
}

// getSystemStatus returns overall system status
func (n *NOACore) getSystemStatus(c *gin.Context) {
	status := map[string]interface{}{
		"system_name":     n.config.SystemName,
		"version":         "5.0.0",
		"status":          "operational",
		"timestamp":       time.Now().UTC(),
		"uptime":          "unknown", // TODO: Implement uptime tracking
		"active_agents":   0,         // TODO: Get from agent manager
		"pending_tasks":   0,         // TODO: Get from task manager
		"system_health":   "good",    // TODO: Get from system monitor
	}

	c.JSON(http.StatusOK, status)
}

// initiateShutdown initiates a graceful system shutdown
func (n *NOACore) initiateShutdown(c *gin.Context) {
	n.logger.Info("🛑 Shutdown initiated via API")

	// TODO: Implement graceful shutdown logic
	// - Stop accepting new tasks
	// - Wait for running tasks to complete
	// - Save system state
	// - Shutdown all services

	c.JSON(http.StatusOK, gin.H{
		"message": "Shutdown initiated",
		"status":  "shutting_down",
	})
}

// corsMiddleware adds CORS headers
func corsMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Header("Access-Control-Allow-Origin", "*")
		c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		c.Header("Access-Control-Allow-Headers", "Origin, Content-Type, Accept, Authorization")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(http.StatusNoContent)
			return
		}

		c.Next()
	}
}

// getEnv gets environment variable or returns default value
func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}</content>
<parameter name="filePath">/home/deflex/workspaces/deflex-ai-os/project-workspace/ark-os-noa-v5/services/noa-core/main.go
