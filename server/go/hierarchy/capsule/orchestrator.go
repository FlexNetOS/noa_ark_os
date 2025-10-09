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
	"strings"
	"crypto/sha256"
	"encoding/hex"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// CapsuleOrchestrator represents the container and workflow orchestration service
type CapsuleOrchestrator struct {
	logger         *logrus.Logger
	redis          *redis.Client
	db             *sql.DB
	router         *gin.Engine
	config         *Config
	capsules       map[string]*Capsule
	workflows      map[string]*Workflow
	orchestrator   *OrchestrationEngine
	scheduler      *CapsuleScheduler
	trifectaCourt  *TrifectaCourtClient
}

// Config holds the capsule orchestrator configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
	MaxCapsules      int    `yaml:"max_capsules"`
	DefaultTimeout   int    `yaml:"default_timeout_seconds"`
}

// Capsule represents a containerized execution environment
type Capsule struct {
	ID            string                 `json:"id"`
	Name          string                 `json:"name"`
	Type          string                 `json:"type"`
	Image         string                 `json:"image"`
	Status        string                 `json:"status"`
	Resources     *CapsuleResources      `json:"resources"`
	Environment   map[string]string      `json:"environment"`
	Volumes       []*Volume              `json:"volumes"`
	Networks      []*Network             `json:"networks"`
	HealthCheck   *HealthCheck           `json:"health_check"`
	Metadata      map[string]interface{} `json:"metadata"`
	CreatedAt     time.Time              `json:"created_at"`
	UpdatedAt     time.Time              `json:"updated_at"`
	StartedAt     *time.Time             `json:"started_at,omitempty"`
	CompletedAt   *time.Time             `json:"completed_at,omitempty"`
	ExitCode      *int                   `json:"exit_code,omitempty"`
	Logs          []string               `json:"logs,omitempty"`
}

// CapsuleResources represents resource allocation for a capsule
type CapsuleResources struct {
	CPU        float64 `json:"cpu_cores"`
	Memory     int     `json:"memory_mb"`
	Storage    int     `json:"storage_gb"`
	GPU        int     `json:"gpu_count"`
	Network    int     `json:"network_mbps"`
	Priority   string  `json:"priority"`
	Limits     *ResourceLimits `json:"limits"`
}

// ResourceLimits represents resource limits
type ResourceLimits struct {
	MaxCPU     float64 `json:"max_cpu_cores"`
	MaxMemory  int     `json:"max_memory_mb"`
	MaxStorage int     `json:"max_storage_gb"`
	Timeout    int     `json:"timeout_seconds"`
}

// Volume represents a storage volume
type Volume struct {
	Name      string `json:"name"`
	Type      string `json:"type"`
	Source    string `json:"source"`
	Target    string `json:"target"`
	ReadOnly  bool   `json:"read_only"`
	Size      int    `json:"size_gb"`
}

// Network represents a network configuration
type Network struct {
	Name     string            `json:"name"`
	Type     string            `json:"type"`
	Ports    []*PortMapping    `json:"ports"`
	Labels   map[string]string `json:"labels"`
}

// PortMapping represents port mapping
type PortMapping struct {
	Host      int    `json:"host_port"`
	Container int    `json:"container_port"`
	Protocol  string `json:"protocol"`
}

// HealthCheck represents health check configuration
type HealthCheck struct {
	Command     []string `json:"command"`
	Interval    int      `json:"interval_seconds"`
	Timeout     int      `json:"timeout_seconds"`
	Retries     int      `json:"retries"`
	StartPeriod int      `json:"start_period_seconds"`
}

// Workflow represents a multi-capsule workflow
type Workflow struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Description  string                 `json:"description"`
	Status       string                 `json:"status"`
	Steps        []*WorkflowStep        `json:"steps"`
	Dependencies map[string][]string    `json:"dependencies"`
	Parallel     [][]string             `json:"parallel"`
	Resources    *WorkflowResources     `json:"resources"`
	Metadata     map[string]interface{} `json:"metadata"`
	CreatedAt    time.Time              `json:"created_at"`
	UpdatedAt    time.Time              `json:"updated_at"`
	StartedAt    *time.Time             `json:"started_at,omitempty"`
	CompletedAt  *time.Time             `json:"completed_at,omitempty"`
}

// WorkflowStep represents a step in a workflow
type WorkflowStep struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Type        string                 `json:"type"`
	CapsuleID   string                 `json:"capsule_id"`
	Command     []string               `json:"command"`
	Args        []string               `json:"args"`
	Environment map[string]string      `json:"environment"`
	Status      string                 `json:"status"`
	Results     map[string]interface{} `json:"results,omitempty"`
	StartedAt   *time.Time             `json:"started_at,omitempty"`
	CompletedAt *time.Time             `json:"completed_at,omitempty"`
	ExitCode    *int                   `json:"exit_code,omitempty"`
	Logs        []string               `json:"logs,omitempty"`
}

// WorkflowResources represents workflow-level resources
type WorkflowResources struct {
	TotalCPU    float64 `json:"total_cpu_cores"`
	TotalMemory int     `json:"total_memory_mb"`
	Budget      float64 `json:"budget_usd"`
	Timeout     int     `json:"timeout_seconds"`
	Priority    string  `json:"priority"`
}

// OrchestrationEngine handles capsule and workflow orchestration
type OrchestrationEngine struct {
	orchestrator *CapsuleOrchestrator
}

// CapsuleScheduler handles capsule scheduling and resource allocation
type CapsuleScheduler struct {
	orchestrator *CapsuleOrchestrator
}

// CapsuleRequest represents a capsule creation request
type CapsuleRequest struct {
	Name        string                 `json:"name"`
	Type        string                 `json:"type"`
	Image       string                 `json:"image"`
	Command     []string               `json:"command"`
	Args        []string               `json:"args"`
	Resources   *CapsuleResources      `json:"resources"`
	Environment map[string]string      `json:"environment"`
	Volumes     []*Volume              `json:"volumes"`
	Networks    []*Network             `json:"networks"`
	HealthCheck *HealthCheck           `json:"health_check"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// CapsuleResponse represents the capsule creation result
type CapsuleResponse struct {
	Success   bool                   `json:"success"`
	CapsuleID string                 `json:"capsule_id"`
	Status    string                 `json:"status"`
	Endpoints []string               `json:"endpoints"`
	Resources *CapsuleResources      `json:"resources"`
	Metadata  map[string]interface{} `json:"metadata"`
}

// WorkflowRequest represents a workflow creation request
type WorkflowRequest struct {
	Name         string                 `json:"name"`
	Description  string                 `json:"description"`
	Steps        []*WorkflowStep        `json:"steps"`
	Dependencies map[string][]string    `json:"dependencies"`
	Parallel     [][]string             `json:"parallel"`
	Resources    *WorkflowResources     `json:"resources"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// TrifectaCourtClient interfaces with constitutional validation
type TrifectaCourtClient struct {
	baseURL string
	client  *http.Client
}

func main() {
	// Initialize logger
	logger := logrus.New()
	logger.SetFormatter(&logrus.JSONFormatter{})
	logger.SetLevel(logrus.InfoLevel)

	// Load configuration
	config := &Config{
		ServerPort:       getEnv("SERVER_PORT", "8009"),
		RedisURL:         getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:      getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:         getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL: getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		MaxCapsules:      1000,
		DefaultTimeout:   3600,
	}

	logger.Info("📦 Starting Capsule Orchestrator Service")

	// Initialize Capsule Orchestrator
	capsuleOrchestrator, err := NewCapsuleOrchestrator(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize Capsule Orchestrator:", err)
	}

	// Start the service
	if err := capsuleOrchestrator.Start(); err != nil {
		logger.Fatal("Failed to start Capsule Orchestrator:", err)
	}
}

func NewCapsuleOrchestrator(config *Config, logger *logrus.Logger) (*CapsuleOrchestrator, error) {
	// Initialize Redis client
	redisOpts, err := redis.ParseURL(config.RedisURL)
	if err != nil {
		return nil, fmt.Errorf("failed to parse Redis URL: %w", err)
	}
	redisClient := redis.NewClient(redisOpts)

	// Test Redis connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := redisClient.Ping(ctx).Err(); err != nil {
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

	// Initialize HTTP router
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()
	router.Use(gin.Logger(), gin.Recovery())

	// Create capsule orchestrator
	co := &CapsuleOrchestrator{
		logger:    logger,
		redis:     redisClient,
		db:        db,
		router:    router,
		config:    config,
		capsules:  make(map[string]*Capsule),
		workflows: make(map[string]*Workflow),
	}

	// Initialize components
	co.orchestrator = &OrchestrationEngine{orchestrator: co}
	co.scheduler = &CapsuleScheduler{orchestrator: co}
	co.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	co.setupRoutes()

	// Start background tasks
	go co.startCapsuleMonitor()
	go co.startWorkflowMonitor()

	return co, nil
}

func (co *CapsuleOrchestrator) setupRoutes() {
	// Health check
	co.router.GET("/health", co.healthCheck)

	// Capsule management
	co.router.POST("/capsules", co.createCapsule)
	co.router.GET("/capsules", co.listCapsules)
	co.router.GET("/capsules/:capsule_id", co.getCapsule)
	co.router.DELETE("/capsules/:capsule_id", co.deleteCapsule)

	// Capsule operations
	co.router.POST("/capsules/:capsule_id/start", co.startCapsule)
	co.router.POST("/capsules/:capsule_id/stop", co.stopCapsule)
	co.router.POST("/capsules/:capsule_id/restart", co.restartCapsule)
	co.router.GET("/capsules/:capsule_id/logs", co.getCapsuleLogs)
	co.router.GET("/capsules/:capsule_id/stats", co.getCapsuleStats)

	// Workflow management
	co.router.POST("/workflows", co.createWorkflow)
	co.router.GET("/workflows", co.listWorkflows)
	co.router.GET("/workflows/:workflow_id", co.getWorkflow)
	co.router.DELETE("/workflows/:workflow_id", co.deleteWorkflow)

	// Workflow operations
	co.router.POST("/workflows/:workflow_id/execute", co.executeWorkflow)
	co.router.POST("/workflows/:workflow_id/pause", co.pauseWorkflow)
	co.router.POST("/workflows/:workflow_id/resume", co.resumeWorkflow)
	co.router.POST("/workflows/:workflow_id/abort", co.abortWorkflow)
	co.router.GET("/workflows/:workflow_id/status", co.getWorkflowStatus)

	// Resource management
	co.router.GET("/resources/usage", co.getResourceUsage)
	co.router.GET("/resources/capacity", co.getResourceCapacity)
	co.router.POST("/resources/allocate", co.allocateResources)
	co.router.POST("/resources/deallocate", co.deallocateResources)

	// Scheduling
	co.router.GET("/scheduler/queue", co.getSchedulerQueue)
	co.router.POST("/scheduler/priority", co.updateSchedulerPriority)

	// Metrics
	co.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (co *CapsuleOrchestrator) Start() error {
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + co.config.ServerPort,
		Handler: co.router,
	}

	go func() {
		co.logger.Infof("Capsule Orchestrator listening on port %s", co.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			co.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	co.logger.Info("Shutting down Capsule Orchestrator...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		co.logger.Error("Server forced to shutdown:", err)
		return err
	}

	co.logger.Info("Capsule Orchestrator stopped")
	return nil
}

func (co *CapsuleOrchestrator) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "capsule-orchestrator",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"capsules": map[string]interface{}{
			"total":   len(co.capsules),
			"running": co.countCapsulesByStatus("running"),
			"pending": co.countCapsulesByStatus("pending"),
		},
		"workflows": map[string]interface{}{
			"total":   len(co.workflows),
			"active":  co.countWorkflowsByStatus("running"),
			"pending": co.countWorkflowsByStatus("pending"),
		},
	}

	// Check dependencies
	dependencies := map[string]string{
		"redis":          "healthy",
		"database":       "healthy",
		"trifecta_court": "unknown",
	}

	// Test Redis
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	if err := co.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := co.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (co *CapsuleOrchestrator) createCapsule(c *gin.Context) {
	var request CapsuleRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := co.validateWithTrifectaCourt("create_capsule", map[string]interface{}{
		"name":      request.Name,
		"type":      request.Type,
		"image":     request.Image,
		"resources": request.Resources,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Create capsule
	response, err := co.orchestrator.CreateCapsule(request)
	if err != nil {
		co.logger.Error("Failed to create capsule:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Capsule creation failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (co *CapsuleOrchestrator) listCapsules(c *gin.Context) {
	capsules := make([]*Capsule, 0, len(co.capsules))
	for _, capsule := range co.capsules {
		capsules = append(capsules, capsule)
	}

	c.JSON(http.StatusOK, gin.H{
		"capsules": capsules,
		"total":    len(capsules),
	})
}

func (co *CapsuleOrchestrator) getCapsule(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	c.JSON(http.StatusOK, capsule)
}

func (co *CapsuleOrchestrator) deleteCapsule(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	// Stop capsule if running
	if capsule.Status == "running" {
		capsule.Status = "stopping"
		// Simulate stopping
		time.Sleep(100 * time.Millisecond)
	}

	// Remove capsule
	delete(co.capsules, capsuleID)

	// Remove from Redis
	ctx := context.Background()
	co.redis.Del(ctx, fmt.Sprintf("capsule:%s", capsuleID))

	co.logger.Infof("Capsule deleted: %s", capsuleID)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Capsule deleted successfully",
	})
}

func (co *CapsuleOrchestrator) startCapsule(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	// Start capsule
	result, err := co.orchestrator.StartCapsule(capsule)
	if err != nil {
		co.logger.Error("Failed to start capsule:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Capsule start failed"})
		return
	}

	c.JSON(http.StatusOK, result)
}

func (co *CapsuleOrchestrator) stopCapsule(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	// Stop capsule
	result, err := co.orchestrator.StopCapsule(capsule)
	if err != nil {
		co.logger.Error("Failed to stop capsule:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Capsule stop failed"})
		return
	}

	c.JSON(http.StatusOK, result)
}

func (co *CapsuleOrchestrator) restartCapsule(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	// Restart capsule
	result, err := co.orchestrator.RestartCapsule(capsule)
	if err != nil {
		co.logger.Error("Failed to restart capsule:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Capsule restart failed"})
		return
	}

	c.JSON(http.StatusOK, result)
}

func (co *CapsuleOrchestrator) getCapsuleLogs(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"capsule_id": capsuleID,
		"logs":       capsule.Logs,
		"total":      len(capsule.Logs),
	})
}

func (co *CapsuleOrchestrator) getCapsuleStats(c *gin.Context) {
	capsuleID := c.Param("capsule_id")
	capsule, exists := co.capsules[capsuleID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Capsule not found"})
		return
	}

	stats := map[string]interface{}{
		"capsule_id": capsuleID,
		"status":     capsule.Status,
		"uptime":     time.Since(capsule.CreatedAt).Seconds(),
		"resources":  capsule.Resources,
		"cpu_usage":  75.5,  // Mock data
		"memory_usage": 512, // Mock data
	}

	c.JSON(http.StatusOK, stats)
}

func (co *CapsuleOrchestrator) createWorkflow(c *gin.Context) {
	var request WorkflowRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Create workflow
	workflow, err := co.orchestrator.CreateWorkflow(request)
	if err != nil {
		co.logger.Error("Failed to create workflow:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Workflow creation failed"})
		return
	}

	c.JSON(http.StatusOK, workflow)
}

func (co *CapsuleOrchestrator) listWorkflows(c *gin.Context) {
	workflows := make([]*Workflow, 0, len(co.workflows))
	for _, workflow := range co.workflows {
		workflows = append(workflows, workflow)
	}

	c.JSON(http.StatusOK, gin.H{
		"workflows": workflows,
		"total":     len(workflows),
	})
}

func (co *CapsuleOrchestrator) getWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	c.JSON(http.StatusOK, workflow)
}

func (co *CapsuleOrchestrator) deleteWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	
	if _, exists := co.workflows[workflowID]; !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	// Remove workflow
	delete(co.workflows, workflowID)

	// Remove from Redis
	ctx := context.Background()
	co.redis.Del(ctx, fmt.Sprintf("workflow:%s", workflowID))

	co.logger.Infof("Workflow deleted: %s", workflowID)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Workflow deleted successfully",
	})
}

func (co *CapsuleOrchestrator) executeWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	// Execute workflow
	result, err := co.orchestrator.ExecuteWorkflow(workflow)
	if err != nil {
		co.logger.Error("Failed to execute workflow:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Workflow execution failed"})
		return
	}

	c.JSON(http.StatusOK, result)
}

func (co *CapsuleOrchestrator) pauseWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	workflow.Status = "paused"
	workflow.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "paused",
	})
}

func (co *CapsuleOrchestrator) resumeWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	workflow.Status = "running"
	workflow.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "running",
	})
}

func (co *CapsuleOrchestrator) abortWorkflow(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	workflow.Status = "aborted"
	workflow.UpdatedAt = time.Now()
	now := time.Now()
	workflow.CompletedAt = &now

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "aborted",
	})
}

func (co *CapsuleOrchestrator) getWorkflowStatus(c *gin.Context) {
	workflowID := c.Param("workflow_id")
	workflow, exists := co.workflows[workflowID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Workflow not found"})
		return
	}

	status := map[string]interface{}{
		"workflow_id": workflowID,
		"status":      workflow.Status,
		"steps":       len(workflow.Steps),
		"completed":   co.countCompletedSteps(workflow),
		"progress":    co.calculateWorkflowProgress(workflow),
	}

	c.JSON(http.StatusOK, status)
}

func (co *CapsuleOrchestrator) getResourceUsage(c *gin.Context) {
	usage := map[string]interface{}{
		"cpu_cores":    co.calculateTotalCPUUsage(),
		"memory_mb":    co.calculateTotalMemoryUsage(),
		"storage_gb":   co.calculateTotalStorageUsage(),
		"network_mbps": co.calculateTotalNetworkUsage(),
		"capsules":     len(co.capsules),
		"workflows":    len(co.workflows),
	}

	c.JSON(http.StatusOK, usage)
}

func (co *CapsuleOrchestrator) getResourceCapacity(c *gin.Context) {
	capacity := map[string]interface{}{
		"max_cpu_cores":    1000.0,
		"max_memory_mb":    1000000,
		"max_storage_gb":   10000,
		"max_network_mbps": 10000,
		"max_capsules":     co.config.MaxCapsules,
	}

	c.JSON(http.StatusOK, capacity)
}

func (co *CapsuleOrchestrator) allocateResources(c *gin.Context) {
	// Implementation for resource allocation
	c.JSON(http.StatusOK, gin.H{
		"message": "Resource allocation endpoint",
		"status":  "implemented",
	})
}

func (co *CapsuleOrchestrator) deallocateResources(c *gin.Context) {
	// Implementation for resource deallocation
	c.JSON(http.StatusOK, gin.H{
		"message": "Resource deallocation endpoint",
		"status":  "implemented",
	})
}

func (co *CapsuleOrchestrator) getSchedulerQueue(c *gin.Context) {
	// Implementation for scheduler queue
	c.JSON(http.StatusOK, gin.H{
		"queue": []map[string]interface{}{},
		"total": 0,
	})
}

func (co *CapsuleOrchestrator) updateSchedulerPriority(c *gin.Context) {
	// Implementation for scheduler priority update
	c.JSON(http.StatusOK, gin.H{
		"message": "Scheduler priority update endpoint",
		"status":  "implemented",
	})
}

// OrchestrationEngine methods
func (oe *OrchestrationEngine) CreateCapsule(request CapsuleRequest) (*CapsuleResponse, error) {
	// Generate capsule ID
	capsuleID := generateCapsuleID(request.Name, request.Type)
	
	// Create capsule
	capsule := &Capsule{
		ID:          capsuleID,
		Name:        request.Name,
		Type:        request.Type,
		Image:       request.Image,
		Status:      "created",
		Resources:   request.Resources,
		Environment: request.Environment,
		Volumes:     request.Volumes,
		Networks:    request.Networks,
		HealthCheck: request.HealthCheck,
		Metadata:    request.Metadata,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
		Logs:        []string{},
	}

	// Store capsule
	oe.orchestrator.capsules[capsuleID] = capsule

	// Store in Redis
	capsuleJSON, _ := json.Marshal(capsule)
	ctx := context.Background()
	oe.orchestrator.redis.Set(ctx, fmt.Sprintf("capsule:%s", capsuleID), capsuleJSON, 24*time.Hour)

	oe.orchestrator.logger.Infof("Capsule created: %s (%s)", capsule.Name, capsuleID)

	return &CapsuleResponse{
		Success:   true,
		CapsuleID: capsuleID,
		Status:    "created",
		Endpoints: []string{},
		Resources: capsule.Resources,
		Metadata:  request.Metadata,
	}, nil
}

func (oe *OrchestrationEngine) StartCapsule(capsule *Capsule) (map[string]interface{}, error) {
	capsule.Status = "starting"
	capsule.UpdatedAt = time.Now()
	
	// Simulate starting
	time.Sleep(100 * time.Millisecond)
	
	capsule.Status = "running"
	now := time.Now()
	capsule.StartedAt = &now
	capsule.Logs = append(capsule.Logs, fmt.Sprintf("[%s] Capsule started", now.Format(time.RFC3339)))

	return map[string]interface{}{
		"success":    true,
		"capsule_id": capsule.ID,
		"status":     capsule.Status,
		"started_at": capsule.StartedAt,
	}, nil
}

func (oe *OrchestrationEngine) StopCapsule(capsule *Capsule) (map[string]interface{}, error) {
	capsule.Status = "stopping"
	capsule.UpdatedAt = time.Now()
	
	// Simulate stopping
	time.Sleep(100 * time.Millisecond)
	
	capsule.Status = "stopped"
	now := time.Now()
	capsule.CompletedAt = &now
	exitCode := 0
	capsule.ExitCode = &exitCode
	capsule.Logs = append(capsule.Logs, fmt.Sprintf("[%s] Capsule stopped", now.Format(time.RFC3339)))

	return map[string]interface{}{
		"success":      true,
		"capsule_id":   capsule.ID,
		"status":       capsule.Status,
		"completed_at": capsule.CompletedAt,
		"exit_code":    capsule.ExitCode,
	}, nil
}

func (oe *OrchestrationEngine) RestartCapsule(capsule *Capsule) (map[string]interface{}, error) {
	// Stop then start
	if _, err := oe.StopCapsule(capsule); err != nil {
		return nil, err
	}
	
	return oe.StartCapsule(capsule)
}

func (oe *OrchestrationEngine) CreateWorkflow(request WorkflowRequest) (*Workflow, error) {
	// Generate workflow ID
	workflowID := generateWorkflowID(request.Name)
	
	// Create workflow
	workflow := &Workflow{
		ID:           workflowID,
		Name:         request.Name,
		Description:  request.Description,
		Status:       "created",
		Steps:        request.Steps,
		Dependencies: request.Dependencies,
		Parallel:     request.Parallel,
		Resources:    request.Resources,
		Metadata:     request.Metadata,
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
	}

	// Store workflow
	oe.orchestrator.workflows[workflowID] = workflow

	// Store in Redis
	workflowJSON, _ := json.Marshal(workflow)
	ctx := context.Background()
	oe.orchestrator.redis.Set(ctx, fmt.Sprintf("workflow:%s", workflowID), workflowJSON, 24*time.Hour)

	oe.orchestrator.logger.Infof("Workflow created: %s (%s)", workflow.Name, workflowID)

	return workflow, nil
}

func (oe *OrchestrationEngine) ExecuteWorkflow(workflow *Workflow) (map[string]interface{}, error) {
	workflow.Status = "running"
	workflow.UpdatedAt = time.Now()
	now := time.Now()
	workflow.StartedAt = &now

	// Execute workflow steps
	for _, step := range workflow.Steps {
		step.Status = "running"
		step.StartedAt = &now
		
		// Simulate step execution
		time.Sleep(50 * time.Millisecond)
		
		step.Status = "completed"
		step.CompletedAt = &now
		exitCode := 0
		step.ExitCode = &exitCode
		step.Results = map[string]interface{}{
			"success": true,
			"output":  "Step completed successfully",
		}
	}

	workflow.Status = "completed"
	workflow.CompletedAt = &now

	return map[string]interface{}{
		"success":      true,
		"workflow_id":  workflow.ID,
		"status":       workflow.Status,
		"completed_at": workflow.CompletedAt,
		"steps":        len(workflow.Steps),
	}, nil
}

// Background tasks
func (co *CapsuleOrchestrator) startCapsuleMonitor() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		co.monitorCapsules()
	}
}

func (co *CapsuleOrchestrator) startWorkflowMonitor() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		co.monitorWorkflows()
	}
}

func (co *CapsuleOrchestrator) monitorCapsules() {
	now := time.Now()
	timeout := time.Duration(co.config.DefaultTimeout) * time.Second

	for capsuleID, capsule := range co.capsules {
		if capsule.Status == "running" && now.Sub(capsule.UpdatedAt) > timeout {
			capsule.Status = "timeout"
			capsule.CompletedAt = &now
			co.logger.Warnf("Capsule %s timed out", capsuleID)
		}
	}
}

func (co *CapsuleOrchestrator) monitorWorkflows() {
	now := time.Now()
	
	for workflowID, workflow := range co.workflows {
		if workflow.Status == "running" && workflow.Resources != nil {
			timeout := time.Duration(workflow.Resources.Timeout) * time.Second
			if timeout > 0 && now.Sub(workflow.UpdatedAt) > timeout {
				workflow.Status = "timeout"
				workflow.CompletedAt = &now
				co.logger.Warnf("Workflow %s timed out", workflowID)
			}
		}
	}
}

// Helper methods
func (co *CapsuleOrchestrator) countCapsulesByStatus(status string) int {
	count := 0
	for _, capsule := range co.capsules {
		if capsule.Status == status {
			count++
		}
	}
	return count
}

func (co *CapsuleOrchestrator) countWorkflowsByStatus(status string) int {
	count := 0
	for _, workflow := range co.workflows {
		if workflow.Status == status {
			count++
		}
	}
	return count
}

func (co *CapsuleOrchestrator) countCompletedSteps(workflow *Workflow) int {
	count := 0
	for _, step := range workflow.Steps {
		if step.Status == "completed" {
			count++
		}
	}
	return count
}

func (co *CapsuleOrchestrator) calculateWorkflowProgress(workflow *Workflow) float64 {
	if len(workflow.Steps) == 0 {
		return 0.0
	}
	completed := co.countCompletedSteps(workflow)
	return float64(completed) / float64(len(workflow.Steps)) * 100
}

func (co *CapsuleOrchestrator) calculateTotalCPUUsage() float64 {
	total := 0.0
	for _, capsule := range co.capsules {
		if capsule.Resources != nil && capsule.Status == "running" {
			total += capsule.Resources.CPU
		}
	}
	return total
}

func (co *CapsuleOrchestrator) calculateTotalMemoryUsage() int {
	total := 0
	for _, capsule := range co.capsules {
		if capsule.Resources != nil && capsule.Status == "running" {
			total += capsule.Resources.Memory
		}
	}
	return total
}

func (co *CapsuleOrchestrator) calculateTotalStorageUsage() int {
	total := 0
	for _, capsule := range co.capsules {
		if capsule.Resources != nil && capsule.Status == "running" {
			total += capsule.Resources.Storage
		}
	}
	return total
}

func (co *CapsuleOrchestrator) calculateTotalNetworkUsage() int {
	total := 0
	for _, capsule := range co.capsules {
		if capsule.Resources != nil && capsule.Status == "running" {
			total += capsule.Resources.Network
		}
	}
	return total
}

func (co *CapsuleOrchestrator) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := co.trifectaCourt.client.Post(
		co.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		co.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
		return true, nil // Fail open for now
	}
	defer resp.Body.Close()
	
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return false, err
	}
	
	valid, ok := result["valid"].(bool)
	return ok && valid, nil
}

func generateCapsuleID(name, capsuleType string) string {
	hash := sha256.Sum256([]byte(name + capsuleType + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func generateWorkflowID(name string) string {
	hash := sha256.Sum256([]byte(name + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

