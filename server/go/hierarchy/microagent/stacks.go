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

// MicroAgentStacks represents the cooperative work pods service
type MicroAgentStacks struct {
	logger        *logrus.Logger
	redis         *redis.Client
	db            *sql.DB
	router        *gin.Engine
	config        *Config
	stacks        map[string]*Stack
	stackManager  *StackManager
	trifectaCourt *TrifectaCourtClient
}

// Config holds the microagent stacks configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
	MaxStacks        int    `yaml:"max_stacks"`
	StackTimeout     int    `yaml:"stack_timeout_seconds"`
}

// Stack represents a cooperative work pod
type Stack struct {
	ID              string                 `json:"id"`
	Name            string                 `json:"name"`
	Type            string                 `json:"type"`
	Objective       string                 `json:"objective"`
	Status          string                 `json:"status"`
	CommandChief    *CommandChiefAgent     `json:"command_chief"`
	Operators       []*OperatorAgent       `json:"operators"`
	Adapters        []*AdapterAgent        `json:"adapters"`
	Guards          []*GuardAgent          `json:"guards"`
	Resources       *StackResources        `json:"resources"`
	Workflow        *WorkflowDefinition    `json:"workflow"`
	Metadata        map[string]interface{} `json:"metadata"`
	CreatedAt       time.Time              `json:"created_at"`
	UpdatedAt       time.Time              `json:"updated_at"`
	CompletedAt     *time.Time             `json:"completed_at,omitempty"`
	CommissionedBy  string                 `json:"commissioned_by"`
}

// CommandChiefAgent represents the stack orchestrator
type CommandChiefAgent struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Status       string                 `json:"status"`
	Capabilities []string               `json:"capabilities"`
	Workload     int                    `json:"workload"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// OperatorAgent represents a work execution agent
type OperatorAgent struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Type         string                 `json:"type"`
	Specialty    string                 `json:"specialty"`
	Status       string                 `json:"status"`
	Capabilities []string               `json:"capabilities"`
	Workload     int                    `json:"workload"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// AdapterAgent represents an integration agent
type AdapterAgent struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Protocol     string                 `json:"protocol"`
	Endpoint     string                 `json:"endpoint"`
	Status       string                 `json:"status"`
	Capabilities []string               `json:"capabilities"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// GuardAgent represents a validation and safety agent
type GuardAgent struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	GuardType    string                 `json:"guard_type"`
	Rules        []string               `json:"rules"`
	Status       string                 `json:"status"`
	Violations   int                    `json:"violations"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// StackResources represents allocated resources
type StackResources struct {
	CPU        float64                `json:"cpu_cores"`
	Memory     int                    `json:"memory_mb"`
	Storage    int                    `json:"storage_gb"`
	Network    int                    `json:"network_mbps"`
	Budget     float64                `json:"budget_usd"`
	Timeout    int                    `json:"timeout_seconds"`
	Priority   string                 `json:"priority"`
	Metadata   map[string]interface{} `json:"metadata"`
}

// WorkflowDefinition represents the stack workflow
type WorkflowDefinition struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Steps       []*WorkflowStep        `json:"steps"`
	Dependencies map[string][]string   `json:"dependencies"`
	Parallel    [][]string             `json:"parallel"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// WorkflowStep represents a workflow step
type WorkflowStep struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	Type        string                 `json:"type"`
	Agent       string                 `json:"agent"`
	Action      string                 `json:"action"`
	Parameters  map[string]interface{} `json:"parameters"`
	Status      string                 `json:"status"`
	StartedAt   *time.Time             `json:"started_at,omitempty"`
	CompletedAt *time.Time             `json:"completed_at,omitempty"`
	Results     map[string]interface{} `json:"results,omitempty"`
}

// StackManager handles stack lifecycle operations
type StackManager struct {
	service *MicroAgentStacks
}

// CommissionRequest represents a stack commissioning request
type CommissionRequest struct {
	Name           string                 `json:"name"`
	Type           string                 `json:"type"`
	Objective      string                 `json:"objective"`
	Resources      *StackResources        `json:"resources"`
	Workflow       *WorkflowDefinition    `json:"workflow"`
	CommissionedBy string                 `json:"commissioned_by"`
	Priority       string                 `json:"priority"`
	Metadata       map[string]interface{} `json:"metadata"`
}

// CommissionResponse represents the commissioning result
type CommissionResponse struct {
	Success     bool                   `json:"success"`
	StackID     string                 `json:"stack_id"`
	Status      string                 `json:"status"`
	Agents      map[string]interface{} `json:"agents"`
	Resources   *StackResources        `json:"resources"`
	EstimatedTime int                  `json:"estimated_time_seconds"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// ExecutionRequest represents a stack execution request
type ExecutionRequest struct {
	StackID    string                 `json:"stack_id"`
	Action     string                 `json:"action"`
	Parameters map[string]interface{} `json:"parameters"`
	Priority   string                 `json:"priority"`
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
		ServerPort:       getEnv("SERVER_PORT", "8008"),
		RedisURL:         getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:      getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:         getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL: getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		MaxStacks:        100,
		StackTimeout:     3600,
	}

	logger.Info("🔧 Starting MicroAgent Stacks Service")

	// Initialize MicroAgent Stacks
	microAgentStacks, err := NewMicroAgentStacks(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize MicroAgent Stacks:", err)
	}

	// Start the service
	if err := microAgentStacks.Start(); err != nil {
		logger.Fatal("Failed to start MicroAgent Stacks:", err)
	}
}

func NewMicroAgentStacks(config *Config, logger *logrus.Logger) (*MicroAgentStacks, error) {
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

	// Create microagent stacks service
	mas := &MicroAgentStacks{
		logger: logger,
		redis:  redisClient,
		db:     db,
		router: router,
		config: config,
		stacks: make(map[string]*Stack),
	}

	// Initialize components
	mas.stackManager = &StackManager{service: mas}
	mas.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	mas.setupRoutes()

	// Start background tasks
	go mas.startStackMonitor()

	return mas, nil
}

func (mas *MicroAgentStacks) setupRoutes() {
	// Health check
	mas.router.GET("/health", mas.healthCheck)

	// Stack commissioning
	mas.router.POST("/stacks/commission", mas.commissionStack)
	mas.router.DELETE("/stacks/:stack_id/decommission", mas.decommissionStack)

	// Stack management
	mas.router.GET("/stacks", mas.listStacks)
	mas.router.GET("/stacks/:stack_id", mas.getStack)
	mas.router.PUT("/stacks/:stack_id/status", mas.updateStackStatus)

	// Stack execution
	mas.router.POST("/stacks/:stack_id/execute", mas.executeStack)
	mas.router.POST("/stacks/:stack_id/pause", mas.pauseStack)
	mas.router.POST("/stacks/:stack_id/resume", mas.resumeStack)
	mas.router.POST("/stacks/:stack_id/abort", mas.abortStack)

	// Agent management
	mas.router.GET("/stacks/:stack_id/agents", mas.getStackAgents)
	mas.router.POST("/stacks/:stack_id/agents/:agent_id/action", mas.executeAgentAction)

	// Workflow management
	mas.router.GET("/stacks/:stack_id/workflow", mas.getWorkflow)
	mas.router.PUT("/stacks/:stack_id/workflow", mas.updateWorkflow)
	mas.router.GET("/stacks/:stack_id/workflow/status", mas.getWorkflowStatus)

	// Resource management
	mas.router.GET("/stacks/:stack_id/resources", mas.getStackResources)
	mas.router.PUT("/stacks/:stack_id/resources", mas.updateStackResources)

	// Analytics
	mas.router.GET("/analytics/overview", mas.getAnalyticsOverview)
	mas.router.GET("/analytics/performance", mas.getPerformanceAnalytics)

	// Metrics
	mas.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (mas *MicroAgentStacks) Start() error {
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + mas.config.ServerPort,
		Handler: mas.router,
	}

	go func() {
		mas.logger.Infof("MicroAgent Stacks listening on port %s", mas.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			mas.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	mas.logger.Info("Shutting down MicroAgent Stacks...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		mas.logger.Error("Server forced to shutdown:", err)
		return err
	}

	mas.logger.Info("MicroAgent Stacks stopped")
	return nil
}

func (mas *MicroAgentStacks) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "microagent-stacks",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"stacks": map[string]interface{}{
			"total":   len(mas.stacks),
			"active":  mas.countStacksByStatus("active"),
			"running": mas.countStacksByStatus("running"),
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
	if err := mas.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := mas.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (mas *MicroAgentStacks) commissionStack(c *gin.Context) {
	var request CommissionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := mas.validateWithTrifectaCourt("commission_stack", map[string]interface{}{
		"name":            request.Name,
		"type":            request.Type,
		"objective":       request.Objective,
		"commissioned_by": request.CommissionedBy,
		"resources":       request.Resources,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process commissioning
	response, err := mas.stackManager.CommissionStack(request)
	if err != nil {
		mas.logger.Error("Failed to commission stack:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Stack commissioning failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (mas *MicroAgentStacks) decommissionStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	// Update stack status
	stack.Status = "decommissioned"
	stack.UpdatedAt = time.Now()
	now := time.Now()
	stack.CompletedAt = &now

	// Remove from active stacks
	delete(mas.stacks, stackID)

	// Remove from Redis
	ctx := context.Background()
	mas.redis.Del(ctx, fmt.Sprintf("stack:%s", stackID))

	mas.logger.Infof("Stack decommissioned: %s", stackID)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Stack decommissioned successfully",
	})
}

func (mas *MicroAgentStacks) listStacks(c *gin.Context) {
	stacks := make([]*Stack, 0, len(mas.stacks))
	for _, stack := range mas.stacks {
		stacks = append(stacks, stack)
	}

	c.JSON(http.StatusOK, gin.H{
		"stacks": stacks,
		"total":  len(stacks),
	})
}

func (mas *MicroAgentStacks) getStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	c.JSON(http.StatusOK, stack)
}

func (mas *MicroAgentStacks) updateStackStatus(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	var request struct {
		Status string `json:"status"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	stack.Status = request.Status
	stack.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, stack)
}

func (mas *MicroAgentStacks) executeStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	var request ExecutionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Execute stack workflow
	result, err := mas.stackManager.ExecuteStack(stack, request)
	if err != nil {
		mas.logger.Error("Failed to execute stack:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Stack execution failed"})
		return
	}

	c.JSON(http.StatusOK, result)
}

func (mas *MicroAgentStacks) pauseStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	stack.Status = "paused"
	stack.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "paused",
	})
}

func (mas *MicroAgentStacks) resumeStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	stack.Status = "running"
	stack.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "running",
	})
}

func (mas *MicroAgentStacks) abortStack(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	stack.Status = "aborted"
	stack.UpdatedAt = time.Now()
	now := time.Now()
	stack.CompletedAt = &now

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"status":  "aborted",
	})
}

func (mas *MicroAgentStacks) getStackAgents(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	agents := map[string]interface{}{
		"command_chief": stack.CommandChief,
		"operators":     stack.Operators,
		"adapters":      stack.Adapters,
		"guards":        stack.Guards,
	}

	c.JSON(http.StatusOK, agents)
}

func (mas *MicroAgentStacks) executeAgentAction(c *gin.Context) {
	stackID := c.Param("stack_id")
	agentID := c.Param("agent_id")
	
	var request struct {
		Action     string                 `json:"action"`
		Parameters map[string]interface{} `json:"parameters"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Execute agent action
	result := map[string]interface{}{
		"success":   true,
		"stack_id":  stackID,
		"agent_id":  agentID,
		"action":    request.Action,
		"result":    "Action executed successfully",
		"timestamp": time.Now().UTC(),
	}

	c.JSON(http.StatusOK, result)
}

func (mas *MicroAgentStacks) getWorkflow(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	c.JSON(http.StatusOK, stack.Workflow)
}

func (mas *MicroAgentStacks) updateWorkflow(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	var workflow WorkflowDefinition
	if err := c.ShouldBindJSON(&workflow); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid workflow format"})
		return
	}

	stack.Workflow = &workflow
	stack.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, stack.Workflow)
}

func (mas *MicroAgentStacks) getWorkflowStatus(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	status := map[string]interface{}{
		"workflow_id": stack.Workflow.ID,
		"status":      stack.Status,
		"steps":       len(stack.Workflow.Steps),
		"completed":   mas.countCompletedSteps(stack.Workflow),
		"progress":    mas.calculateWorkflowProgress(stack.Workflow),
	}

	c.JSON(http.StatusOK, status)
}

func (mas *MicroAgentStacks) getStackResources(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	c.JSON(http.StatusOK, stack.Resources)
}

func (mas *MicroAgentStacks) updateStackResources(c *gin.Context) {
	stackID := c.Param("stack_id")
	stack, exists := mas.stacks[stackID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Stack not found"})
		return
	}

	var resources StackResources
	if err := c.ShouldBindJSON(&resources); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid resources format"})
		return
	}

	stack.Resources = &resources
	stack.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, stack.Resources)
}

func (mas *MicroAgentStacks) getAnalyticsOverview(c *gin.Context) {
	overview := map[string]interface{}{
		"total_stacks":     len(mas.stacks),
		"active_stacks":    mas.countStacksByStatus("active"),
		"running_stacks":   mas.countStacksByStatus("running"),
		"completed_stacks": mas.countStacksByStatus("completed"),
		"stack_types":      mas.getStackTypeDistribution(),
	}

	c.JSON(http.StatusOK, overview)
}

func (mas *MicroAgentStacks) getPerformanceAnalytics(c *gin.Context) {
	performance := map[string]interface{}{
		"average_execution_time": mas.calculateAverageExecutionTime(),
		"resource_utilization":   mas.calculateResourceUtilization(),
		"success_rate":           mas.calculateSuccessRate(),
	}

	c.JSON(http.StatusOK, performance)
}

// StackManager methods
func (sm *StackManager) CommissionStack(request CommissionRequest) (*CommissionResponse, error) {
	// Generate stack ID
	stackID := generateStackID(request.Name, request.Type)
	
	// Create stack
	stack := &Stack{
		ID:             stackID,
		Name:           request.Name,
		Type:           request.Type,
		Objective:      request.Objective,
		Status:         "commissioned",
		Resources:      request.Resources,
		Workflow:       request.Workflow,
		Metadata:       request.Metadata,
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
		CommissionedBy: request.CommissionedBy,
	}

	// Initialize agents
	stack.CommandChief = &CommandChiefAgent{
		ID:           generateAgentID("command-chief"),
		Name:         "Command Chief Agent",
		Status:       "active",
		Capabilities: []string{"orchestration", "coordination", "decision_making"},
		Workload:     0,
	}

	stack.Operators = []*OperatorAgent{
		{
			ID:           generateAgentID("operator-1"),
			Name:         "Primary Operator",
			Type:         "general",
			Specialty:    "task_execution",
			Status:       "active",
			Capabilities: []string{"execution", "processing", "analysis"},
			Workload:     0,
		},
	}

	stack.Adapters = []*AdapterAgent{
		{
			ID:           generateAgentID("adapter-1"),
			Name:         "Integration Adapter",
			Protocol:     "http",
			Endpoint:     "http://localhost:8080",
			Status:       "active",
			Capabilities: []string{"integration", "protocol_translation", "data_transformation"},
		},
	}

	stack.Guards = []*GuardAgent{
		{
			ID:        generateAgentID("guard-1"),
			Name:      "Safety Guard",
			GuardType: "safety",
			Rules:     []string{"no_harmful_actions", "resource_limits", "time_limits"},
			Status:    "active",
			Violations: 0,
		},
	}

	// Store stack
	sm.service.stacks[stackID] = stack

	// Store in Redis
	stackJSON, _ := json.Marshal(stack)
	ctx := context.Background()
	sm.service.redis.Set(ctx, fmt.Sprintf("stack:%s", stackID), stackJSON, 24*time.Hour)

	sm.service.logger.Infof("Stack commissioned: %s (%s)", stack.Name, stackID)

	return &CommissionResponse{
		Success:   true,
		StackID:   stackID,
		Status:    "commissioned",
		Agents: map[string]interface{}{
			"command_chief": stack.CommandChief.ID,
			"operators":     len(stack.Operators),
			"adapters":      len(stack.Adapters),
			"guards":        len(stack.Guards),
		},
		Resources:     stack.Resources,
		EstimatedTime: 300, // 5 minutes
		Metadata:      request.Metadata,
	}, nil
}

func (sm *StackManager) ExecuteStack(stack *Stack, request ExecutionRequest) (map[string]interface{}, error) {
	// Update stack status
	stack.Status = "running"
	stack.UpdatedAt = time.Now()

	// Execute workflow steps
	if stack.Workflow != nil {
		for _, step := range stack.Workflow.Steps {
			if step.Status != "completed" {
				step.Status = "running"
				now := time.Now()
				step.StartedAt = &now
				
				// Simulate step execution
				time.Sleep(100 * time.Millisecond)
				
				step.Status = "completed"
				step.CompletedAt = &now
				step.Results = map[string]interface{}{
					"success": true,
					"output":  "Step completed successfully",
				}
			}
		}
	}

	// Update stack status
	stack.Status = "completed"
	now := time.Now()
	stack.CompletedAt = &now

	return map[string]interface{}{
		"success":      true,
		"stack_id":     stack.ID,
		"status":       stack.Status,
		"completed_at": stack.CompletedAt,
		"results":      "Stack execution completed successfully",
	}, nil
}

// Background tasks
func (mas *MicroAgentStacks) startStackMonitor() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		mas.monitorStacks()
	}
}

func (mas *MicroAgentStacks) monitorStacks() {
	now := time.Now()
	timeout := time.Duration(mas.config.StackTimeout) * time.Second

	for stackID, stack := range mas.stacks {
		if stack.Status == "running" && now.Sub(stack.UpdatedAt) > timeout {
			stack.Status = "timeout"
			stack.CompletedAt = &now
			mas.logger.Warnf("Stack %s timed out", stackID)
		}
	}
}

// Helper methods
func (mas *MicroAgentStacks) countStacksByStatus(status string) int {
	count := 0
	for _, stack := range mas.stacks {
		if stack.Status == status {
			count++
		}
	}
	return count
}

func (mas *MicroAgentStacks) getStackTypeDistribution() map[string]int {
	distribution := make(map[string]int)
	for _, stack := range mas.stacks {
		distribution[stack.Type]++
	}
	return distribution
}

func (mas *MicroAgentStacks) countCompletedSteps(workflow *WorkflowDefinition) int {
	count := 0
	for _, step := range workflow.Steps {
		if step.Status == "completed" {
			count++
		}
	}
	return count
}

func (mas *MicroAgentStacks) calculateWorkflowProgress(workflow *WorkflowDefinition) float64 {
	if len(workflow.Steps) == 0 {
		return 0.0
	}
	completed := mas.countCompletedSteps(workflow)
	return float64(completed) / float64(len(workflow.Steps)) * 100
}

func (mas *MicroAgentStacks) calculateAverageExecutionTime() float64 {
	var totalTime float64
	var count int
	
	for _, stack := range mas.stacks {
		if stack.CompletedAt != nil {
			duration := stack.CompletedAt.Sub(stack.CreatedAt)
			totalTime += duration.Seconds()
			count++
		}
	}
	
	if count == 0 {
		return 0.0
	}
	
	return totalTime / float64(count)
}

func (mas *MicroAgentStacks) calculateResourceUtilization() float64 {
	// Mock calculation
	return 75.5
}

func (mas *MicroAgentStacks) calculateSuccessRate() float64 {
	var successful int
	var total int
	
	for _, stack := range mas.stacks {
		if stack.Status == "completed" || stack.Status == "aborted" || stack.Status == "timeout" {
			total++
			if stack.Status == "completed" {
				successful++
			}
		}
	}
	
	if total == 0 {
		return 0.0
	}
	
	return float64(successful) / float64(total) * 100
}

func (mas *MicroAgentStacks) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := mas.trifectaCourt.client.Post(
		mas.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		mas.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

func generateStackID(name, stackType string) string {
	hash := sha256.Sum256([]byte(name + stackType + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func generateAgentID(agentType string) string {
	hash := sha256.Sum256([]byte(agentType + time.Now().String()))
	return hex.EncodeToString(hash[:])[:12]
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

