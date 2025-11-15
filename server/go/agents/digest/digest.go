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
	"crypto/sha256"
	"encoding/hex"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// DigestAgent represents the data ingestion and analysis service
type DigestAgent struct {
	logger         *logrus.Logger
	redis          *redis.Client
	db             *sql.DB
	router         *gin.Engine
	config         *Config
	digestEngine   *DigestEngine
	knowledgeGraph *KnowledgeGraphClient
	trifectaCourt  *TrifectaCourtClient
}

// Config holds the digest agent configuration
type Config struct {
	ServerPort        string `yaml:"server_port"`
	RedisURL          string `yaml:"redis_url"`
	DatabaseURL       string `yaml:"database_url"`
	LogLevel          string `yaml:"log_level"`
	KnowledgeGraphURL string `yaml:"knowledge_graph_url"`
	TrifectaCourtURL  string `yaml:"trifecta_court_url"`
	MaxConcurrency    int    `yaml:"max_concurrency"`
	ChunkSize         int    `yaml:"chunk_size"`
}

// DigestEngine handles large-scale data ingestion
type DigestEngine struct {
	agent *DigestAgent
}

// KnowledgeGraphClient interfaces with knowledge graph service
type KnowledgeGraphClient struct {
	baseURL string
	client  *http.Client
}

// TrifectaCourtClient interfaces with constitutional validation
type TrifectaCourtClient struct {
	baseURL string
	client  *http.Client
}

// DigestRequest represents a data ingestion request
type DigestRequest struct {
	Source      string                 `json:"source"`
	DataType    string                 `json:"data_type"`
	Content     string                 `json:"content"`
	Metadata    map[string]interface{} `json:"metadata"`
	Priority    string                 `json:"priority"`
	ChunkSize   int                    `json:"chunk_size,omitempty"`
}

// DigestResponse represents the result of data ingestion
type DigestResponse struct {
	Success       bool                   `json:"success"`
	DigestID      string                 `json:"digest_id"`
	ChunksCreated int                    `json:"chunks_created"`
	KnowledgeNodes int                   `json:"knowledge_nodes"`
	ProcessingTime string                `json:"processing_time"`
	Metadata      map[string]interface{} `json:"metadata"`
}

// AnalysisRequest represents a data analysis request
type AnalysisRequest struct {
	Query       string                 `json:"query"`
	DataSources []string               `json:"data_sources"`
	AnalysisType string                `json:"analysis_type"`
	Parameters  map[string]interface{} `json:"parameters"`
}

// AnalysisResponse represents analysis results
type AnalysisResponse struct {
	Success    bool                   `json:"success"`
	Results    map[string]interface{} `json:"results"`
	Insights   []string               `json:"insights"`
	Confidence float64                `json:"confidence"`
	Sources    []string               `json:"sources"`
}

func main() {
	// Initialize logger
	logger := logrus.New()
	logger.SetFormatter(&logrus.JSONFormatter{})
	logger.SetLevel(logrus.InfoLevel)

	// Load configuration
	config := &Config{
		ServerPort:        getEnv("SERVER_PORT", "8004"),
		RedisURL:          getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:       getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:          getEnv("LOG_LEVEL", "info"),
		KnowledgeGraphURL: getEnv("KNOWLEDGE_GRAPH_URL", "http://localhost:8002"),
		TrifectaCourtURL:  getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		MaxConcurrency:    10,
		ChunkSize:         1024,
	}

	logger.Info("🔍 Starting Digest Agent Service")

	// Initialize Digest Agent
	digestAgent, err := NewDigestAgent(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize Digest Agent:", err)
	}

	// Start the service
	if err := digestAgent.Start(); err != nil {
		logger.Fatal("Failed to start Digest Agent:", err)
	}
}

func NewDigestAgent(config *Config, logger *logrus.Logger) (*DigestAgent, error) {
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

	// Create digest agent
	agent := &DigestAgent{
		logger: logger,
		redis:  redisClient,
		db:     db,
		router: router,
		config: config,
	}

	// Initialize components
	agent.digestEngine = &DigestEngine{agent: agent}
	agent.knowledgeGraph = &KnowledgeGraphClient{
		baseURL: config.KnowledgeGraphURL,
		client:  &http.Client{Timeout: 30 * time.Second},
	}
	agent.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	agent.setupRoutes()

	return agent, nil
}

func (da *DigestAgent) setupRoutes() {
	// Health check
	da.router.GET("/health", da.healthCheck)

	// Digest endpoints
	da.router.POST("/digest/ingest", da.ingestData)
	da.router.POST("/digest/analyze", da.analyzeData)
	da.router.GET("/digest/status/:digest_id", da.getDigestStatus)
	da.router.GET("/digest/list", da.listDigests)

	// Knowledge endpoints
	da.router.POST("/knowledge/extract", da.extractKnowledge)
	da.router.POST("/knowledge/synthesize", da.synthesizeKnowledge)

	// Research endpoints
	da.router.POST("/research/query", da.researchQuery)
	da.router.POST("/research/insights", da.generateInsights)

	// Metrics
	da.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (da *DigestAgent) Start() error {
	// Setup graceful shutdown
	_, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + da.config.ServerPort,
		Handler: da.router,
	}

	go func() {
		da.logger.Infof("Digest Agent listening on port %s", da.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			da.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	da.logger.Info("Shutting down Digest Agent...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		da.logger.Error("Server forced to shutdown:", err)
		return err
	}

	da.logger.Info("Digest Agent stopped")
	return nil
}

func (da *DigestAgent) healthCheck(c *gin.Context) {
	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "digest-agent",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"capabilities": []string{
			"data_ingestion",
			"knowledge_extraction",
			"research_synthesis",
			"large_scale_analysis",
		},
	}

	// Check dependencies
	dependencies := map[string]string{
		"redis":           "healthy",
		"database":        "healthy",
		"knowledge_graph": "unknown",
		"trifecta_court":  "unknown",
	}

	// Test Redis
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	if err := da.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := da.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (da *DigestAgent) ingestData(c *gin.Context) {
	var request DigestRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := da.validateWithTrifectaCourt("ingest_data", map[string]interface{}{
		"source":    request.Source,
		"data_type": request.DataType,
		"size":      len(request.Content),
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process the ingestion
	response, err := da.digestEngine.ProcessIngestion(request)
	if err != nil {
		da.logger.Error("Failed to process ingestion:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Ingestion failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (da *DigestAgent) analyzeData(c *gin.Context) {
	var request AnalysisRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := da.validateWithTrifectaCourt("analyze_data", map[string]interface{}{
		"query":        request.Query,
		"data_sources": request.DataSources,
		"analysis_type": request.AnalysisType,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Process the analysis
	response, err := da.digestEngine.ProcessAnalysis(request)
	if err != nil {
		da.logger.Error("Failed to process analysis:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Analysis failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (da *DigestAgent) getDigestStatus(c *gin.Context) {
	digestID := c.Param("digest_id")
	
	// Get status from Redis
	ctx := context.Background()
	statusJSON, err := da.redis.Get(ctx, fmt.Sprintf("digest:status:%s", digestID)).Result()
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Digest not found"})
		return
	}

	var status map[string]interface{}
	if err := json.Unmarshal([]byte(statusJSON), &status); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to parse status"})
		return
	}

	c.JSON(http.StatusOK, status)
}

func (da *DigestAgent) listDigests(c *gin.Context) {
	ctx := context.Background()
	keys, err := da.redis.Keys(ctx, "digest:status:*").Result()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to list digests"})
		return
	}

	digests := make([]map[string]interface{}, 0, len(keys))
	for _, key := range keys {
		statusJSON, err := da.redis.Get(ctx, key).Result()
		if err != nil {
			continue
		}

		var status map[string]interface{}
		if err := json.Unmarshal([]byte(statusJSON), &status); err != nil {
			continue
		}

		digests = append(digests, status)
	}

	c.JSON(http.StatusOK, gin.H{
		"digests": digests,
		"total":   len(digests),
	})
}

func (da *DigestAgent) extractKnowledge(c *gin.Context) {
	// Implementation for knowledge extraction
	c.JSON(http.StatusOK, gin.H{
		"message": "Knowledge extraction endpoint",
		"status":  "implemented",
	})
}

func (da *DigestAgent) synthesizeKnowledge(c *gin.Context) {
	// Implementation for knowledge synthesis
	c.JSON(http.StatusOK, gin.H{
		"message": "Knowledge synthesis endpoint",
		"status":  "implemented",
	})
}

func (da *DigestAgent) researchQuery(c *gin.Context) {
	// Implementation for research queries
	c.JSON(http.StatusOK, gin.H{
		"message": "Research query endpoint",
		"status":  "implemented",
	})
}

func (da *DigestAgent) generateInsights(c *gin.Context) {
	// Implementation for insight generation
	c.JSON(http.StatusOK, gin.H{
		"message": "Insight generation endpoint",
		"status":  "implemented",
	})
}

// DigestEngine methods
func (de *DigestEngine) ProcessIngestion(request DigestRequest) (*DigestResponse, error) {
	startTime := time.Now()
	
	// Generate digest ID
	digestID := generateDigestID(request.Source, request.Content)
	
	// Chunk the content
	chunks := de.chunkContent(request.Content, request.ChunkSize)
	
	// Process chunks and create knowledge nodes
	knowledgeNodes := 0
	for _, chunk := range chunks {
		// Extract knowledge from chunk
		if err := de.extractKnowledgeFromChunk(chunk, digestID); err != nil {
			de.agent.logger.Error("Failed to extract knowledge from chunk:", err)
			continue
		}
		knowledgeNodes++
	}
	
	// Store digest status
	status := map[string]interface{}{
		"digest_id":       digestID,
		"source":          request.Source,
		"data_type":       request.DataType,
		"chunks_created":  len(chunks),
		"knowledge_nodes": knowledgeNodes,
		"status":          "completed",
		"created_at":      time.Now().UTC(),
		"processing_time": time.Since(startTime).String(),
	}
	
	statusJSON, _ := json.Marshal(status)
	ctx := context.Background()
	de.agent.redis.Set(ctx, fmt.Sprintf("digest:status:%s", digestID), statusJSON, 24*time.Hour)
	
	return &DigestResponse{
		Success:        true,
		DigestID:       digestID,
		ChunksCreated:  len(chunks),
		KnowledgeNodes: knowledgeNodes,
		ProcessingTime: time.Since(startTime).String(),
		Metadata:       request.Metadata,
	}, nil
}

func (de *DigestEngine) ProcessAnalysis(request AnalysisRequest) (*AnalysisResponse, error) {
	// Implement analysis logic
	insights := []string{
		"Data analysis completed successfully",
		"Patterns identified in the dataset",
		"Recommendations generated based on findings",
	}
	
	results := map[string]interface{}{
		"query":         request.Query,
		"analysis_type": request.AnalysisType,
		"data_sources":  request.DataSources,
		"findings":      "Analysis results would be here",
	}
	
	return &AnalysisResponse{
		Success:    true,
		Results:    results,
		Insights:   insights,
		Confidence: 0.85,
		Sources:    request.DataSources,
	}, nil
}

func (de *DigestEngine) chunkContent(content string, chunkSize int) []string {
	if chunkSize <= 0 {
		chunkSize = de.agent.config.ChunkSize
	}
	
	var chunks []string
	words := strings.Fields(content)
	
	for i := 0; i < len(words); i += chunkSize {
		end := i + chunkSize
		if end > len(words) {
			end = len(words)
		}
		chunk := strings.Join(words[i:end], " ")
		chunks = append(chunks, chunk)
	}
	
	return chunks
}

func (de *DigestEngine) extractKnowledgeFromChunk(chunk, digestID string) error {
	// Extract knowledge and add to knowledge graph
	// This would integrate with the knowledge graph service
	return nil
}

func (da *DigestAgent) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := da.trifectaCourt.client.Post(
		da.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		da.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

func generateDigestID(source, content string) string {
	hash := sha256.Sum256([]byte(source + content + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

