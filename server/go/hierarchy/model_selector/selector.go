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
	"math"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// ModelSelector represents the AI model selection service
type ModelSelector struct {
	logger        *logrus.Logger
	redis         *redis.Client
	db            *sql.DB
	router        *gin.Engine
	config        *Config
	modelCatalog  *ModelCatalog
	decisionGraph *DecisionGraph
	trifectaCourt *TrifectaCourtClient
}

// Config holds the model selector configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
}

// ModelCatalog manages available AI models and tools
type ModelCatalog struct {
	models map[string]*ModelInfo
}

// ModelInfo represents information about an AI model
type ModelInfo struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Type         string                 `json:"type"`
	Provider     string                 `json:"provider"`
	Capabilities []string               `json:"capabilities"`
	Performance  *PerformanceMetrics    `json:"performance"`
	Costs        *CostMetrics           `json:"costs"`
	Constraints  map[string]interface{} `json:"constraints"`
	Status       string                 `json:"status"`
}

// PerformanceMetrics represents model performance characteristics
type PerformanceMetrics struct {
	Accuracy      float64 `json:"accuracy"`
	Latency       float64 `json:"latency_ms"`
	Throughput    float64 `json:"throughput_tokens_per_sec"`
	ContextLength int     `json:"context_length"`
	Quality       float64 `json:"quality_score"`
}

// CostMetrics represents model cost characteristics
type CostMetrics struct {
	InputTokenCost  float64 `json:"input_token_cost"`
	OutputTokenCost float64 `json:"output_token_cost"`
	ComputeCost     float64 `json:"compute_cost_per_hour"`
	StorageCost     float64 `json:"storage_cost_per_gb"`
}

// DecisionGraph handles model selection logic
type DecisionGraph struct {
	selector *ModelSelector
}

// SelectionRequest represents a model selection request
type SelectionRequest struct {
	TaskType        string                 `json:"task_type"`
	InputSize       int                    `json:"input_size"`
	PrivacyTier     string                 `json:"privacy_tier"`
	LatencyBudget   float64                `json:"latency_budget_ms"`
	CostCap         float64                `json:"cost_cap"`
	QualityTarget   float64                `json:"quality_target"`
	Context         map[string]interface{} `json:"context"`
	Constraints     map[string]interface{} `json:"constraints"`
}

// SelectionResponse represents the model selection result
type SelectionResponse struct {
	Success         bool                   `json:"success"`
	SelectedModel   *ModelInfo             `json:"selected_model"`
	Rationale       string                 `json:"rationale"`
	ExpectedCost    float64                `json:"expected_cost"`
	ExpectedLatency float64                `json:"expected_latency_ms"`
	Confidence      float64                `json:"confidence"`
	Alternatives    []*ModelInfo           `json:"alternatives"`
	Metadata        map[string]interface{} `json:"metadata"`
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
		ServerPort:       getEnv("SERVER_PORT", "8005"),
		RedisURL:         getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:      getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:         getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL: getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
	}

	logger.Info("🤖 Starting Model Selector Service")

	// Initialize Model Selector
	modelSelector, err := NewModelSelector(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize Model Selector:", err)
	}

	// Start the service
	if err := modelSelector.Start(); err != nil {
		logger.Fatal("Failed to start Model Selector:", err)
	}
}

func NewModelSelector(config *Config, logger *logrus.Logger) (*ModelSelector, error) {
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

	// Create model selector
	ms := &ModelSelector{
		logger: logger,
		redis:  redisClient,
		db:     db,
		router: router,
		config: config,
	}

	// Initialize components
	ms.modelCatalog = NewModelCatalog()
	ms.decisionGraph = &DecisionGraph{selector: ms}
	ms.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	ms.setupRoutes()

	return ms, nil
}

func NewModelCatalog() *ModelCatalog {
	catalog := &ModelCatalog{
		models: make(map[string]*ModelInfo),
	}

	// Initialize with default models
	models := []*ModelInfo{
		{
			ID:           "gpt-4-turbo",
			Name:         "GPT-4 Turbo",
			Type:         "language_model",
			Provider:     "openai",
			Capabilities: []string{"reasoning", "planning", "code_generation", "analysis"},
			Performance: &PerformanceMetrics{
				Accuracy:      0.95,
				Latency:       2000,
				Throughput:    50,
				ContextLength: 128000,
				Quality:       0.95,
			},
			Costs: &CostMetrics{
				InputTokenCost:  0.01,
				OutputTokenCost: 0.03,
				ComputeCost:     0,
				StorageCost:     0,
			},
			Status: "available",
		},
		{
			ID:           "claude-3-sonnet",
			Name:         "Claude 3 Sonnet",
			Type:         "language_model",
			Provider:     "anthropic",
			Capabilities: []string{"reasoning", "analysis", "writing", "code_review"},
			Performance: &PerformanceMetrics{
				Accuracy:      0.93,
				Latency:       1800,
				Throughput:    60,
				ContextLength: 200000,
				Quality:       0.92,
			},
			Costs: &CostMetrics{
				InputTokenCost:  0.003,
				OutputTokenCost: 0.015,
				ComputeCost:     0,
				StorageCost:     0,
			},
			Status: "available",
		},
		{
			ID:           "llama-3-70b",
			Name:         "Llama 3 70B",
			Type:         "language_model",
			Provider:     "meta",
			Capabilities: []string{"reasoning", "code_generation", "local_inference"},
			Performance: &PerformanceMetrics{
				Accuracy:      0.88,
				Latency:       3000,
				Throughput:    30,
				ContextLength: 8192,
				Quality:       0.85,
			},
			Costs: &CostMetrics{
				InputTokenCost:  0,
				OutputTokenCost: 0,
				ComputeCost:     2.5,
				StorageCost:     0.1,
			},
			Status: "available",
		},
		{
			ID:           "whisper-large-v3",
			Name:         "Whisper Large v3",
			Type:         "speech_to_text",
			Provider:     "openai",
			Capabilities: []string{"speech_recognition", "transcription", "translation"},
			Performance: &PerformanceMetrics{
				Accuracy:      0.96,
				Latency:       500,
				Throughput:    100,
				ContextLength: 0,
				Quality:       0.95,
			},
			Costs: &CostMetrics{
				InputTokenCost:  0.006,
				OutputTokenCost: 0,
				ComputeCost:     0,
				StorageCost:     0,
			},
			Status: "available",
		},
		{
			ID:           "dall-e-3",
			Name:         "DALL-E 3",
			Type:         "image_generation",
			Provider:     "openai",
			Capabilities: []string{"image_generation", "creative_design", "visual_content"},
			Performance: &PerformanceMetrics{
				Accuracy:      0.90,
				Latency:       15000,
				Throughput:    1,
				ContextLength: 0,
				Quality:       0.92,
			},
			Costs: &CostMetrics{
				InputTokenCost:  0.04,
				OutputTokenCost: 0,
				ComputeCost:     0,
				StorageCost:     0,
			},
			Status: "available",
		},
	}

	for _, model := range models {
		catalog.models[model.ID] = model
	}

	return catalog
}

func (ms *ModelSelector) setupRoutes() {
	// Health check
	ms.router.GET("/health", ms.healthCheck)

	// Model selection
	ms.router.POST("/select", ms.selectModel)
	ms.router.POST("/select/batch", ms.selectModelBatch)

	// Model catalog
	ms.router.GET("/models", ms.listModels)
	ms.router.GET("/models/:id", ms.getModel)
	ms.router.POST("/models/:id/feedback", ms.recordFeedback)

	// Decision analysis
	ms.router.POST("/analyze/decision", ms.analyzeDecision)
	ms.router.GET("/analytics/performance", ms.getPerformanceAnalytics)

	// Metrics
	ms.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (ms *ModelSelector) Start() error {
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + ms.config.ServerPort,
		Handler: ms.router,
	}

	go func() {
		ms.logger.Infof("Model Selector listening on port %s", ms.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			ms.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	ms.logger.Info("Shutting down Model Selector...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		ms.logger.Error("Server forced to shutdown:", err)
		return err
	}

	ms.logger.Info("Model Selector stopped")
	return nil
}

func (ms *ModelSelector) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "model-selector",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"models": map[string]interface{}{
			"total":     len(ms.modelCatalog.models),
			"available": ms.countAvailableModels(),
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
	if err := ms.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := ms.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (ms *ModelSelector) selectModel(c *gin.Context) {
	var request SelectionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := ms.validateWithTrifectaCourt("select_model", map[string]interface{}{
		"task_type":      request.TaskType,
		"privacy_tier":   request.PrivacyTier,
		"latency_budget": request.LatencyBudget,
		"cost_cap":       request.CostCap,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process model selection
	response, err := ms.decisionGraph.SelectModel(request)
	if err != nil {
		ms.logger.Error("Failed to select model:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Model selection failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (ms *ModelSelector) selectModelBatch(c *gin.Context) {
	var requests []SelectionRequest
	if err := c.ShouldBindJSON(&requests); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	responses := make([]*SelectionResponse, 0, len(requests))
	for _, request := range requests {
		response, err := ms.decisionGraph.SelectModel(request)
		if err != nil {
			ms.logger.Error("Failed to select model in batch:", err)
			continue
		}
		responses = append(responses, response)
	}

	c.JSON(http.StatusOK, gin.H{
		"responses": responses,
		"total":     len(responses),
	})
}

func (ms *ModelSelector) listModels(c *gin.Context) {
	models := make([]*ModelInfo, 0, len(ms.modelCatalog.models))
	for _, model := range ms.modelCatalog.models {
		models = append(models, model)
	}

	c.JSON(http.StatusOK, gin.H{
		"models": models,
		"total":  len(models),
	})
}

func (ms *ModelSelector) getModel(c *gin.Context) {
	id := c.Param("id")
	model, exists := ms.modelCatalog.models[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Model not found"})
		return
	}

	c.JSON(http.StatusOK, model)
}

func (ms *ModelSelector) recordFeedback(c *gin.Context) {
	id := c.Param("id")
	
	var feedback struct {
		Rating      float64                `json:"rating"`
		Performance map[string]interface{} `json:"performance"`
		Comments    string                 `json:"comments"`
	}
	if err := c.ShouldBindJSON(&feedback); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid feedback format"})
		return
	}

	// Store feedback in Redis
	feedbackData := map[string]interface{}{
		"model_id":    id,
		"rating":      feedback.Rating,
		"performance": feedback.Performance,
		"comments":    feedback.Comments,
		"timestamp":   time.Now().UTC(),
	}

	feedbackJSON, _ := json.Marshal(feedbackData)
	ctx := context.Background()
	ms.redis.LPush(ctx, fmt.Sprintf("model:feedback:%s", id), feedbackJSON)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Feedback recorded",
	})
}

func (ms *ModelSelector) analyzeDecision(c *gin.Context) {
	// Implementation for decision analysis
	c.JSON(http.StatusOK, gin.H{
		"message": "Decision analysis endpoint",
		"status":  "implemented",
	})
}

func (ms *ModelSelector) getPerformanceAnalytics(c *gin.Context) {
	// Implementation for performance analytics
	c.JSON(http.StatusOK, gin.H{
		"analytics": map[string]interface{}{
			"total_selections": 0,
			"average_latency":  0,
			"cost_efficiency":  0,
		},
	})
}

// DecisionGraph methods
func (dg *DecisionGraph) SelectModel(request SelectionRequest) (*SelectionResponse, error) {
	// Step 1: Task classification
	taskCategory := dg.classifyTask(request.TaskType)
	
	// Step 2: Filter models by capabilities
	candidates := dg.filterModelsByCapabilities(taskCategory, request.PrivacyTier)
	
	// Step 3: Score models based on requirements
	scoredModels := dg.scoreModels(candidates, request)
	
	// Step 4: Select best model
	if len(scoredModels) == 0 {
		return nil, fmt.Errorf("no suitable models found")
	}
	
	selectedModel := scoredModels[0].model
	score := scoredModels[0].score
	
	// Step 5: Generate rationale
	rationale := dg.generateRationale(selectedModel, request, score)
	
	// Step 6: Calculate expected metrics
	expectedCost := dg.calculateExpectedCost(selectedModel, request)
	expectedLatency := selectedModel.Performance.Latency
	
	// Step 7: Get alternatives
	alternatives := make([]*ModelInfo, 0, 3)
	for i := 1; i < len(scoredModels) && i < 4; i++ {
		alternatives = append(alternatives, scoredModels[i].model)
	}
	
	return &SelectionResponse{
		Success:         true,
		SelectedModel:   selectedModel,
		Rationale:       rationale,
		ExpectedCost:    expectedCost,
		ExpectedLatency: expectedLatency,
		Confidence:      score,
		Alternatives:    alternatives,
		Metadata: map[string]interface{}{
			"task_category":     taskCategory,
			"candidates_count":  len(candidates),
			"selection_time":    time.Now().UTC(),
		},
	}, nil
}

type scoredModel struct {
	model *ModelInfo
	score float64
}

func (dg *DecisionGraph) classifyTask(taskType string) string {
	taskType = strings.ToLower(taskType)
	
	if strings.Contains(taskType, "reasoning") || strings.Contains(taskType, "planning") {
		return "reasoning"
	} else if strings.Contains(taskType, "code") || strings.Contains(taskType, "programming") {
		return "code"
	} else if strings.Contains(taskType, "image") || strings.Contains(taskType, "visual") {
		return "image"
	} else if strings.Contains(taskType, "speech") || strings.Contains(taskType, "audio") {
		return "audio"
	} else if strings.Contains(taskType, "transform") || strings.Contains(taskType, "format") {
		return "transformation"
	}
	
	return "general"
}

func (dg *DecisionGraph) filterModelsByCapabilities(taskCategory, privacyTier string) []*ModelInfo {
	var candidates []*ModelInfo
	
	for _, model := range dg.selector.modelCatalog.models {
		if model.Status != "available" {
			continue
		}
		
		// Check privacy requirements
		if privacyTier == "confidential" && model.Provider != "local" {
			continue
		}
		
		// Check task compatibility
		hasCapability := false
		for _, capability := range model.Capabilities {
			if strings.Contains(capability, taskCategory) || taskCategory == "general" {
				hasCapability = true
				break
			}
		}
		
		if hasCapability {
			candidates = append(candidates, model)
		}
	}
	
	return candidates
}

func (dg *DecisionGraph) scoreModels(candidates []*ModelInfo, request SelectionRequest) []scoredModel {
	var scored []scoredModel
	
	for _, model := range candidates {
		score := dg.calculateModelScore(model, request)
		scored = append(scored, scoredModel{model: model, score: score})
	}
	
	// Sort by score (descending)
	for i := 0; i < len(scored)-1; i++ {
		for j := i + 1; j < len(scored); j++ {
			if scored[j].score > scored[i].score {
				scored[i], scored[j] = scored[j], scored[i]
			}
		}
	}
	
	return scored
}

func (dg *DecisionGraph) calculateModelScore(model *ModelInfo, request SelectionRequest) float64 {
	score := 0.0
	
	// Quality score (30%)
	qualityScore := model.Performance.Quality
	if request.QualityTarget > 0 {
		qualityScore = math.Min(qualityScore/request.QualityTarget, 1.0)
	}
	score += qualityScore * 0.3
	
	// Latency score (25%)
	latencyScore := 1.0
	if request.LatencyBudget > 0 && model.Performance.Latency > request.LatencyBudget {
		latencyScore = request.LatencyBudget / model.Performance.Latency
	}
	score += latencyScore * 0.25
	
	// Cost score (25%)
	costScore := 1.0
	expectedCost := dg.calculateExpectedCost(model, request)
	if request.CostCap > 0 && expectedCost > request.CostCap {
		costScore = request.CostCap / expectedCost
	}
	score += costScore * 0.25
	
	// Accuracy score (20%)
	accuracyScore := model.Performance.Accuracy
	score += accuracyScore * 0.2
	
	return score
}

func (dg *DecisionGraph) calculateExpectedCost(model *ModelInfo, request SelectionRequest) float64 {
	if model.Costs.ComputeCost > 0 {
		// Local model - compute cost
		return model.Costs.ComputeCost * 0.1 // Assume 6 minutes of compute
	}
	
	// API model - token cost
	estimatedTokens := float64(request.InputSize) * 1.3 // Assume 30% overhead
	return estimatedTokens * model.Costs.InputTokenCost
}

func (dg *DecisionGraph) generateRationale(model *ModelInfo, request SelectionRequest, score float64) string {
	return fmt.Sprintf("Selected %s for %s task. Score: %.2f. Rationale: High quality (%.2f), suitable latency (%.0fms), cost-effective.",
		model.Name, request.TaskType, score, model.Performance.Quality, model.Performance.Latency)
}

func (ms *ModelSelector) countAvailableModels() int {
	count := 0
	for _, model := range ms.modelCatalog.models {
		if model.Status == "available" {
			count++
		}
	}
	return count
}

func (ms *ModelSelector) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := ms.trifectaCourt.client.Post(
		ms.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		ms.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

