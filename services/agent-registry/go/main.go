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

// AgentRegistry represents the agent lifecycle management service
type AgentRegistry struct {
	logger        *logrus.Logger
	redis         *redis.Client
	db            *sql.DB
	router        *gin.Engine
	config        *Config
	agents        map[string]*AgentInfo
	lifecycle     *LifecycleManager
	trifectaCourt *TrifectaCourtClient
}

// Config holds the agent registry configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
	HeartbeatTimeout int    `yaml:"heartbeat_timeout_seconds"`
}

// AgentInfo represents information about a registered agent
type AgentInfo struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Type         string                 `json:"type"`
	Category     string                 `json:"category"`
	Version      string                 `json:"version"`
	Capabilities []string               `json:"capabilities"`
	Endpoints    map[string]string      `json:"endpoints"`
	Status       string                 `json:"status"`
	Health       *HealthInfo            `json:"health"`
	Metadata     map[string]interface{} `json:"metadata"`
	RegisteredAt time.Time              `json:"registered_at"`
	LastSeen     time.Time              `json:"last_seen"`
	Heartbeat    *HeartbeatInfo         `json:"heartbeat"`
}

// HealthInfo represents agent health status
type HealthInfo struct {
	Status      string                 `json:"status"`
	LastCheck   time.Time              `json:"last_check"`
	Uptime      float64                `json:"uptime_seconds"`
	CPU         float64                `json:"cpu_percent"`
	Memory      float64                `json:"memory_percent"`
	Connections int                    `json:"active_connections"`
	Errors      int                    `json:"error_count"`
	Metrics     map[string]interface{} `json:"metrics"`
}

// HeartbeatInfo represents agent heartbeat information
type HeartbeatInfo struct {
	Interval    int       `json:"interval_seconds"`
	LastBeat    time.Time `json:"last_beat"`
	MissedBeats int       `json:"missed_beats"`
	Enabled     bool      `json:"enabled"`
}

// LifecycleManager handles agent lifecycle operations
type LifecycleManager struct {
	registry *AgentRegistry
}

// RegistrationRequest represents an agent registration request
type RegistrationRequest struct {
	Name         string                 `json:"name"`
	Type         string                 `json:"type"`
	Category     string                 `json:"category"`
	Version      string                 `json:"version"`
	Capabilities []string               `json:"capabilities"`
	Endpoints    map[string]string      `json:"endpoints"`
	Metadata     map[string]interface{} `json:"metadata"`
	Heartbeat    *HeartbeatConfig       `json:"heartbeat"`
}

// HeartbeatConfig represents heartbeat configuration
type HeartbeatConfig struct {
	Enabled  bool `json:"enabled"`
	Interval int  `json:"interval_seconds"`
}

// RegistrationResponse represents the registration result
type RegistrationResponse struct {
	Success   bool      `json:"success"`
	AgentID   string    `json:"agent_id"`
	Message   string    `json:"message"`
	Endpoints []string  `json:"endpoints"`
	ExpiresAt time.Time `json:"expires_at"`
}

// DiscoveryRequest represents an agent discovery request
type DiscoveryRequest struct {
	Type         string   `json:"type,omitempty"`
	Category     string   `json:"category,omitempty"`
	Capabilities []string `json:"capabilities,omitempty"`
	Status       string   `json:"status,omitempty"`
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
		ServerPort:       getEnv("SERVER_PORT", "8006"),
		RedisURL:         getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:      getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:         getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL: getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		HeartbeatTimeout: 60,
	}

	logger.Info("📋 Starting Agent Registry Service")

	// Initialize Agent Registry
	agentRegistry, err := NewAgentRegistry(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize Agent Registry:", err)
	}

	// Start the service
	if err := agentRegistry.Start(); err != nil {
		logger.Fatal("Failed to start Agent Registry:", err)
	}
}

func NewAgentRegistry(config *Config, logger *logrus.Logger) (*AgentRegistry, error) {
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

	// Create agent registry
	ar := &AgentRegistry{
		logger: logger,
		redis:  redisClient,
		db:     db,
		router: router,
		config: config,
		agents: make(map[string]*AgentInfo),
	}

	// Initialize components
	ar.lifecycle = &LifecycleManager{registry: ar}
	ar.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	ar.setupRoutes()

	// Start background tasks
	go ar.startHeartbeatMonitor()
	go ar.startHealthChecker()

	return ar, nil
}

func (ar *AgentRegistry) setupRoutes() {
	// Health check
	ar.router.GET("/health", ar.healthCheck)

	// Agent registration
	ar.router.POST("/agents/register", ar.registerAgent)
	ar.router.DELETE("/agents/:id/deregister", ar.deregisterAgent)

	// Agent discovery
	ar.router.GET("/agents", ar.listAgents)
	ar.router.POST("/agents/discover", ar.discoverAgents)
	ar.router.GET("/agents/:id", ar.getAgent)

	// Agent lifecycle
	ar.router.POST("/agents/:id/heartbeat", ar.recordHeartbeat)
	ar.router.PUT("/agents/:id/status", ar.updateAgentStatus)
	ar.router.GET("/agents/:id/health", ar.getAgentHealth)

	// Agent communication
	ar.router.POST("/agents/:id/message", ar.sendMessage)
	ar.router.GET("/agents/:id/messages", ar.getMessages)

	// Registry analytics
	ar.router.GET("/analytics/overview", ar.getAnalyticsOverview)
	ar.router.GET("/analytics/health", ar.getHealthAnalytics)

	// Metrics
	ar.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (ar *AgentRegistry) Start() error {
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + ar.config.ServerPort,
		Handler: ar.router,
	}

	go func() {
		ar.logger.Infof("Agent Registry listening on port %s", ar.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			ar.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	ar.logger.Info("Shutting down Agent Registry...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		ar.logger.Error("Server forced to shutdown:", err)
		return err
	}

	ar.logger.Info("Agent Registry stopped")
	return nil
}

func (ar *AgentRegistry) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "agent-registry",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"agents": map[string]interface{}{
			"total":   len(ar.agents),
			"active":  ar.countAgentsByStatus("active"),
			"healthy": ar.countHealthyAgents(),
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
	if err := ar.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := ar.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (ar *AgentRegistry) registerAgent(c *gin.Context) {
	var request RegistrationRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := ar.validateWithTrifectaCourt("register_agent", map[string]interface{}{
		"name":         request.Name,
		"type":         request.Type,
		"category":     request.Category,
		"capabilities": request.Capabilities,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process registration
	response, err := ar.lifecycle.RegisterAgent(request)
	if err != nil {
		ar.logger.Error("Failed to register agent:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Registration failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (ar *AgentRegistry) deregisterAgent(c *gin.Context) {
	agentID := c.Param("id")
	
	if _, exists := ar.agents[agentID]; !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Agent not found"})
		return
	}

	// Remove agent
	delete(ar.agents, agentID)
	
	// Remove from Redis
	ctx := context.Background()
	ar.redis.Del(ctx, fmt.Sprintf("agent:info:%s", agentID))
	ar.redis.Del(ctx, fmt.Sprintf("agent:heartbeat:%s", agentID))

	ar.logger.Infof("Agent deregistered: %s", agentID)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Agent deregistered successfully",
	})
}

func (ar *AgentRegistry) listAgents(c *gin.Context) {
	agents := make([]*AgentInfo, 0, len(ar.agents))
	for _, agent := range ar.agents {
		agents = append(agents, agent)
	}

	c.JSON(http.StatusOK, gin.H{
		"agents": agents,
		"total":  len(agents),
	})
}

func (ar *AgentRegistry) discoverAgents(c *gin.Context) {
	var request DiscoveryRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Filter agents based on criteria
	var matchingAgents []*AgentInfo
	for _, agent := range ar.agents {
		if ar.matchesDiscoveryCriteria(agent, request) {
			matchingAgents = append(matchingAgents, agent)
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"agents": matchingAgents,
		"total":  len(matchingAgents),
		"query":  request,
	})
}

func (ar *AgentRegistry) getAgent(c *gin.Context) {
	agentID := c.Param("id")
	agent, exists := ar.agents[agentID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Agent not found"})
		return
	}

	c.JSON(http.StatusOK, agent)
}

func (ar *AgentRegistry) recordHeartbeat(c *gin.Context) {
	agentID := c.Param("id")
	agent, exists := ar.agents[agentID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Agent not found"})
		return
	}

	var heartbeatData struct {
		Health  *HealthInfo            `json:"health"`
		Metrics map[string]interface{} `json:"metrics"`
	}
	if err := c.ShouldBindJSON(&heartbeatData); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid heartbeat data"})
		return
	}

	// Update agent heartbeat
	now := time.Now()
	agent.LastSeen = now
	if agent.Heartbeat != nil {
		agent.Heartbeat.LastBeat = now
		agent.Heartbeat.MissedBeats = 0
	}

	// Update health info
	if heartbeatData.Health != nil {
		agent.Health = heartbeatData.Health
		agent.Health.LastCheck = now
	}

	// Store in Redis
	ctx := context.Background()
	heartbeatInfo := map[string]interface{}{
		"agent_id":  agentID,
		"timestamp": now,
		"health":    heartbeatData.Health,
		"metrics":   heartbeatData.Metrics,
	}
	heartbeatJSON, _ := json.Marshal(heartbeatInfo)
	ar.redis.Set(ctx, fmt.Sprintf("agent:heartbeat:%s", agentID), heartbeatJSON, time.Duration(ar.config.HeartbeatTimeout*2)*time.Second)

	c.JSON(http.StatusOK, gin.H{
		"success":   true,
		"timestamp": now,
		"message":   "Heartbeat recorded",
	})
}

func (ar *AgentRegistry) updateAgentStatus(c *gin.Context) {
	agentID := c.Param("id")
	agent, exists := ar.agents[agentID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Agent not found"})
		return
	}

	var request struct {
		Status string `json:"status"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	agent.Status = request.Status
	agent.LastSeen = time.Now()

	c.JSON(http.StatusOK, agent)
}

func (ar *AgentRegistry) getAgentHealth(c *gin.Context) {
	agentID := c.Param("id")
	agent, exists := ar.agents[agentID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Agent not found"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"agent_id": agentID,
		"health":   agent.Health,
		"status":   agent.Status,
		"last_seen": agent.LastSeen,
	})
}

func (ar *AgentRegistry) sendMessage(c *gin.Context) {
	agentID := c.Param("id")
	
	var message struct {
		Type    string                 `json:"type"`
		Content string                 `json:"content"`
		Data    map[string]interface{} `json:"data"`
	}
	if err := c.ShouldBindJSON(&message); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid message format"})
		return
	}

	// Store message in Redis
	messageData := map[string]interface{}{
		"to":        agentID,
		"type":      message.Type,
		"content":   message.Content,
		"data":      message.Data,
		"timestamp": time.Now().UTC(),
		"id":        generateMessageID(),
	}

	messageJSON, _ := json.Marshal(messageData)
	ctx := context.Background()
	ar.redis.LPush(ctx, fmt.Sprintf("agent:messages:%s", agentID), messageJSON)

	c.JSON(http.StatusOK, gin.H{
		"success":    true,
		"message_id": messageData["id"],
		"delivered":  true,
	})
}

func (ar *AgentRegistry) getMessages(c *gin.Context) {
	agentID := c.Param("id")
	
	ctx := context.Background()
	messagesJSON, err := ar.redis.LRange(ctx, fmt.Sprintf("agent:messages:%s", agentID), 0, 99).Result()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to retrieve messages"})
		return
	}

	var messages []map[string]interface{}
	for _, msgJSON := range messagesJSON {
		var message map[string]interface{}
		if err := json.Unmarshal([]byte(msgJSON), &message); err != nil {
			continue
		}
		messages = append(messages, message)
	}

	c.JSON(http.StatusOK, gin.H{
		"messages": messages,
		"total":    len(messages),
	})
}

func (ar *AgentRegistry) getAnalyticsOverview(c *gin.Context) {
	overview := map[string]interface{}{
		"total_agents":   len(ar.agents),
		"active_agents":  ar.countAgentsByStatus("active"),
		"healthy_agents": ar.countHealthyAgents(),
		"agent_types":    ar.getAgentTypeDistribution(),
		"categories":     ar.getAgentCategoryDistribution(),
	}

	c.JSON(http.StatusOK, overview)
}

func (ar *AgentRegistry) getHealthAnalytics(c *gin.Context) {
	healthStats := map[string]interface{}{
		"overall_health": ar.calculateOverallHealth(),
		"unhealthy_agents": ar.getUnhealthyAgents(),
		"missed_heartbeats": ar.getMissedHeartbeats(),
	}

	c.JSON(http.StatusOK, healthStats)
}

// LifecycleManager methods
func (lm *LifecycleManager) RegisterAgent(request RegistrationRequest) (*RegistrationResponse, error) {
	// Generate agent ID
	agentID := generateAgentID(request.Name, request.Type)
	
	// Create agent info
	agent := &AgentInfo{
		ID:           agentID,
		Name:         request.Name,
		Type:         request.Type,
		Category:     request.Category,
		Version:      request.Version,
		Capabilities: request.Capabilities,
		Endpoints:    request.Endpoints,
		Status:       "active",
		Health: &HealthInfo{
			Status:    "unknown",
			LastCheck: time.Now(),
		},
		Metadata:     request.Metadata,
		RegisteredAt: time.Now(),
		LastSeen:     time.Now(),
	}

	// Setup heartbeat if enabled
	if request.Heartbeat != nil && request.Heartbeat.Enabled {
		agent.Heartbeat = &HeartbeatInfo{
			Interval:    request.Heartbeat.Interval,
			LastBeat:    time.Now(),
			MissedBeats: 0,
			Enabled:     true,
		}
	}

	// Store agent
	lm.registry.agents[agentID] = agent

	// Store in Redis
	agentJSON, _ := json.Marshal(agent)
	ctx := context.Background()
	lm.registry.redis.Set(ctx, fmt.Sprintf("agent:info:%s", agentID), agentJSON, 24*time.Hour)

	lm.registry.logger.Infof("Agent registered: %s (%s)", agent.Name, agentID)

	// Prepare endpoints list
	var endpoints []string
	for _, endpoint := range request.Endpoints {
		endpoints = append(endpoints, endpoint)
	}

	return &RegistrationResponse{
		Success:   true,
		AgentID:   agentID,
		Message:   "Agent registered successfully",
		Endpoints: endpoints,
		ExpiresAt: time.Now().Add(24 * time.Hour),
	}, nil
}

// Background tasks
func (ar *AgentRegistry) startHeartbeatMonitor() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		ar.checkHeartbeats()
	}
}

func (ar *AgentRegistry) startHealthChecker() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		ar.checkAgentHealth()
	}
}

func (ar *AgentRegistry) checkHeartbeats() {
	now := time.Now()
	timeout := time.Duration(ar.config.HeartbeatTimeout) * time.Second

	for agentID, agent := range ar.agents {
		if agent.Heartbeat != nil && agent.Heartbeat.Enabled {
			if now.Sub(agent.Heartbeat.LastBeat) > timeout {
				agent.Heartbeat.MissedBeats++
				if agent.Heartbeat.MissedBeats > 3 {
					agent.Status = "unhealthy"
					ar.logger.Warnf("Agent %s marked unhealthy due to missed heartbeats", agentID)
				}
			}
		}
	}
}

func (ar *AgentRegistry) checkAgentHealth() {
	// Implementation for health checking
	for agentID, agent := range ar.agents {
		if agent.Health != nil && time.Since(agent.Health.LastCheck) > 5*time.Minute {
			agent.Health.Status = "stale"
			ar.logger.Debugf("Agent %s health status marked as stale", agentID)
		}
	}
}

// Helper methods
func (ar *AgentRegistry) matchesDiscoveryCriteria(agent *AgentInfo, request DiscoveryRequest) bool {
	if request.Type != "" && agent.Type != request.Type {
		return false
	}
	
	if request.Category != "" && agent.Category != request.Category {
		return false
	}
	
	if request.Status != "" && agent.Status != request.Status {
		return false
	}
	
	if len(request.Capabilities) > 0 {
		for _, reqCap := range request.Capabilities {
			found := false
			for _, agentCap := range agent.Capabilities {
				if agentCap == reqCap {
					found = true
					break
				}
			}
			if !found {
				return false
			}
		}
	}
	
	return true
}

func (ar *AgentRegistry) countAgentsByStatus(status string) int {
	count := 0
	for _, agent := range ar.agents {
		if agent.Status == status {
			count++
		}
	}
	return count
}

func (ar *AgentRegistry) countHealthyAgents() int {
	count := 0
	for _, agent := range ar.agents {
		if agent.Health != nil && agent.Health.Status == "healthy" {
			count++
		}
	}
	return count
}

func (ar *AgentRegistry) getAgentTypeDistribution() map[string]int {
	distribution := make(map[string]int)
	for _, agent := range ar.agents {
		distribution[agent.Type]++
	}
	return distribution
}

func (ar *AgentRegistry) getAgentCategoryDistribution() map[string]int {
	distribution := make(map[string]int)
	for _, agent := range ar.agents {
		distribution[agent.Category]++
	}
	return distribution
}

func (ar *AgentRegistry) calculateOverallHealth() float64 {
	if len(ar.agents) == 0 {
		return 1.0
	}
	
	healthyCount := ar.countHealthyAgents()
	return float64(healthyCount) / float64(len(ar.agents))
}

func (ar *AgentRegistry) getUnhealthyAgents() []string {
	var unhealthy []string
	for agentID, agent := range ar.agents {
		if agent.Status != "active" || (agent.Health != nil && agent.Health.Status != "healthy") {
			unhealthy = append(unhealthy, agentID)
		}
	}
	return unhealthy
}

func (ar *AgentRegistry) getMissedHeartbeats() map[string]int {
	missed := make(map[string]int)
	for agentID, agent := range ar.agents {
		if agent.Heartbeat != nil && agent.Heartbeat.MissedBeats > 0 {
			missed[agentID] = agent.Heartbeat.MissedBeats
		}
	}
	return missed
}

func (ar *AgentRegistry) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := ar.trifectaCourt.client.Post(
		ar.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		ar.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

func generateAgentID(name, agentType string) string {
	hash := sha256.Sum256([]byte(name + agentType + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func generateMessageID() string {
	hash := sha256.Sum256([]byte(time.Now().String()))
	return hex.EncodeToString(hash[:])[:12]
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

