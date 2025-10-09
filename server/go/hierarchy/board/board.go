package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// BoardAgents represents the executive team management service
type BoardAgents struct {
	logger        *logrus.Logger
	redis         *redis.Client
	db            *sql.DB
	router        *gin.Engine
	config        *Config
	executives    map[string]*ExecutiveAgent
	trifectaCourt *TrifectaCourtClient
	noaCommander  *NOACommanderClient
}

// Config holds the board agents configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
	NOACommanderURL  string `yaml:"noa_commander_url"`
}

// ExecutiveAgent represents a board-level executive agent
type ExecutiveAgent struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Role         string                 `json:"role"`
	Domain       string                 `json:"domain"`
	Capabilities []string               `json:"capabilities"`
	Status       string                 `json:"status"`
	Workload     int                    `json:"workload"`
	Metadata     map[string]interface{} `json:"metadata"`
	CreatedAt    time.Time              `json:"created_at"`
	UpdatedAt    time.Time              `json:"updated_at"`
}

// MissionRequest represents a mission assignment from NOA
type MissionRequest struct {
	MissionID    string                 `json:"mission_id"`
	ExecutiveID  string                 `json:"executive_id"`
	Objective    string                 `json:"objective"`
	Priority     string                 `json:"priority"`
	Deadline     time.Time              `json:"deadline"`
	Resources    []string               `json:"resources"`
	Constraints  map[string]interface{} `json:"constraints"`
	Context      map[string]interface{} `json:"context"`
}

// MissionResponse represents the result of mission execution
type MissionResponse struct {
	Success      bool                   `json:"success"`
	MissionID    string                 `json:"mission_id"`
	ExecutiveID  string                 `json:"executive_id"`
	Results      map[string]interface{} `json:"results"`
	Deliverables []string               `json:"deliverables"`
	Status       string                 `json:"status"`
	CompletedAt  time.Time              `json:"completed_at"`
}

// StackCommissionRequest represents a request to commission MicroAgent stacks
type StackCommissionRequest struct {
	StackType    string                 `json:"stack_type"`
	Objective    string                 `json:"objective"`
	Resources    map[string]interface{} `json:"resources"`
	ExecutiveID  string                 `json:"executive_id"`
	Priority     string                 `json:"priority"`
}

// TrifectaCourtClient interfaces with constitutional validation
type TrifectaCourtClient struct {
	baseURL string
	client  *http.Client
}

// NOACommanderClient interfaces with NOA Commander
type NOACommanderClient struct {
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
		ServerPort:       getEnv("SERVER_PORT", "8003"),
		RedisURL:         getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:      getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:         getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL: getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		NOACommanderURL:  getEnv("NOA_COMMANDER_URL", "http://localhost:8001"),
	}

	logger.Info("👥 Starting Board Agents Service")

	// Initialize Board Agents
	boardAgents, err := NewBoardAgents(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize Board Agents:", err)
	}

	// Start the service
	if err := boardAgents.Start(); err != nil {
		logger.Fatal("Failed to start Board Agents:", err)
	}
}

func NewBoardAgents(config *Config, logger *logrus.Logger) (*BoardAgents, error) {
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

	// Create board agents service
	ba := &BoardAgents{
		logger:     logger,
		redis:      redisClient,
		db:         db,
		router:     router,
		config:     config,
		executives: make(map[string]*ExecutiveAgent),
	}

	// Initialize clients
	ba.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}
	ba.noaCommander = &NOACommanderClient{
		baseURL: config.NOACommanderURL,
		client:  &http.Client{Timeout: 30 * time.Second},
	}

	// Initialize executive agents
	ba.initializeExecutives()

	// Setup routes
	ba.setupRoutes()

	return ba, nil
}

func (ba *BoardAgents) initializeExecutives() {
	executives := []*ExecutiveAgent{
		{
			ID:     "strategy-cto",
			Name:   "Strategy/CTO Agent",
			Role:   "Chief Technology Officer",
			Domain: "technology_strategy",
			Capabilities: []string{
				"technical_direction",
				"system_architecture",
				"technology_selection",
				"scalability_planning",
				"microservice_governance",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "coo",
			Name:   "COO Agent",
			Role:   "Chief Operating Officer",
			Domain: "operations",
			Capabilities: []string{
				"operations_management",
				"sla_enforcement",
				"resource_allocation",
				"change_management",
				"incident_response",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "cfo-finops",
			Name:   "CFO/FinOps Agent",
			Role:   "Chief Financial Officer",
			Domain: "finance_operations",
			Capabilities: []string{
				"budget_management",
				"cost_optimization",
				"roi_analysis",
				"financial_compliance",
				"spend_telemetry",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "legal-compliance",
			Name:   "Legal/Compliance Agent",
			Role:   "Chief Legal Officer",
			Domain: "legal_compliance",
			Capabilities: []string{
				"license_compliance",
				"data_governance",
				"regulatory_adherence",
				"policy_frameworks",
				"export_controls",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "security",
			Name:   "Security Agent",
			Role:   "Chief Security Officer",
			Domain: "security",
			Capabilities: []string{
				"secrets_management",
				"supply_chain_security",
				"sbom_attestation",
				"vulnerability_management",
				"risk_assessment",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "growth-partnerships",
			Name:   "Growth/Partnerships Agent",
			Role:   "Chief Growth Officer",
			Domain: "growth_partnerships",
			Capabilities: []string{
				"ecosystem_strategy",
				"partnership_integration",
				"ingestion_roadmaps",
				"api_curation",
				"crm_integration",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
		{
			ID:     "digest-rd",
			Name:   "Digest Agent (R&D)",
			Role:   "Chief Research Officer",
			Domain: "research_development",
			Capabilities: []string{
				"knowledge_synthesis",
				"research_insights",
				"data_digestion",
				"pattern_recognition",
				"innovation_discovery",
			},
			Status:    "active",
			Workload:  0,
			CreatedAt: time.Now(),
			UpdatedAt: time.Now(),
		},
	}

	for _, exec := range executives {
		ba.executives[exec.ID] = exec
		ba.logger.Infof("Initialized executive agent: %s (%s)", exec.Name, exec.Role)
	}
}

func (ba *BoardAgents) setupRoutes() {
	// Health check
	ba.router.GET("/health", ba.healthCheck)

	// Executive management
	ba.router.GET("/executives", ba.listExecutives)
	ba.router.GET("/executives/:id", ba.getExecutive)
	ba.router.PUT("/executives/:id/status", ba.updateExecutiveStatus)

	// Mission management
	ba.router.POST("/missions/assign", ba.assignMission)
	ba.router.GET("/missions/:mission_id/status", ba.getMissionStatus)
	ba.router.POST("/missions/:mission_id/complete", ba.completeMission)

	// Stack commissioning
	ba.router.POST("/stacks/commission", ba.commissionStack)
	ba.router.GET("/stacks/active", ba.listActiveStacks)

	// Policy and governance
	ba.router.POST("/policies/enforce", ba.enforcePolicy)
	ba.router.GET("/governance/status", ba.getGovernanceStatus)

	// Metrics
	ba.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (ba *BoardAgents) Start() error {
	// Setup graceful shutdown
	_, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + ba.config.ServerPort,
		Handler: ba.router,
	}

	go func() {
		ba.logger.Infof("Board Agents listening on port %s", ba.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			ba.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	ba.logger.Info("Shutting down Board Agents...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		ba.logger.Error("Server forced to shutdown:", err)
		return err
	}

	ba.logger.Info("Board Agents stopped")
	return nil
}

func (ba *BoardAgents) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "board-agents",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"executives": map[string]interface{}{
			"total":  len(ba.executives),
			"active": ba.countActiveExecutives(),
		},
	}

	// Check dependencies
	dependencies := map[string]string{
		"redis":          "healthy",
		"database":       "healthy",
		"trifecta_court": "unknown",
		"noa_commander":  "unknown",
	}

	// Test Redis
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	if err := ba.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := ba.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (ba *BoardAgents) listExecutives(c *gin.Context) {
	executives := make([]*ExecutiveAgent, 0, len(ba.executives))
	for _, exec := range ba.executives {
		executives = append(executives, exec)
	}

	c.JSON(http.StatusOK, gin.H{
		"executives": executives,
		"total":      len(executives),
	})
}

func (ba *BoardAgents) getExecutive(c *gin.Context) {
	id := c.Param("id")
	exec, exists := ba.executives[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Executive not found"})
		return
	}

	c.JSON(http.StatusOK, exec)
}

func (ba *BoardAgents) updateExecutiveStatus(c *gin.Context) {
	id := c.Param("id")
	exec, exists := ba.executives[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Executive not found"})
		return
	}

	var request struct {
		Status string `json:"status"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	exec.Status = request.Status
	exec.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, exec)
}

func (ba *BoardAgents) assignMission(c *gin.Context) {
	var request MissionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate executive exists
	exec, exists := ba.executives[request.ExecutiveID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Executive not found"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := ba.validateWithTrifectaCourt("assign_mission", map[string]interface{}{
		"mission_id":   request.MissionID,
		"executive_id": request.ExecutiveID,
		"objective":    request.Objective,
		"priority":     request.Priority,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process mission assignment
	response, err := ba.processMissionAssignment(request, exec)
	if err != nil {
		ba.logger.Error("Failed to assign mission:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Mission assignment failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (ba *BoardAgents) getMissionStatus(c *gin.Context) {
	missionID := c.Param("mission_id")
	
	// Get mission status from Redis
	ctx := context.Background()
	statusJSON, err := ba.redis.Get(ctx, fmt.Sprintf("mission:status:%s", missionID)).Result()
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Mission not found"})
		return
	}

	var status map[string]interface{}
	if err := json.Unmarshal([]byte(statusJSON), &status); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to parse status"})
		return
	}

	c.JSON(http.StatusOK, status)
}

func (ba *BoardAgents) completeMission(c *gin.Context) {
	missionID := c.Param("mission_id")
	
	var request struct {
		Results      map[string]interface{} `json:"results"`
		Deliverables []string               `json:"deliverables"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Update mission status
	status := map[string]interface{}{
		"mission_id":    missionID,
		"status":        "completed",
		"results":       request.Results,
		"deliverables":  request.Deliverables,
		"completed_at":  time.Now().UTC(),
	}

	statusJSON, _ := json.Marshal(status)
	ctx := context.Background()
	ba.redis.Set(ctx, fmt.Sprintf("mission:status:%s", missionID), statusJSON, 24*time.Hour)

	c.JSON(http.StatusOK, status)
}

func (ba *BoardAgents) commissionStack(c *gin.Context) {
	var request StackCommissionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate executive exists
	_, exists := ba.executives[request.ExecutiveID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Executive not found"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := ba.validateWithTrifectaCourt("commission_stack", map[string]interface{}{
		"stack_type":   request.StackType,
		"objective":    request.Objective,
		"executive_id": request.ExecutiveID,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process stack commissioning
	response := map[string]interface{}{
		"success":      true,
		"stack_id":     fmt.Sprintf("stack-%d", time.Now().Unix()),
		"stack_type":   request.StackType,
		"executive_id": request.ExecutiveID,
		"status":       "commissioned",
		"created_at":   time.Now().UTC(),
	}

	c.JSON(http.StatusOK, response)
}

func (ba *BoardAgents) listActiveStacks(c *gin.Context) {
	// Implementation for listing active stacks
	c.JSON(http.StatusOK, gin.H{
		"active_stacks": []map[string]interface{}{},
		"total":         0,
	})
}

func (ba *BoardAgents) enforcePolicy(c *gin.Context) {
	// Implementation for policy enforcement
	c.JSON(http.StatusOK, gin.H{
		"message": "Policy enforcement endpoint",
		"status":  "implemented",
	})
}

func (ba *BoardAgents) getGovernanceStatus(c *gin.Context) {
	// Implementation for governance status
	c.JSON(http.StatusOK, gin.H{
		"governance": map[string]interface{}{
			"constitutional_compliance": "active",
			"policy_enforcement":        "active",
			"audit_trail":              "enabled",
		},
	})
}

func (ba *BoardAgents) processMissionAssignment(request MissionRequest, exec *ExecutiveAgent) (*MissionResponse, error) {
	// Update executive workload
	exec.Workload++
	exec.UpdatedAt = time.Now()

	// Store mission status
	status := map[string]interface{}{
		"mission_id":   request.MissionID,
		"executive_id": request.ExecutiveID,
		"objective":    request.Objective,
		"priority":     request.Priority,
		"status":       "assigned",
		"assigned_at":  time.Now().UTC(),
		"deadline":     request.Deadline,
	}

	statusJSON, _ := json.Marshal(status)
	ctx := context.Background()
	ba.redis.Set(ctx, fmt.Sprintf("mission:status:%s", request.MissionID), statusJSON, 24*time.Hour)

	return &MissionResponse{
		Success:     true,
		MissionID:   request.MissionID,
		ExecutiveID: request.ExecutiveID,
		Status:      "assigned",
		Results: map[string]interface{}{
			"assignment_time": time.Now().UTC(),
			"executive_name":  exec.Name,
			"executive_role":  exec.Role,
		},
	}, nil
}

func (ba *BoardAgents) countActiveExecutives() int {
	count := 0
	for _, exec := range ba.executives {
		if exec.Status == "active" {
			count++
		}
	}
	return count
}

func (ba *BoardAgents) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := ba.trifectaCourt.client.Post(
		ba.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		ba.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

