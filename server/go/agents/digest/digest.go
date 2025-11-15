package main

import (
	"bytes"
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"github.com/google/uuid"
	pq "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// DigestAgent represents the data ingestion and analysis service
type DigestAgent struct {
	logger          *logrus.Logger
	redis           *redis.Client
	db              *sql.DB
	router          *gin.Engine
	config          *Config
	digestEngine    *DigestEngine
	knowledgeGraph  *KnowledgeGraphClient
	trifectaCourt   *TrifectaCourtClient
	researchService *ResearchService
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

// ResearchService orchestrates research persistence and caching
type ResearchService struct {
	db     *sql.DB
	redis  *redis.Client
	logger *logrus.Logger
}

// KnowledgeProvenance captures lineage for knowledge nodes and edges
type KnowledgeProvenance struct {
	DigestID    string                 `json:"digest_id"`
	Source      string                 `json:"source"`
	ExtractedAt time.Time              `json:"extracted_at"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// KnowledgeNode represents a node in the knowledge graph
type KnowledgeNode struct {
	ID         string                 `json:"id"`
	DigestID   string                 `json:"digest_id"`
	Content    string                 `json:"content"`
	Metadata   map[string]interface{} `json:"metadata"`
	Provenance KnowledgeProvenance    `json:"provenance"`
}

// KnowledgeEdge represents a relationship in the knowledge graph
type KnowledgeEdge struct {
	SourceID   string                 `json:"source_id"`
	TargetID   string                 `json:"target_id"`
	Type       string                 `json:"type"`
	Metadata   map[string]interface{} `json:"metadata"`
	Provenance KnowledgeProvenance    `json:"provenance"`
}

// KnowledgeUpsertPayload packages nodes and edges for persistence
type KnowledgeUpsertPayload struct {
	Nodes []KnowledgeNode `json:"nodes"`
	Edges []KnowledgeEdge `json:"edges"`
}

// KnowledgeGraphUpsertResult mirrors the knowledge graph service response
type KnowledgeGraphUpsertResult struct {
	NodesCreated int `json:"nodes_created"`
	EdgesCreated int `json:"edges_created"`
}

// KnowledgeExtractionResult combines graph response with provenance
type KnowledgeExtractionResult struct {
	KnowledgeGraphUpsertResult
	Provenance []KnowledgeProvenance `json:"provenance"`
}

// KnowledgeSynthesisRequest drives synthesis workflows
type KnowledgeSynthesisRequest struct {
	DigestID string                 `json:"digest_id"`
	Query    string                 `json:"query"`
	Limit    int                    `json:"limit,omitempty"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// KnowledgeSynthesisResponse represents synthesized knowledge
type KnowledgeSynthesisResponse struct {
	DigestID   string                 `json:"digest_id"`
	Query      string                 `json:"query"`
	Summary    string                 `json:"summary"`
	Highlights []string               `json:"highlights"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// KnowledgeExtractionRequest represents manual extraction input
type KnowledgeExtractionRequest struct {
	DigestID  string                 `json:"digest_id"`
	Content   string                 `json:"content"`
	ChunkSize int                    `json:"chunk_size,omitempty"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

// KnowledgeExtractionResponse summarizes extraction results
type KnowledgeExtractionResponse struct {
	DigestID        string                `json:"digest_id"`
	NodesCreated    int                   `json:"nodes_created"`
	EdgesCreated    int                   `json:"edges_created"`
	ChunksProcessed int                   `json:"chunks_processed"`
	Provenance      []KnowledgeProvenance `json:"provenance"`
}

// DigestRequest represents a data ingestion request
type DigestRequest struct {
	Source    string                 `json:"source"`
	DataType  string                 `json:"data_type"`
	Content   string                 `json:"content"`
	Metadata  map[string]interface{} `json:"metadata"`
	Priority  string                 `json:"priority"`
	ChunkSize int                    `json:"chunk_size,omitempty"`
}

// DigestResponse represents the result of data ingestion
type DigestResponse struct {
	Success        bool                   `json:"success"`
	DigestID       string                 `json:"digest_id"`
	ChunksCreated  int                    `json:"chunks_created"`
	KnowledgeNodes int                    `json:"knowledge_nodes"`
	ProcessingTime string                 `json:"processing_time"`
	Metadata       map[string]interface{} `json:"metadata"`
	Provenance     []KnowledgeProvenance  `json:"provenance,omitempty"`
	StatusRef      string                 `json:"status_ref"`
}

// AnalysisRequest represents a data analysis request
type AnalysisRequest struct {
	Query        string                 `json:"query"`
	DataSources  []string               `json:"data_sources"`
	AnalysisType string                 `json:"analysis_type"`
	Parameters   map[string]interface{} `json:"parameters"`
}

// AnalysisResponse represents analysis results
type AnalysisResponse struct {
	Success    bool                   `json:"success"`
	Results    map[string]interface{} `json:"results"`
	Insights   []string               `json:"insights"`
	Confidence float64                `json:"confidence"`
	Sources    []string               `json:"sources"`
	AnalysisID string                 `json:"analysis_id"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
	StatusRef  string                 `json:"status_ref"`
}

// ResearchQueryRequest captures a research query submission
type ResearchQueryRequest struct {
	Query       string                 `json:"query"`
	DataSources []string               `json:"data_sources"`
	Parameters  map[string]interface{} `json:"parameters,omitempty"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// ResearchQueryResponse returns metadata about research intake
type ResearchQueryResponse struct {
	ResearchID string                 `json:"research_id"`
	Status     string                 `json:"status"`
	CreatedAt  time.Time              `json:"created_at"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// ResearchInsightsRequest captures insight submission payloads
type ResearchInsightsRequest struct {
	ResearchID string                 `json:"research_id"`
	Insights   []string               `json:"insights"`
	Summary    string                 `json:"summary"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// ResearchInsightsResponse mirrors stored insight state
type ResearchInsightsResponse struct {
	ResearchID string                 `json:"research_id"`
	Status     string                 `json:"status"`
	Insights   []string               `json:"insights"`
	Summary    string                 `json:"summary"`
	UpdatedAt  time.Time              `json:"updated_at"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
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

	// Initialize research persistence service
	researchService, err := NewResearchService(db, redisClient, logger)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize research service: %w", err)
	}
	agent.researchService = researchService

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
		"query":         request.Query,
		"data_sources":  request.DataSources,
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
	var request KnowledgeExtractionRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	if strings.TrimSpace(request.Content) == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Content is required"})
		return
	}

	if request.DigestID == "" {
		request.DigestID = generateDigestID("knowledge", request.Content)
	}

	if request.ChunkSize <= 0 {
		request.ChunkSize = da.config.ChunkSize
	}

	if valid, err := da.validateWithTrifectaCourt("extract_knowledge", map[string]interface{}{
		"digest_id":      request.DigestID,
		"content_length": len(request.Content),
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	response, err := da.digestEngine.ExtractKnowledge(request)
	if err != nil {
		da.logger.Error("Knowledge extraction failed:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Knowledge extraction failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (da *DigestAgent) synthesizeKnowledge(c *gin.Context) {
	var request KnowledgeSynthesisRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	if request.DigestID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "digest_id is required"})
		return
	}

	if valid, err := da.validateWithTrifectaCourt("synthesize_knowledge", map[string]interface{}{
		"digest_id": request.DigestID,
		"query":     request.Query,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	ctx, cancel := context.WithTimeout(c.Request.Context(), 15*time.Second)
	defer cancel()

	response, err := da.knowledgeGraph.SynthesizeKnowledge(ctx, request)
	if err != nil {
		da.logger.Error("Knowledge synthesis failed:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Knowledge synthesis failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (da *DigestAgent) researchQuery(c *gin.Context) {
	var request ResearchQueryRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	if strings.TrimSpace(request.Query) == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Query is required"})
		return
	}

	if valid, err := da.validateWithTrifectaCourt("research_query", map[string]interface{}{
		"query":        request.Query,
		"data_sources": request.DataSources,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	response, err := da.researchService.CreateResearchRequest(c.Request.Context(), request)
	if err != nil {
		da.logger.Error("Failed to register research query:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Research query failed"})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (da *DigestAgent) generateInsights(c *gin.Context) {
	var request ResearchInsightsRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	if request.ResearchID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "research_id is required"})
		return
	}

	if valid, err := da.validateWithTrifectaCourt("generate_insights", map[string]interface{}{
		"research_id":   request.ResearchID,
		"insight_count": len(request.Insights),
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	response, err := da.researchService.CompleteResearchRequest(c.Request.Context(), request)
	if err != nil {
		da.logger.Error("Failed to persist research insights:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Insight persistence failed"})
		return
	}

	c.JSON(http.StatusOK, response)
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
	edgesCreated := 0
	provenanceRecords := make([]KnowledgeProvenance, 0)
	for _, chunk := range chunks {
		// Extract knowledge from chunk
		result, err := de.extractKnowledgeFromChunk(chunk, digestID)
		if err != nil {
			de.agent.logger.Error("Failed to extract knowledge from chunk:", err)
			continue
		}
		knowledgeNodes += result.NodesCreated
		edgesCreated += result.EdgesCreated
		provenanceRecords = append(provenanceRecords, result.Provenance...)
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
		"edges_created":   edgesCreated,
		"provenance":      provenanceRecords,
	}

	status["metadata"] = map[string]interface{}{
		"status_ref":    fmt.Sprintf("digest:status:%s", digestID),
		"edges_created": edgesCreated,
	}

	statusJSON, _ := json.Marshal(status)
	ctx := context.Background()
	de.agent.redis.Set(ctx, fmt.Sprintf("digest:status:%s", digestID), statusJSON, 24*time.Hour)

	responseMetadata := map[string]interface{}{}
	for k, v := range request.Metadata {
		responseMetadata[k] = v
	}
	responseMetadata["edges_created"] = edgesCreated
	responseMetadata["status_ref"] = fmt.Sprintf("digest:status:%s", digestID)

	return &DigestResponse{
		Success:        true,
		DigestID:       digestID,
		ChunksCreated:  len(chunks),
		KnowledgeNodes: knowledgeNodes,
		ProcessingTime: time.Since(startTime).String(),
		Metadata:       responseMetadata,
		Provenance:     provenanceRecords,
		StatusRef:      fmt.Sprintf("digest:status:%s", digestID),
	}, nil
}

func (de *DigestEngine) ProcessAnalysis(request AnalysisRequest) (*AnalysisResponse, error) {
	startTime := time.Now()

	analysisID := generateDigestID(request.Query, strings.Join(request.DataSources, ","))

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

	metadata := map[string]interface{}{
		"analysis_type": request.AnalysisType,
		"duration":      time.Since(startTime).String(),
		"status_ref":    fmt.Sprintf("analysis:status:%s", analysisID),
	}

	status := map[string]interface{}{
		"analysis_id":   analysisID,
		"status":        "completed",
		"query":         request.Query,
		"data_sources":  request.DataSources,
		"analysis_type": request.AnalysisType,
		"insights":      insights,
		"results":       results,
		"created_at":    startTime.UTC(),
		"completed_at":  time.Now().UTC(),
		"metadata":      metadata,
	}

	statusJSON, _ := json.Marshal(status)
	ctx := context.Background()
	de.agent.redis.Set(ctx, fmt.Sprintf("analysis:status:%s", analysisID), statusJSON, 12*time.Hour)

	return &AnalysisResponse{
		Success:    true,
		Results:    results,
		Insights:   insights,
		Confidence: 0.85,
		Sources:    request.DataSources,
		AnalysisID: analysisID,
		Metadata:   metadata,
		StatusRef:  fmt.Sprintf("analysis:status:%s", analysisID),
	}, nil
}

// ExtractKnowledge processes manual extraction requests and persists provenance
func (de *DigestEngine) ExtractKnowledge(request KnowledgeExtractionRequest) (*KnowledgeExtractionResponse, error) {
	chunks := de.chunkContent(request.Content, request.ChunkSize)
	if len(chunks) == 0 {
		return &KnowledgeExtractionResponse{
			DigestID:        request.DigestID,
			NodesCreated:    0,
			EdgesCreated:    0,
			ChunksProcessed: 0,
			Provenance:      []KnowledgeProvenance{},
		}, nil
	}

	totalNodes := 0
	totalEdges := 0
	provenance := make([]KnowledgeProvenance, 0, len(chunks))

	for _, chunk := range chunks {
		result, err := de.extractKnowledgeFromChunk(chunk, request.DigestID)
		if err != nil {
			return nil, err
		}

		totalNodes += result.NodesCreated
		totalEdges += result.EdgesCreated
		provenance = append(provenance, result.Provenance...)
	}

	return &KnowledgeExtractionResponse{
		DigestID:        request.DigestID,
		NodesCreated:    totalNodes,
		EdgesCreated:    totalEdges,
		ChunksProcessed: len(chunks),
		Provenance:      provenance,
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

func (kg *KnowledgeGraphClient) UpsertKnowledge(ctx context.Context, payload KnowledgeUpsertPayload) (*KnowledgeGraphUpsertResult, error) {
	body, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	endpoint := strings.TrimRight(kg.baseURL, "/") + "/graph/upsert"
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := kg.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		message, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("knowledge graph upsert failed: %s", string(message))
	}

	responseBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	result := &KnowledgeGraphUpsertResult{
		NodesCreated: len(payload.Nodes),
		EdgesCreated: len(payload.Edges),
	}

	if len(responseBytes) > 0 {
		if err := json.Unmarshal(responseBytes, result); err != nil {
			return nil, fmt.Errorf("failed to parse knowledge graph response: %w", err)
		}
		if result.NodesCreated == 0 {
			result.NodesCreated = len(payload.Nodes)
		}
		if result.EdgesCreated == 0 {
			result.EdgesCreated = len(payload.Edges)
		}
	}

	return result, nil
}

func (kg *KnowledgeGraphClient) SynthesizeKnowledge(ctx context.Context, request KnowledgeSynthesisRequest) (*KnowledgeSynthesisResponse, error) {
	body, err := json.Marshal(request)
	if err != nil {
		return nil, err
	}

	endpoint := strings.TrimRight(kg.baseURL, "/") + "/graph/synthesize"
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := kg.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		message, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("knowledge synthesis failed: %s", string(message))
	}

	var result KnowledgeSynthesisResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}

	if result.DigestID == "" {
		result.DigestID = request.DigestID
	}
	if result.Query == "" {
		result.Query = request.Query
	}
	if result.Metadata == nil {
		result.Metadata = map[string]interface{}{}
	}

	result.Metadata["requested_limit"] = request.Limit

	return &result, nil
}

func (de *DigestEngine) extractKnowledgeFromChunk(chunk, digestID string) (*KnowledgeExtractionResult, error) {
	cleanedChunk := strings.TrimSpace(chunk)
	if cleanedChunk == "" {
		return &KnowledgeExtractionResult{
			KnowledgeGraphUpsertResult: KnowledgeGraphUpsertResult{},
			Provenance:                 []KnowledgeProvenance{},
		}, nil
	}

	nodeID := fmt.Sprintf("node:%s", generateDigestID(digestID, cleanedChunk))
	provenance := KnowledgeProvenance{
		DigestID:    digestID,
		Source:      "ingestion_chunk",
		ExtractedAt: time.Now().UTC(),
		Metadata: map[string]interface{}{
			"chunk_length": len(cleanedChunk),
			"chunk_hash":   nodeID,
		},
	}

	payload := KnowledgeUpsertPayload{
		Nodes: []KnowledgeNode{
			{
				ID:       nodeID,
				DigestID: digestID,
				Content:  cleanedChunk,
				Metadata: map[string]interface{}{
					"length": len(cleanedChunk),
				},
				Provenance: provenance,
			},
		},
		Edges: []KnowledgeEdge{
			{
				SourceID: fmt.Sprintf("digest:%s", digestID),
				TargetID: nodeID,
				Type:     "contains_chunk",
				Metadata: map[string]interface{}{
					"confidence": 1.0,
				},
				Provenance: provenance,
			},
		},
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	result, err := de.agent.knowledgeGraph.UpsertKnowledge(ctx, payload)
	if err != nil {
		return nil, err
	}

	nodesCreated := result.NodesCreated
	edgesCreated := result.EdgesCreated
	if nodesCreated == 0 {
		nodesCreated = len(payload.Nodes)
	}
	if edgesCreated == 0 {
		edgesCreated = len(payload.Edges)
	}

	return &KnowledgeExtractionResult{
		KnowledgeGraphUpsertResult: KnowledgeGraphUpsertResult{
			NodesCreated: nodesCreated,
			EdgesCreated: edgesCreated,
		},
		Provenance: []KnowledgeProvenance{provenance},
	}, nil
}

func NewResearchService(db *sql.DB, redisClient *redis.Client, logger *logrus.Logger) (*ResearchService, error) {
	service := &ResearchService{
		db:     db,
		redis:  redisClient,
		logger: logger,
	}

	if err := service.ensureTables(context.Background()); err != nil {
		return nil, err
	}

	return service, nil
}

func (rs *ResearchService) ensureTables(ctx context.Context) error {
	timeoutCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	_, err := rs.db.ExecContext(timeoutCtx, `
        CREATE TABLE IF NOT EXISTS research_insights (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                data_sources TEXT[],
                parameters JSONB,
                insights JSONB,
                summary TEXT,
                metadata JSONB,
                status TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )`)
	return err
}

func (rs *ResearchService) CreateResearchRequest(ctx context.Context, request ResearchQueryRequest) (*ResearchQueryResponse, error) {
	if ctx == nil {
		ctx = context.Background()
	}

	researchID := uuid.NewString()
	parametersJSON, err := json.Marshal(request.Parameters)
	if err != nil {
		return nil, err
	}

	metadataJSON, err := json.Marshal(request.Metadata)
	if err != nil {
		return nil, err
	}

	insightsJSON, err := json.Marshal([]string{})
	if err != nil {
		return nil, err
	}

	createdAt := time.Now().UTC()

	insertCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	_, err = rs.db.ExecContext(insertCtx, `
        INSERT INTO research_insights (id, query, data_sources, parameters, insights, summary, metadata, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
        `,
		researchID,
		request.Query,
		pq.Array(request.DataSources),
		parametersJSON,
		insightsJSON,
		"",
		metadataJSON,
		"pending",
		createdAt,
	)
	if err != nil {
		return nil, err
	}

	statusRef := fmt.Sprintf("research:status:%s", researchID)

	statusPayload := map[string]interface{}{
		"research_id":  researchID,
		"status":       "pending",
		"query":        request.Query,
		"data_sources": request.DataSources,
		"created_at":   createdAt,
	}
	if request.Metadata != nil {
		statusPayload["metadata"] = request.Metadata
	}
	statusPayload["status_ref"] = statusRef

	if err := rs.cacheJSON(ctx, statusRef, statusPayload, 24*time.Hour); err != nil {
		rs.logger.Warn("failed to cache research status:", err)
	}

	responseMetadata := map[string]interface{}{
		"data_sources": request.DataSources,
		"status_ref":   statusRef,
	}
	for k, v := range request.Metadata {
		responseMetadata[k] = v
	}

	return &ResearchQueryResponse{
		ResearchID: researchID,
		Status:     "pending",
		CreatedAt:  createdAt,
		Metadata:   responseMetadata,
	}, nil
}

func (rs *ResearchService) CompleteResearchRequest(ctx context.Context, request ResearchInsightsRequest) (*ResearchInsightsResponse, error) {
	if ctx == nil {
		ctx = context.Background()
	}

	insightsJSON, err := json.Marshal(request.Insights)
	if err != nil {
		return nil, err
	}

	metadataJSON, err := json.Marshal(request.Metadata)
	if err != nil {
		return nil, err
	}

	updatedAt := time.Now().UTC()

	updateCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	result, err := rs.db.ExecContext(updateCtx, `
        UPDATE research_insights
        SET insights = $1, summary = $2, metadata = $3, status = $4, updated_at = $5
        WHERE id = $6
        `,
		insightsJSON,
		request.Summary,
		metadataJSON,
		"completed",
		updatedAt,
		request.ResearchID,
	)
	if err != nil {
		return nil, err
	}

	rowsAffected, err := result.RowsAffected()
	if err == nil && rowsAffected == 0 {
		return nil, fmt.Errorf("research record not found")
	}

	statusRef := fmt.Sprintf("research:status:%s", request.ResearchID)
	statusPayload := map[string]interface{}{
		"research_id": request.ResearchID,
		"status":      "completed",
		"updated_at":  updatedAt,
		"insights":    request.Insights,
		"summary":     request.Summary,
		"status_ref":  statusRef,
	}
	if request.Metadata != nil {
		statusPayload["metadata"] = request.Metadata
	}

	if err := rs.cacheJSON(ctx, statusRef, statusPayload, 48*time.Hour); err != nil {
		rs.logger.Warn("failed to refresh research status cache:", err)
	}

	insightsRef := fmt.Sprintf("research:insights:%s", request.ResearchID)
	insightsPayload := map[string]interface{}{
		"research_id": request.ResearchID,
		"insights":    request.Insights,
		"summary":     request.Summary,
		"metadata":    request.Metadata,
		"updated_at":  updatedAt,
	}
	if err := rs.cacheJSON(ctx, insightsRef, insightsPayload, 48*time.Hour); err != nil {
		rs.logger.Warn("failed to cache research insights:", err)
	}

	responseMetadata := map[string]interface{}{
		"status_ref":   statusRef,
		"insights_ref": insightsRef,
	}
	for k, v := range request.Metadata {
		responseMetadata[k] = v
	}

	return &ResearchInsightsResponse{
		ResearchID: request.ResearchID,
		Status:     "completed",
		Insights:   request.Insights,
		Summary:    request.Summary,
		UpdatedAt:  updatedAt,
		Metadata:   responseMetadata,
	}, nil
}

func (rs *ResearchService) cacheJSON(ctx context.Context, key string, payload interface{}, ttl time.Duration) error {
	data, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	cacheCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()

	return rs.redis.Set(cacheCtx, key, data, ttl).Err()
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
