package main

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// CapabilityRegistry manages content-addressable capability storage
type CapabilityRegistry struct {
	logger          *logrus.Logger
	redis           *redis.Client
	db              *sql.DB
	config          *RegistryConfig
	capabilities    map[string]*RegisteredCapability
	compatMatrix    *CompatibilityMatrix
	dependencyGraph *DependencyGraph
	versionIndex    map[string][]string
	mutex           sync.RWMutex
	metrics         *RegistryMetrics
}

// RegistryConfig holds configuration for the capability registry
type RegistryConfig struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	StorageBackend   string `yaml:"storage_backend"` // redis, postgres, filesystem
	ContentAddressed bool   `yaml:"content_addressed"`
	SignatureRequired bool  `yaml:"signature_required"`
	RetentionDays    int    `yaml:"retention_days"`
	MaxVersions      int    `yaml:"max_versions"`
}

// RegisteredCapability represents a capability in the registry
type RegisteredCapability struct {
	ID              string                 `json:"id"`
	ContentHash     string                 `json:"content_hash"`
	Capability      *CapabilityPack        `json:"capability"`
	RegistrationID  string                 `json:"registration_id"`
	Status          string                 `json:"status"` // registered, validated, promoted, deprecated
	Source          string                 `json:"source"`
	Provenance      *ProvenanceRecord      `json:"provenance"`
	Validation      *ValidationRecord      `json:"validation"`
	Dependencies    []string               `json:"dependencies"`
	Dependents      []string               `json:"dependents"`
	Conflicts       []string               `json:"conflicts"`
	Versions        []string               `json:"versions"`
	LatestVersion   string                 `json:"latest_version"`
	Metadata        map[string]interface{} `json:"metadata"`
	RegisteredAt    time.Time              `json:"registered_at"`
	UpdatedAt       time.Time              `json:"updated_at"`
	AccessCount     int64                  `json:"access_count"`
	LastAccessed    time.Time              `json:"last_accessed"`
}

// CapabilityPack represents the capability specification (from ingestor)
type CapabilityPack struct {
	Kind       string                 `json:"kind" yaml:"kind"`
	APIVersion string                 `json:"apiVersion" yaml:"apiVersion"`
	Metadata   CapabilityMetadata     `json:"metadata" yaml:"metadata"`
	Spec       CapabilitySpec         `json:"spec" yaml:"spec"`
	Signature  CapabilitySignature    `json:"signature" yaml:"signature"`
}

// CapabilityMetadata holds capability metadata
type CapabilityMetadata struct {
	ID           string            `json:"id" yaml:"id"`
	Type         string            `json:"type" yaml:"type"`
	Version      string            `json:"version" yaml:"version"`
	Name         string            `json:"name" yaml:"name"`
	Description  string            `json:"description" yaml:"description"`
	CreatedAt    time.Time         `json:"created_at" yaml:"created_at"`
	Issuer       string            `json:"issuer" yaml:"issuer"`
	Tags         []string          `json:"tags" yaml:"tags"`
	Dependencies []string          `json:"dependencies" yaml:"dependencies"`
	Conflicts    []string          `json:"conflicts" yaml:"conflicts"`
}

// CapabilitySpec holds capability specification
type CapabilitySpec struct {
	Purpose       string                 `json:"purpose" yaml:"purpose"`
	Inputs        []CapabilityIO         `json:"inputs" yaml:"inputs"`
	Outputs       []CapabilityIO         `json:"outputs" yaml:"outputs"`
	Dependencies  CapabilityDeps         `json:"deps" yaml:"deps"`
	Risks         CapabilityRisks        `json:"risks" yaml:"risks"`
	Tests         CapabilityTests        `json:"tests" yaml:"tests"`
	Rollout       CapabilityRollout      `json:"rollout" yaml:"rollout"`
	Budgets       CapabilityBudgets      `json:"budgets" yaml:"budgets"`
	Observability CapabilityObservability `json:"observability" yaml:"observability"`
}

// CapabilityIO represents input/output specification
type CapabilityIO struct {
	Name        string `json:"name" yaml:"name"`
	Type        string `json:"type" yaml:"type"`
	Schema      string `json:"schema" yaml:"schema"`
	Required    bool   `json:"required" yaml:"required"`
	Description string `json:"description" yaml:"description"`
}

// CapabilityDeps represents capability dependencies
type CapabilityDeps struct {
	OS        []string `json:"os" yaml:"os"`
	GPU       []string `json:"gpu" yaml:"gpu"`
	Network   []string `json:"net" yaml:"net"`
	MemoryGB  int      `json:"memory_gb" yaml:"memory_gb"`
	StorageGB int      `json:"storage_gb" yaml:"storage_gb"`
	CPUCores  int      `json:"cpu_cores" yaml:"cpu_cores"`
}

// CapabilityRisks represents risk assessment
type CapabilityRisks struct {
	Privacy        string `json:"privacy" yaml:"privacy"`
	SupplyChain    string `json:"supply_chain" yaml:"supply_chain"`
	License        string `json:"license" yaml:"license"`
	Security       string `json:"security" yaml:"security"`
	Cost           string `json:"cost" yaml:"cost"`
	Constitutional string `json:"constitutional" yaml:"constitutional"`
}

// CapabilityTests represents test configuration
type CapabilityTests struct {
	Unit           []string               `json:"unit" yaml:"unit"`
	Integration    []string               `json:"integration" yaml:"integration"`
	Soak           map[string]interface{} `json:"soak" yaml:"soak"`
	Security       []string               `json:"security" yaml:"security"`
	Constitutional []string               `json:"constitutional" yaml:"constitutional"`
}

// CapabilityRollout represents rollout configuration
type CapabilityRollout struct {
	Strategy  string                 `json:"strategy" yaml:"strategy"`
	Canary    map[string]interface{} `json:"canary" yaml:"canary"`
	BlueGreen map[string]interface{} `json:"blue_green" yaml:"blue_green"`
	Rollback  map[string]interface{} `json:"rollback" yaml:"rollback"`
}

// CapabilityBudgets represents resource budgets
type CapabilityBudgets struct {
	MonthlyCostUSD  int `json:"monthly_cost_usd" yaml:"monthly_cost_usd"`
	VRAMGB          int `json:"vram_gb" yaml:"vram_gb"`
	IOPS            int `json:"iops" yaml:"iops"`
	BandwidthMbps   int `json:"bandwidth_mbps" yaml:"bandwidth_mbps"`
	CPUHours        int `json:"cpu_hours" yaml:"cpu_hours"`
}

// CapabilityObservability represents observability configuration
type CapabilityObservability struct {
	Logs          bool    `json:"logs" yaml:"logs"`
	Traces        bool    `json:"traces" yaml:"traces"`
	Metrics       bool    `json:"metrics" yaml:"metrics"`
	Redaction     string  `json:"redaction" yaml:"redaction"`
	RetentionDays int     `json:"retention_days" yaml:"retention_days"`
	SamplingRate  float64 `json:"sampling_rate" yaml:"sampling_rate"`
}

// CapabilitySignature represents cryptographic signature
type CapabilitySignature struct {
	SHA256      string    `json:"sha256" yaml:"sha256"`
	Issuer      string    `json:"issuer" yaml:"issuer"`
	Timestamp   time.Time `json:"timestamp" yaml:"timestamp"`
	Algorithm   string    `json:"algorithm" yaml:"algorithm"`
	Certificate string    `json:"certificate" yaml:"certificate"`
}

// ProvenanceRecord tracks capability origin and history
type ProvenanceRecord struct {
	SourceType      string                 `json:"source_type"`
	SourceURL       string                 `json:"source_url"`
	SourceCommit    string                 `json:"source_commit"`
	BuildInfo       map[string]interface{} `json:"build_info"`
	SandboxResults  map[string]interface{} `json:"sandbox_results"`
	Chain           []ProvenanceStep       `json:"chain"`
	Verified        bool                   `json:"verified"`
	VerificationLog []string               `json:"verification_log"`
}

// ProvenanceStep represents a step in the provenance chain
type ProvenanceStep struct {
	Timestamp   time.Time              `json:"timestamp"`
	Action      string                 `json:"action"`
	Actor       string                 `json:"actor"`
	Context     map[string]interface{} `json:"context"`
	Signature   string                 `json:"signature"`
}

// ValidationRecord tracks validation results
type ValidationRecord struct {
	ValidationID    string                 `json:"validation_id"`
	Status          string                 `json:"status"` // pending, passed, failed, conditional
	Results         map[string]interface{} `json:"results"`
	Constitutional  *ConstitutionalResult  `json:"constitutional"`
	Security        *SecurityResult        `json:"security"`
	Performance     *PerformanceResult     `json:"performance"`
	Compliance      *ComplianceResult      `json:"compliance"`
	ValidatedAt     time.Time              `json:"validated_at"`
	ValidatedBy     string                 `json:"validated_by"`
	ExpiresAt       time.Time              `json:"expires_at"`
}

// ConstitutionalResult represents Trifecta Court validation
type ConstitutionalResult struct {
	ScriptureCourt string                 `json:"scripture_court"`
	GeometryCourt  string                 `json:"geometry_court"`
	BridgePathCourt string                `json:"bridge_path_court"`
	OverallVerdict string                 `json:"overall_verdict"`
	Conditions     []string               `json:"conditions"`
	Reasoning      map[string]interface{} `json:"reasoning"`
}

// SecurityResult represents security validation
type SecurityResult struct {
	VulnerabilityCount int                    `json:"vulnerability_count"`
	HighSeverityCount  int                    `json:"high_severity_count"`
	LicenseIssues      []string               `json:"license_issues"`
	SecurityScore      float64                `json:"security_score"`
	Recommendations    []string               `json:"recommendations"`
	ScanResults        map[string]interface{} `json:"scan_results"`
}

// PerformanceResult represents performance validation
type PerformanceResult struct {
	LatencyP95        float64                `json:"latency_p95_ms"`
	ThroughputRPS     float64                `json:"throughput_rps"`
	ResourceUsage     map[string]float64     `json:"resource_usage"`
	CostEstimate      float64                `json:"cost_estimate_usd"`
	PerformanceScore  float64                `json:"performance_score"`
	BenchmarkResults  map[string]interface{} `json:"benchmark_results"`
}

// ComplianceResult represents compliance validation
type ComplianceResult struct {
	GDPRCompliant     bool                   `json:"gdpr_compliant"`
	SOCCompliant      bool                   `json:"soc_compliant"`
	HIPAACompliant    bool                   `json:"hipaa_compliant"`
	ComplianceScore   float64                `json:"compliance_score"`
	Issues            []string               `json:"issues"`
	ComplianceChecks  map[string]interface{} `json:"compliance_checks"`
}

// CompatibilityMatrix tracks capability compatibility
type CompatibilityMatrix struct {
	registry *CapabilityRegistry
	matrix   map[string]map[string]*CompatibilityEntry
	mutex    sync.RWMutex
}

// CompatibilityEntry represents compatibility between capabilities
type CompatibilityEntry struct {
	Compatible    bool                   `json:"compatible"`
	Conflicts     []string               `json:"conflicts"`
	Requirements  []string               `json:"requirements"`
	Tested        bool                   `json:"tested"`
	TestResults   map[string]interface{} `json:"test_results"`
	LastTested    time.Time              `json:"last_tested"`
	Confidence    float64                `json:"confidence"`
}

// DependencyGraph tracks capability dependencies
type DependencyGraph struct {
	registry     *CapabilityRegistry
	dependencies map[string][]string
	dependents   map[string][]string
	mutex        sync.RWMutex
}

// RegistryMetrics tracks registry performance metrics
type RegistryMetrics struct {
	CapabilityCount     prometheus.Gauge
	RegistrationRate    prometheus.Counter
	AccessRate          prometheus.Counter
	ValidationRate      prometheus.Counter
	StorageSize         prometheus.Gauge
	DependencyDepth     prometheus.Histogram
	CompatibilityScore  prometheus.Gauge
}

// RegistrationRequest represents a capability registration request
type RegistrationRequest struct {
	Capability   *CapabilityPack        `json:"capability"`
	Source       string                 `json:"source"`
	Provenance   *ProvenanceRecord      `json:"provenance"`
	Validation   *ValidationRecord      `json:"validation"`
	Metadata     map[string]interface{} `json:"metadata"`
	ForceUpdate  bool                   `json:"force_update"`
}

// RegistrationResponse represents the registration result
type RegistrationResponse struct {
	Success        bool                   `json:"success"`
	RegistrationID string                 `json:"registration_id"`
	ContentHash    string                 `json:"content_hash"`
	Status         string                 `json:"status"`
	Conflicts      []string               `json:"conflicts"`
	Dependencies   []string               `json:"dependencies"`
	Metadata       map[string]interface{} `json:"metadata"`
	ErrorMessage   string                 `json:"error_message"`
}

// QueryRequest represents a capability query
type QueryRequest struct {
	ID           string            `json:"id"`
	Type         string            `json:"type"`
	Version      string            `json:"version"`
	Tags         []string          `json:"tags"`
	Dependencies []string          `json:"dependencies"`
	Filters      map[string]string `json:"filters"`
	Limit        int               `json:"limit"`
	Offset       int               `json:"offset"`
}

// QueryResponse represents query results
type QueryResponse struct {
	Capabilities []*RegisteredCapability `json:"capabilities"`
	Total        int                     `json:"total"`
	Offset       int                     `json:"offset"`
	Limit        int                     `json:"limit"`
	Query        *QueryRequest           `json:"query"`
}

func NewCapabilityRegistry(config *RegistryConfig) (*CapabilityRegistry, error) {
	// Initialize logger
	logger := logrus.New()
	logger.SetFormatter(&logrus.JSONFormatter{})
	logger.SetLevel(logrus.InfoLevel)

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

	// Initialize metrics
	metrics := &RegistryMetrics{
		CapabilityCount: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "capability_registry_total",
			Help: "Total number of registered capabilities",
		}),
		RegistrationRate: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "capability_registrations_total",
			Help: "Total number of capability registrations",
		}),
		AccessRate: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "capability_accesses_total",
			Help: "Total number of capability accesses",
		}),
		ValidationRate: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "capability_validations_total",
			Help: "Total number of capability validations",
		}),
		StorageSize: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "capability_storage_bytes",
			Help: "Total storage size of capabilities",
		}),
		DependencyDepth: prometheus.NewHistogram(prometheus.HistogramOpts{
			Name:    "capability_dependency_depth",
			Help:    "Dependency depth of capabilities",
			Buckets: prometheus.LinearBuckets(0, 1, 10),
		}),
		CompatibilityScore: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "capability_compatibility_score",
			Help: "Overall compatibility score of the registry",
		}),
	}

	// Register metrics
	prometheus.MustRegister(
		metrics.CapabilityCount,
		metrics.RegistrationRate,
		metrics.AccessRate,
		metrics.ValidationRate,
		metrics.StorageSize,
		metrics.DependencyDepth,
		metrics.CompatibilityScore,
	)

	registry := &CapabilityRegistry{
		logger:       logger,
		redis:        redisClient,
		db:           db,
		config:       config,
		capabilities: make(map[string]*RegisteredCapability),
		versionIndex: make(map[string][]string),
		metrics:      metrics,
	}

	// Initialize compatibility matrix
	registry.compatMatrix = &CompatibilityMatrix{
		registry: registry,
		matrix:   make(map[string]map[string]*CompatibilityEntry),
	}

	// Initialize dependency graph
	registry.dependencyGraph = &DependencyGraph{
		registry:     registry,
		dependencies: make(map[string][]string),
		dependents:   make(map[string][]string),
	}

	// Initialize database schema
	if err := registry.initializeSchema(); err != nil {
		return nil, fmt.Errorf("failed to initialize schema: %w", err)
	}

	// Load existing capabilities
	if err := registry.loadCapabilities(); err != nil {
		logger.Warn("Failed to load existing capabilities:", err)
	}

	return registry, nil
}

func (cr *CapabilityRegistry) Start() error {
	cr.logger.Info("🗄️  Starting Capability Registry")

	// Start background tasks
	go cr.maintenanceRoutine()
	go cr.metricsUpdateRoutine()

	cr.logger.Info("✅ Capability Registry started successfully")
	return nil
}

func (cr *CapabilityRegistry) Stop() error {
	cr.logger.Info("🛑 Stopping Capability Registry")

	// Close database connection
	if err := cr.db.Close(); err != nil {
		cr.logger.Error("Failed to close database:", err)
	}

	// Close Redis connection
	if err := cr.redis.Close(); err != nil {
		cr.logger.Error("Failed to close Redis:", err)
	}

	cr.logger.Info("✅ Capability Registry stopped")
	return nil
}

func (cr *CapabilityRegistry) RegisterCapability(request *RegistrationRequest) (*RegistrationResponse, error) {
	startTime := time.Now()
	cr.logger.Infof("📝 Registering capability: %s", request.Capability.Metadata.ID)

	// Generate content hash
	contentHash, err := cr.generateContentHash(request.Capability)
	if err != nil {
		return &RegistrationResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Failed to generate content hash: %v", err),
		}, err
	}

	// Check if capability already exists
	cr.mutex.RLock()
	existing, exists := cr.capabilities[request.Capability.Metadata.ID]
	cr.mutex.RUnlock()

	if exists && !request.ForceUpdate {
		if existing.ContentHash == contentHash {
			return &RegistrationResponse{
				Success:        true,
				RegistrationID: existing.RegistrationID,
				ContentHash:    contentHash,
				Status:         "already_registered",
			}, nil
		}
	}

	// Validate capability
	if err := cr.validateCapability(request.Capability); err != nil {
		return &RegistrationResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Validation failed: %v", err),
		}, err
	}

	// Check dependencies
	dependencies, conflicts, err := cr.analyzeDependencies(request.Capability)
	if err != nil {
		return &RegistrationResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Dependency analysis failed: %v", err),
		}, err
	}

	if len(conflicts) > 0 && !request.ForceUpdate {
		return &RegistrationResponse{
			Success:      false,
			Status:       "conflicts_detected",
			Conflicts:    conflicts,
			ErrorMessage: "Capability conflicts detected",
		}, fmt.Errorf("conflicts detected: %v", conflicts)
	}

	// Create registration
	registrationID := cr.generateRegistrationID(request.Capability.Metadata.ID, contentHash)
	
	registered := &RegisteredCapability{
		ID:              request.Capability.Metadata.ID,
		ContentHash:     contentHash,
		Capability:      request.Capability,
		RegistrationID:  registrationID,
		Status:          "registered",
		Source:          request.Source,
		Provenance:      request.Provenance,
		Validation:      request.Validation,
		Dependencies:    dependencies,
		Conflicts:       conflicts,
		Versions:        []string{request.Capability.Metadata.Version},
		LatestVersion:   request.Capability.Metadata.Version,
		Metadata:        request.Metadata,
		RegisteredAt:    time.Now(),
		UpdatedAt:       time.Now(),
		AccessCount:     0,
	}

	// Store capability
	if err := cr.storeCapability(registered); err != nil {
		return &RegistrationResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Storage failed: %v", err),
		}, err
	}

	// Update in-memory structures
	cr.mutex.Lock()
	cr.capabilities[registered.ID] = registered
	cr.updateVersionIndex(registered)
	cr.mutex.Unlock()

	// Update dependency graph
	cr.dependencyGraph.updateDependencies(registered.ID, dependencies)

	// Update compatibility matrix
	cr.compatMatrix.updateCompatibility(registered)

	// Update metrics
	cr.metrics.RegistrationRate.Inc()
	cr.metrics.CapabilityCount.Set(float64(len(cr.capabilities)))

	duration := time.Since(startTime)
	cr.logger.Infof("✅ Capability registered: %s (duration: %v)", registered.ID, duration)

	return &RegistrationResponse{
		Success:        true,
		RegistrationID: registrationID,
		ContentHash:    contentHash,
		Status:         "registered",
		Dependencies:   dependencies,
		Metadata: map[string]interface{}{
			"registration_time": duration.String(),
			"version":          request.Capability.Metadata.Version,
		},
	}, nil
}

func (cr *CapabilityRegistry) GetCapability(id string, version string) (*RegisteredCapability, error) {
	cr.mutex.RLock()
	defer cr.mutex.RUnlock()

	capability, exists := cr.capabilities[id]
	if !exists {
		return nil, fmt.Errorf("capability not found: %s", id)
	}

	// Update access metrics
	capability.AccessCount++
	capability.LastAccessed = time.Now()
	cr.metrics.AccessRate.Inc()

	// If specific version requested, find it
	if version != "" && version != capability.LatestVersion {
		// TODO: Implement version-specific retrieval
		return nil, fmt.Errorf("version-specific retrieval not implemented")
	}

	cr.logger.Infof("📖 Retrieved capability: %s (version: %s)", id, capability.LatestVersion)
	return capability, nil
}

func (cr *CapabilityRegistry) QueryCapabilities(query *QueryRequest) (*QueryResponse, error) {
	cr.mutex.RLock()
	defer cr.mutex.RUnlock()

	var results []*RegisteredCapability

	// Apply filters
	for _, capability := range cr.capabilities {
		if cr.matchesQuery(capability, query) {
			results = append(results, capability)
		}
	}

	// Sort results
	sort.Slice(results, func(i, j int) bool {
		return results[i].UpdatedAt.After(results[j].UpdatedAt)
	})

	// Apply pagination
	total := len(results)
	start := query.Offset
	end := start + query.Limit

	if start > total {
		start = total
	}
	if end > total {
		end = total
	}

	if query.Limit > 0 {
		results = results[start:end]
	}

	cr.logger.Infof("🔍 Query executed: found %d/%d capabilities", len(results), total)

	return &QueryResponse{
		Capabilities: results,
		Total:        total,
		Offset:       query.Offset,
		Limit:        query.Limit,
		Query:        query,
	}, nil
}

func (cr *CapabilityRegistry) UpdateCapability(id string, request *RegistrationRequest) (*RegistrationResponse, error) {
	cr.logger.Infof("🔄 Updating capability: %s", id)

	// Force update for existing capability
	request.ForceUpdate = true
	
	return cr.RegisterCapability(request)
}

func (cr *CapabilityRegistry) DeleteCapability(id string, version string) error {
	cr.mutex.Lock()
	defer cr.mutex.Unlock()

	capability, exists := cr.capabilities[id]
	if !exists {
		return fmt.Errorf("capability not found: %s", id)
	}

	// Check if capability has dependents
	dependents := cr.dependencyGraph.getDependents(id)
	if len(dependents) > 0 {
		return fmt.Errorf("capability has dependents: %v", dependents)
	}

	// Remove from storage
	if err := cr.removeCapabilityFromStorage(id); err != nil {
		return fmt.Errorf("failed to remove from storage: %w", err)
	}

	// Remove from in-memory structures
	delete(cr.capabilities, id)
	cr.removeFromVersionIndex(capability)

	// Update dependency graph
	cr.dependencyGraph.removeDependencies(id)

	// Update compatibility matrix
	cr.compatMatrix.removeCompatibility(id)

	// Update metrics
	cr.metrics.CapabilityCount.Set(float64(len(cr.capabilities)))

	cr.logger.Infof("🗑️  Capability deleted: %s", id)
	return nil
}

func (cr *CapabilityRegistry) GetDependencies(id string) ([]string, error) {
	cr.dependencyGraph.mutex.RLock()
	defer cr.dependencyGraph.mutex.RUnlock()

	dependencies, exists := cr.dependencyGraph.dependencies[id]
	if !exists {
		return []string{}, nil
	}

	return dependencies, nil
}

func (cr *CapabilityRegistry) GetDependents(id string) ([]string, error) {
	cr.dependencyGraph.mutex.RLock()
	defer cr.dependencyGraph.mutex.RUnlock()

	dependents, exists := cr.dependencyGraph.dependents[id]
	if !exists {
		return []string{}, nil
	}

	return dependents, nil
}

func (cr *CapabilityRegistry) CheckCompatibility(id1, id2 string) (*CompatibilityEntry, error) {
	cr.compatMatrix.mutex.RLock()
	defer cr.compatMatrix.mutex.RUnlock()

	if matrix, exists := cr.compatMatrix.matrix[id1]; exists {
		if entry, exists := matrix[id2]; exists {
			return entry, nil
		}
	}

	// If not found, return default compatibility
	return &CompatibilityEntry{
		Compatible: true,
		Conflicts:  []string{},
		Tested:     false,
		Confidence: 0.5,
	}, nil
}

func (cr *CapabilityRegistry) GetRegistryStats() map[string]interface{} {
	cr.mutex.RLock()
	defer cr.mutex.RUnlock()

	stats := map[string]interface{}{
		"total_capabilities": len(cr.capabilities),
		"total_versions":     cr.getTotalVersions(),
		"storage_backend":    cr.config.StorageBackend,
		"content_addressed":  cr.config.ContentAddressed,
		"dependency_graph": map[string]interface{}{
			"total_dependencies": len(cr.dependencyGraph.dependencies),
			"total_dependents":   len(cr.dependencyGraph.dependents),
		},
		"compatibility_matrix": map[string]interface{}{
			"total_entries": cr.compatMatrix.getTotalEntries(),
		},
	}

	return stats
}

// Internal methods

func (cr *CapabilityRegistry) generateContentHash(capability *CapabilityPack) (string, error) {
	// Serialize capability for hashing
	data, err := json.Marshal(capability)
	if err != nil {
		return "", err
	}

	// Generate SHA-256 hash
	hash := sha256.Sum256(data)
	return hex.EncodeToString(hash[:]), nil
}

func (cr *CapabilityRegistry) generateRegistrationID(id, contentHash string) string {
	timestamp := time.Now().Unix()
	data := fmt.Sprintf("%s:%s:%d", id, contentHash, timestamp)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])[:16]
}

func (cr *CapabilityRegistry) validateCapability(capability *CapabilityPack) error {
	// Validate required fields
	if capability.Kind != "Capability" {
		return fmt.Errorf("invalid kind: %s", capability.Kind)
	}

	if capability.Metadata.ID == "" {
		return fmt.Errorf("missing capability ID")
	}

	if capability.Metadata.Version == "" {
		return fmt.Errorf("missing capability version")
	}

	if capability.Spec.Purpose == "" {
		return fmt.Errorf("missing capability purpose")
	}

	// Validate signature if required
	if cr.config.SignatureRequired {
		if err := cr.validateSignature(capability); err != nil {
			return fmt.Errorf("signature validation failed: %w", err)
		}
	}

	return nil
}

func (cr *CapabilityRegistry) validateSignature(capability *CapabilityPack) error {
	// Implementation for cryptographic signature validation
	if capability.Signature.SHA256 == "" {
		return fmt.Errorf("missing SHA-256 signature")
	}

	if capability.Signature.Issuer == "" {
		return fmt.Errorf("missing signature issuer")
	}

	// TODO: Implement actual cryptographic verification
	return nil
}

func (cr *CapabilityRegistry) analyzeDependencies(capability *CapabilityPack) ([]string, []string, error) {
	dependencies := capability.Metadata.Dependencies
	conflicts := capability.Metadata.Conflicts

	// Validate dependencies exist
	for _, dep := range dependencies {
		if _, err := cr.GetCapability(dep, ""); err != nil {
			cr.logger.Warnf("Dependency not found: %s", dep)
		}
	}

	// Check for conflicts
	var actualConflicts []string
	for _, conflict := range conflicts {
		if _, err := cr.GetCapability(conflict, ""); err == nil {
			actualConflicts = append(actualConflicts, conflict)
		}
	}

	return dependencies, actualConflicts, nil
}

func (cr *CapabilityRegistry) storeCapability(capability *RegisteredCapability) error {
	switch cr.config.StorageBackend {
	case "redis":
		return cr.storeInRedis(capability)
	case "postgres":
		return cr.storeInPostgres(capability)
	case "filesystem":
		return cr.storeInFilesystem(capability)
	default:
		return cr.storeInRedis(capability) // Default to Redis
	}
}

func (cr *CapabilityRegistry) storeInRedis(capability *RegisteredCapability) error {
	data, err := json.Marshal(capability)
	if err != nil {
		return err
	}

	ctx := context.Background()
	key := fmt.Sprintf("capability:%s", capability.ID)
	
	return cr.redis.Set(ctx, key, data, 0).Err()
}

func (cr *CapabilityRegistry) storeInPostgres(capability *RegisteredCapability) error {
	query := `
		INSERT INTO capabilities (
			id, content_hash, capability_data, registration_id, status, source,
			provenance, validation, dependencies, conflicts, versions, latest_version,
			metadata, registered_at, updated_at, access_count
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
		ON CONFLICT (id) DO UPDATE SET
			content_hash = EXCLUDED.content_hash,
			capability_data = EXCLUDED.capability_data,
			status = EXCLUDED.status,
			validation = EXCLUDED.validation,
			versions = EXCLUDED.versions,
			latest_version = EXCLUDED.latest_version,
			metadata = EXCLUDED.metadata,
			updated_at = EXCLUDED.updated_at
	`

	capabilityData, _ := json.Marshal(capability.Capability)
	provenanceData, _ := json.Marshal(capability.Provenance)
	validationData, _ := json.Marshal(capability.Validation)
	dependenciesData, _ := json.Marshal(capability.Dependencies)
	conflictsData, _ := json.Marshal(capability.Conflicts)
	versionsData, _ := json.Marshal(capability.Versions)
	metadataData, _ := json.Marshal(capability.Metadata)

	_, err := cr.db.Exec(query,
		capability.ID,
		capability.ContentHash,
		capabilityData,
		capability.RegistrationID,
		capability.Status,
		capability.Source,
		provenanceData,
		validationData,
		dependenciesData,
		conflictsData,
		versionsData,
		capability.LatestVersion,
		metadataData,
		capability.RegisteredAt,
		capability.UpdatedAt,
		capability.AccessCount,
	)

	return err
}

func (cr *CapabilityRegistry) storeInFilesystem(capability *RegisteredCapability) error {
	// Implementation for filesystem storage
	return fmt.Errorf("filesystem storage not implemented")
}

func (cr *CapabilityRegistry) removeCapabilityFromStorage(id string) error {
	switch cr.config.StorageBackend {
	case "redis":
		ctx := context.Background()
		return cr.redis.Del(ctx, fmt.Sprintf("capability:%s", id)).Err()
	case "postgres":
		_, err := cr.db.Exec("DELETE FROM capabilities WHERE id = $1", id)
		return err
	case "filesystem":
		return fmt.Errorf("filesystem storage not implemented")
	default:
		ctx := context.Background()
		return cr.redis.Del(ctx, fmt.Sprintf("capability:%s", id)).Err()
	}
}

func (cr *CapabilityRegistry) matchesQuery(capability *RegisteredCapability, query *QueryRequest) bool {
	// ID filter
	if query.ID != "" && capability.ID != query.ID {
		return false
	}

	// Type filter
	if query.Type != "" && capability.Capability.Metadata.Type != query.Type {
		return false
	}

	// Version filter
	if query.Version != "" && capability.LatestVersion != query.Version {
		return false
	}

	// Tags filter
	if len(query.Tags) > 0 {
		capabilityTags := capability.Capability.Metadata.Tags
		for _, tag := range query.Tags {
			found := false
			for _, capTag := range capabilityTags {
				if capTag == tag {
					found = true
					break
				}
			}
			if !found {
				return false
			}
		}
	}

	// Dependencies filter
	if len(query.Dependencies) > 0 {
		for _, dep := range query.Dependencies {
			found := false
			for _, capDep := range capability.Dependencies {
				if capDep == dep {
					found = true
					break
				}
			}
			if !found {
				return false
			}
		}
	}

	// Custom filters
	for key, value := range query.Filters {
		if !cr.matchesCustomFilter(capability, key, value) {
			return false
		}
	}

	return true
}

func (cr *CapabilityRegistry) matchesCustomFilter(capability *RegisteredCapability, key, value string) bool {
	switch key {
	case "status":
		return capability.Status == value
	case "source":
		return capability.Source == value
	case "issuer":
		return capability.Capability.Signature.Issuer == value
	default:
		// Check metadata
		if metaValue, exists := capability.Metadata[key]; exists {
			return fmt.Sprintf("%v", metaValue) == value
		}
		return false
	}
}

func (cr *CapabilityRegistry) updateVersionIndex(capability *RegisteredCapability) {
	versions, exists := cr.versionIndex[capability.ID]
	if !exists {
		versions = []string{}
	}

	// Add version if not exists
	found := false
	for _, v := range versions {
		if v == capability.LatestVersion {
			found = true
			break
		}
	}

	if !found {
		versions = append(versions, capability.LatestVersion)
		sort.Strings(versions)
		cr.versionIndex[capability.ID] = versions
	}
}

func (cr *CapabilityRegistry) removeFromVersionIndex(capability *RegisteredCapability) {
	delete(cr.versionIndex, capability.ID)
}

func (cr *CapabilityRegistry) getTotalVersions() int {
	total := 0
	for _, versions := range cr.versionIndex {
		total += len(versions)
	}
	return total
}

func (cr *CapabilityRegistry) loadCapabilities() error {
	switch cr.config.StorageBackend {
	case "redis":
		return cr.loadFromRedis()
	case "postgres":
		return cr.loadFromPostgres()
	case "filesystem":
		return cr.loadFromFilesystem()
	default:
		return cr.loadFromRedis()
	}
}

func (cr *CapabilityRegistry) loadFromRedis() error {
	ctx := context.Background()
	keys, err := cr.redis.Keys(ctx, "capability:*").Result()
	if err != nil {
		return err
	}

	for _, key := range keys {
		data, err := cr.redis.Get(ctx, key).Result()
		if err != nil {
			continue
		}

		var capability RegisteredCapability
		if err := json.Unmarshal([]byte(data), &capability); err != nil {
			continue
		}

		cr.capabilities[capability.ID] = &capability
		cr.updateVersionIndex(&capability)
	}

	return nil
}

func (cr *CapabilityRegistry) loadFromPostgres() error {
	query := `
		SELECT id, content_hash, capability_data, registration_id, status, source,
			   provenance, validation, dependencies, conflicts, versions, latest_version,
			   metadata, registered_at, updated_at, access_count, last_accessed
		FROM capabilities
	`

	rows, err := cr.db.Query(query)
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var capability RegisteredCapability
		var capabilityData, provenanceData, validationData []byte
		var dependenciesData, conflictsData, versionsData, metadataData []byte
		var lastAccessed sql.NullTime

		err := rows.Scan(
			&capability.ID,
			&capability.ContentHash,
			&capabilityData,
			&capability.RegistrationID,
			&capability.Status,
			&capability.Source,
			&provenanceData,
			&validationData,
			&dependenciesData,
			&conflictsData,
			&versionsData,
			&capability.LatestVersion,
			&metadataData,
			&capability.RegisteredAt,
			&capability.UpdatedAt,
			&capability.AccessCount,
			&lastAccessed,
		)
		if err != nil {
			continue
		}

		// Unmarshal JSON fields
		json.Unmarshal(capabilityData, &capability.Capability)
		json.Unmarshal(provenanceData, &capability.Provenance)
		json.Unmarshal(validationData, &capability.Validation)
		json.Unmarshal(dependenciesData, &capability.Dependencies)
		json.Unmarshal(conflictsData, &capability.Conflicts)
		json.Unmarshal(versionsData, &capability.Versions)
		json.Unmarshal(metadataData, &capability.Metadata)

		if lastAccessed.Valid {
			capability.LastAccessed = lastAccessed.Time
		}

		cr.capabilities[capability.ID] = &capability
		cr.updateVersionIndex(&capability)
	}

	return nil
}

func (cr *CapabilityRegistry) loadFromFilesystem() error {
	return fmt.Errorf("filesystem storage not implemented")
}

func (cr *CapabilityRegistry) initializeSchema() error {
	if cr.config.StorageBackend != "postgres" {
		return nil
	}

	schema := `
		CREATE TABLE IF NOT EXISTS capabilities (
			id VARCHAR(255) PRIMARY KEY,
			content_hash VARCHAR(64) NOT NULL,
			capability_data JSONB NOT NULL,
			registration_id VARCHAR(32) NOT NULL,
			status VARCHAR(50) NOT NULL,
			source VARCHAR(255),
			provenance JSONB,
			validation JSONB,
			dependencies JSONB,
			conflicts JSONB,
			versions JSONB,
			latest_version VARCHAR(50),
			metadata JSONB,
			registered_at TIMESTAMP NOT NULL,
			updated_at TIMESTAMP NOT NULL,
			access_count BIGINT DEFAULT 0,
			last_accessed TIMESTAMP
		);

		CREATE INDEX IF NOT EXISTS idx_capabilities_status ON capabilities(status);
		CREATE INDEX IF NOT EXISTS idx_capabilities_type ON capabilities((capability_data->>'metadata'->>'type'));
		CREATE INDEX IF NOT EXISTS idx_capabilities_updated ON capabilities(updated_at);
		CREATE INDEX IF NOT EXISTS idx_capabilities_content_hash ON capabilities(content_hash);
	`

	_, err := cr.db.Exec(schema)
	return err
}

func (cr *CapabilityRegistry) maintenanceRoutine() {
	ticker := time.NewTicker(1 * time.Hour)
	defer ticker.Stop()

	for range ticker.C {
		cr.performMaintenance()
	}
}

func (cr *CapabilityRegistry) performMaintenance() {
	cr.logger.Info("🧹 Performing registry maintenance")

	// Clean up expired capabilities
	cr.cleanupExpiredCapabilities()

	// Update compatibility matrix
	cr.compatMatrix.updateAllCompatibility()

	// Optimize dependency graph
	cr.dependencyGraph.optimize()

	cr.logger.Info("✅ Registry maintenance completed")
}

func (cr *CapabilityRegistry) cleanupExpiredCapabilities() {
	if cr.config.RetentionDays <= 0 {
		return
	}

	cutoff := time.Now().AddDate(0, 0, -cr.config.RetentionDays)

	cr.mutex.Lock()
	defer cr.mutex.Unlock()

	var toDelete []string
	for id, capability := range cr.capabilities {
		if capability.UpdatedAt.Before(cutoff) && capability.Status == "deprecated" {
			toDelete = append(toDelete, id)
		}
	}

	for _, id := range toDelete {
		cr.logger.Infof("🗑️  Cleaning up expired capability: %s", id)
		delete(cr.capabilities, id)
		cr.removeCapabilityFromStorage(id)
	}
}

func (cr *CapabilityRegistry) metricsUpdateRoutine() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		cr.updateMetrics()
	}
}

func (cr *CapabilityRegistry) updateMetrics() {
	cr.mutex.RLock()
	defer cr.mutex.RUnlock()

	// Update capability count
	cr.metrics.CapabilityCount.Set(float64(len(cr.capabilities)))

	// Update dependency depth
	for id := range cr.capabilities {
		depth := cr.dependencyGraph.calculateDepth(id)
		cr.metrics.DependencyDepth.Observe(float64(depth))
	}

	// Update compatibility score
	score := cr.compatMatrix.calculateOverallScore()
	cr.metrics.CompatibilityScore.Set(score)
}

// Dependency graph methods

func (dg *DependencyGraph) updateDependencies(id string, dependencies []string) {
	dg.mutex.Lock()
	defer dg.mutex.Unlock()

	// Remove old dependencies
	if oldDeps, exists := dg.dependencies[id]; exists {
		for _, dep := range oldDeps {
			dg.removeDependentFromList(dep, id)
		}
	}

	// Add new dependencies
	dg.dependencies[id] = dependencies
	for _, dep := range dependencies {
		dg.addDependentToList(dep, id)
	}
}

func (dg *DependencyGraph) removeDependencies(id string) {
	dg.mutex.Lock()
	defer dg.mutex.Unlock()

	// Remove from dependencies
	if deps, exists := dg.dependencies[id]; exists {
		for _, dep := range deps {
			dg.removeDependentFromList(dep, id)
		}
		delete(dg.dependencies, id)
	}

	// Remove from dependents
	delete(dg.dependents, id)
}

func (dg *DependencyGraph) getDependents(id string) []string {
	dg.mutex.RLock()
	defer dg.mutex.RUnlock()

	if dependents, exists := dg.dependents[id]; exists {
		return dependents
	}
	return []string{}
}

func (dg *DependencyGraph) addDependentToList(dependency, dependent string) {
	if dependents, exists := dg.dependents[dependency]; exists {
		// Check if already exists
		for _, d := range dependents {
			if d == dependent {
				return
			}
		}
		dg.dependents[dependency] = append(dependents, dependent)
	} else {
		dg.dependents[dependency] = []string{dependent}
	}
}

func (dg *DependencyGraph) removeDependentFromList(dependency, dependent string) {
	if dependents, exists := dg.dependents[dependency]; exists {
		for i, d := range dependents {
			if d == dependent {
				dg.dependents[dependency] = append(dependents[:i], dependents[i+1:]...)
				break
			}
		}
	}
}

func (dg *DependencyGraph) calculateDepth(id string) int {
	visited := make(map[string]bool)
	return dg.calculateDepthRecursive(id, visited)
}

func (dg *DependencyGraph) calculateDepthRecursive(id string, visited map[string]bool) int {
	if visited[id] {
		return 0 // Circular dependency
	}

	visited[id] = true
	defer delete(visited, id)

	dependencies, exists := dg.dependencies[id]
	if !exists || len(dependencies) == 0 {
		return 0
	}

	maxDepth := 0
	for _, dep := range dependencies {
		depth := dg.calculateDepthRecursive(dep, visited)
		if depth > maxDepth {
			maxDepth = depth
		}
	}

	return maxDepth + 1
}

func (dg *DependencyGraph) optimize() {
	// Implementation for dependency graph optimization
	// This could include cycle detection, redundancy removal, etc.
}

// Compatibility matrix methods

func (cm *CompatibilityMatrix) updateCompatibility(capability *RegisteredCapability) {
	cm.mutex.Lock()
	defer cm.mutex.Unlock()

	id := capability.ID

	// Initialize matrix entry for this capability
	if _, exists := cm.matrix[id]; !exists {
		cm.matrix[id] = make(map[string]*CompatibilityEntry)
	}

	// Check compatibility with all other capabilities
	for otherId := range cm.registry.capabilities {
		if otherId != id {
			entry := cm.calculateCompatibility(capability, cm.registry.capabilities[otherId])
			cm.matrix[id][otherId] = entry

			// Ensure symmetric entry
			if _, exists := cm.matrix[otherId]; !exists {
				cm.matrix[otherId] = make(map[string]*CompatibilityEntry)
			}
			cm.matrix[otherId][id] = entry
		}
	}
}

func (cm *CompatibilityMatrix) removeCompatibility(id string) {
	cm.mutex.Lock()
	defer cm.mutex.Unlock()

	// Remove row
	delete(cm.matrix, id)

	// Remove column
	for otherId := range cm.matrix {
		delete(cm.matrix[otherId], id)
	}
}

func (cm *CompatibilityMatrix) calculateCompatibility(cap1, cap2 *RegisteredCapability) *CompatibilityEntry {
	entry := &CompatibilityEntry{
		Compatible:  true,
		Conflicts:   []string{},
		Requirements: []string{},
		Tested:      false,
		TestResults: make(map[string]interface{}),
		LastTested:  time.Time{},
		Confidence:  0.8, // Default confidence
	}

	// Check for explicit conflicts
	for _, conflict := range cap1.Capability.Metadata.Conflicts {
		if conflict == cap2.ID {
			entry.Compatible = false
			entry.Conflicts = append(entry.Conflicts, "explicit_conflict")
			entry.Confidence = 1.0
			return entry
		}
	}

	// Check resource conflicts
	if cm.hasResourceConflicts(cap1, cap2) {
		entry.Compatible = false
		entry.Conflicts = append(entry.Conflicts, "resource_conflict")
	}

	// Check dependency conflicts
	if cm.hasDependencyConflicts(cap1, cap2) {
		entry.Compatible = false
		entry.Conflicts = append(entry.Conflicts, "dependency_conflict")
	}

	return entry
}

func (cm *CompatibilityMatrix) hasResourceConflicts(cap1, cap2 *RegisteredCapability) bool {
	// Check if capabilities compete for exclusive resources
	// This is a simplified implementation
	return false
}

func (cm *CompatibilityMatrix) hasDependencyConflicts(cap1, cap2 *RegisteredCapability) bool {
	// Check if capabilities have conflicting dependencies
	// This is a simplified implementation
	return false
}

func (cm *CompatibilityMatrix) updateAllCompatibility() {
	cm.registry.mutex.RLock()
	capabilities := make([]*RegisteredCapability, 0, len(cm.registry.capabilities))
	for _, cap := range cm.registry.capabilities {
		capabilities = append(capabilities, cap)
	}
	cm.registry.mutex.RUnlock()

	for _, cap := range capabilities {
		cm.updateCompatibility(cap)
	}
}

func (cm *CompatibilityMatrix) getTotalEntries() int {
	cm.mutex.RLock()
	defer cm.mutex.RUnlock()

	total := 0
	for _, row := range cm.matrix {
		total += len(row)
	}
	return total
}

func (cm *CompatibilityMatrix) calculateOverallScore() float64 {
	cm.mutex.RLock()
	defer cm.mutex.RUnlock()

	if len(cm.matrix) == 0 {
		return 1.0
	}

	totalEntries := 0
	compatibleEntries := 0

	for _, row := range cm.matrix {
		for _, entry := range row {
			totalEntries++
			if entry.Compatible {
				compatibleEntries++
			}
		}
	}

	if totalEntries == 0 {
		return 1.0
	}

	return float64(compatibleEntries) / float64(totalEntries)
}

// REST API endpoints

func (cr *CapabilityRegistry) setupRoutes(router *gin.Engine) {
	api := router.Group("/api/v1")

	// Capability management
	api.POST("/capabilities", cr.registerCapabilityHandler)
	api.GET("/capabilities", cr.queryCapabilitiesHandler)
	api.GET("/capabilities/:id", cr.getCapabilityHandler)
	api.PUT("/capabilities/:id", cr.updateCapabilityHandler)
	api.DELETE("/capabilities/:id", cr.deleteCapabilityHandler)

	// Dependencies
	api.GET("/capabilities/:id/dependencies", cr.getDependenciesHandler)
	api.GET("/capabilities/:id/dependents", cr.getDependentsHandler)

	// Compatibility
	api.GET("/capabilities/:id1/compatibility/:id2", cr.checkCompatibilityHandler)

	// Registry information
	api.GET("/registry/stats", cr.getRegistryStatsHandler)
	api.GET("/registry/health", cr.getRegistryHealthHandler)

	// Metrics
	api.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (cr *CapabilityRegistry) registerCapabilityHandler(c *gin.Context) {
	var request RegistrationRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	response, err := cr.RegisterCapability(&request)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	if !response.Success {
		c.JSON(http.StatusConflict, response)
		return
	}

	c.JSON(http.StatusOK, response)
}

func (cr *CapabilityRegistry) queryCapabilitiesHandler(c *gin.Context) {
	query := &QueryRequest{
		Limit:  50, // Default limit
		Offset: 0,
	}

	// Parse query parameters
	if id := c.Query("id"); id != "" {
		query.ID = id
	}
	if capType := c.Query("type"); capType != "" {
		query.Type = capType
	}
	if version := c.Query("version"); version != "" {
		query.Version = version
	}
	if tags := c.Query("tags"); tags != "" {
		query.Tags = strings.Split(tags, ",")
	}

	response, err := cr.QueryCapabilities(query)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (cr *CapabilityRegistry) getCapabilityHandler(c *gin.Context) {
	id := c.Param("id")
	version := c.Query("version")

	capability, err := cr.GetCapability(id, version)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, capability)
}

func (cr *CapabilityRegistry) updateCapabilityHandler(c *gin.Context) {
	id := c.Param("id")

	var request RegistrationRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	response, err := cr.UpdateCapability(id, &request)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (cr *CapabilityRegistry) deleteCapabilityHandler(c *gin.Context) {
	id := c.Param("id")
	version := c.Query("version")

	if err := cr.DeleteCapability(id, version); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Capability deleted successfully",
	})
}

func (cr *CapabilityRegistry) getDependenciesHandler(c *gin.Context) {
	id := c.Param("id")

	dependencies, err := cr.GetDependencies(id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"capability_id": id,
		"dependencies":  dependencies,
		"count":         len(dependencies),
	})
}

func (cr *CapabilityRegistry) getDependentsHandler(c *gin.Context) {
	id := c.Param("id")

	dependents, err := cr.GetDependents(id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"capability_id": id,
		"dependents":    dependents,
		"count":         len(dependents),
	})
}

func (cr *CapabilityRegistry) checkCompatibilityHandler(c *gin.Context) {
	id1 := c.Param("id1")
	id2 := c.Param("id2")

	compatibility, err := cr.CheckCompatibility(id1, id2)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"capability1":   id1,
		"capability2":   id2,
		"compatibility": compatibility,
	})
}

func (cr *CapabilityRegistry) getRegistryStatsHandler(c *gin.Context) {
	stats := cr.GetRegistryStats()
	c.JSON(http.StatusOK, stats)
}

func (cr *CapabilityRegistry) getRegistryHealthHandler(c *gin.Context) {
	health := map[string]interface{}{
		"status":    "healthy",
		"service":   "capability-registry",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
	}

	// Check Redis connection
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	if err := cr.redis.Ping(ctx).Err(); err != nil {
		health["status"] = "degraded"
		health["redis_error"] = err.Error()
	}

	// Check database connection
	if err := cr.db.Ping(); err != nil {
		health["status"] = "degraded"
		health["database_error"] = err.Error()
	}

	status := http.StatusOK
	if health["status"] == "degraded" {
		status = http.StatusServiceUnavailable
	}

	c.JSON(status, health)
}

func main() {
	config := &RegistryConfig{
		ServerPort:        getEnv("SERVER_PORT", "8011"),
		RedisURL:          getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:       getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		StorageBackend:    getEnv("STORAGE_BACKEND", "redis"),
		ContentAddressed:  true,
		SignatureRequired: false,
		RetentionDays:     30,
		MaxVersions:       10,
	}

	registry, err := NewCapabilityRegistry(config)
	if err != nil {
		log.Fatal("Failed to create capability registry:", err)
	}

	if err := registry.Start(); err != nil {
		log.Fatal("Failed to start capability registry:", err)
	}

	// Setup HTTP server
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()
	router.Use(gin.Logger(), gin.Recovery())

	registry.setupRoutes(router)

	log.Printf("🗄️  Capability Registry listening on port %s", config.ServerPort)
	if err := router.Run(":" + config.ServerPort); err != nil {
		log.Fatal("Failed to start server:", err)
	}
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

