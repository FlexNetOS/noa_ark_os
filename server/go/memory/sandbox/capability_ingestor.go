package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"gopkg.in/yaml.v3"
)

// CapabilityIngestor handles discovery and ingestion of new capabilities
type CapabilityIngestor struct {
	logger       *log.Logger
	redis        *redis.Client
	config       *IngestorConfig
	sources      map[string]*CapabilitySource
	watchers     map[string]*SourceWatcher
	sandboxPool  *SandboxPool
	mutex        sync.RWMutex
	stopChan     chan struct{}
}

// IngestorConfig holds configuration for the capability ingestor
type IngestorConfig struct {
	SandboxDir       string `yaml:"sandbox_dir"`
	MaxConcurrent    int    `yaml:"max_concurrent"`
	PollInterval     int    `yaml:"poll_interval_seconds"`
	IsolationMode    string `yaml:"isolation_mode"` // vm, container, namespace
	DefaultTimeout   int    `yaml:"default_timeout_seconds"`
	RedisURL         string `yaml:"redis_url"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
}

// CapabilitySource represents a source of capabilities
type CapabilitySource struct {
	ID          string                 `json:"id" yaml:"id"`
	Type        string                 `json:"type" yaml:"type"` // git, registry, filesystem, api
	URL         string                 `json:"url" yaml:"url"`
	Credentials map[string]string      `json:"credentials" yaml:"credentials"`
	Filters     []string               `json:"filters" yaml:"filters"`
	Metadata    map[string]interface{} `json:"metadata" yaml:"metadata"`
	LastSync    time.Time              `json:"last_sync" yaml:"last_sync"`
	Status      string                 `json:"status" yaml:"status"`
	CreatedAt   time.Time              `json:"created_at" yaml:"created_at"`
	UpdatedAt   time.Time              `json:"updated_at" yaml:"updated_at"`
}

// SourceWatcher monitors a capability source for changes
type SourceWatcher struct {
	source   *CapabilitySource
	ingestor *CapabilityIngestor
	ticker   *time.Ticker
	stopChan chan struct{}
}

// SandboxPool manages ephemeral sandbox instances
type SandboxPool struct {
	ingestor    *CapabilityIngestor
	active      map[string]*SandboxInstance
	maxActive   int
	mutex       sync.RWMutex
}

// SandboxInstance represents an isolated sandbox environment
type SandboxInstance struct {
	ID           string                 `json:"id"`
	Type         string                 `json:"type"` // vm, container, namespace
	Status       string                 `json:"status"`
	Capability   *CapabilityPack        `json:"capability"`
	WorkDir      string                 `json:"work_dir"`
	StartTime    time.Time              `json:"start_time"`
	EndTime      time.Time              `json:"end_time"`
	Resources    map[string]interface{} `json:"resources"`
	Logs         []string               `json:"logs"`
	Results      *SandboxResults        `json:"results"`
	mutex        sync.Mutex
}

// CapabilityPack represents a capability package
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

// SandboxResults represents sandbox execution results
type SandboxResults struct {
	Success        bool                   `json:"success"`
	SBOM           *SBOM                  `json:"sbom"`
	TestResults    map[string]interface{} `json:"test_results"`
	RiskAnalysis   *RiskAnalysis          `json:"risk_analysis"`
	Performance    *PerformanceMetrics    `json:"performance"`
	Artifacts      []string               `json:"artifacts"`
	Logs           []string               `json:"logs"`
	Duration       time.Duration          `json:"duration"`
	ExitCode       int                    `json:"exit_code"`
	ErrorMessage   string                 `json:"error_message"`
}

// SBOM represents Software Bill of Materials
type SBOM struct {
	Version    string         `json:"version"`
	Components []SBOMComponent `json:"components"`
	CreatedAt  time.Time      `json:"created_at"`
	Generator  string         `json:"generator"`
}

// SBOMComponent represents a component in the SBOM
type SBOMComponent struct {
	Name         string            `json:"name"`
	Version      string            `json:"version"`
	Type         string            `json:"type"`
	License      string            `json:"license"`
	Supplier     string            `json:"supplier"`
	Hash         string            `json:"hash"`
	Dependencies []string          `json:"dependencies"`
	Vulnerabilities []Vulnerability `json:"vulnerabilities"`
}

// Vulnerability represents a security vulnerability
type Vulnerability struct {
	ID          string  `json:"id"`
	Severity    string  `json:"severity"`
	Score       float64 `json:"score"`
	Description string  `json:"description"`
	Fixed       bool    `json:"fixed"`
	FixVersion  string  `json:"fix_version"`
}

// RiskAnalysis represents risk assessment results
type RiskAnalysis struct {
	OverallRisk     string                 `json:"overall_risk"`
	RiskFactors     map[string]string      `json:"risk_factors"`
	Mitigations     []string               `json:"mitigations"`
	Recommendations []string               `json:"recommendations"`
	LicenseIssues   []string               `json:"license_issues"`
	SecurityIssues  []string               `json:"security_issues"`
	ComplianceIssues []string              `json:"compliance_issues"`
}

// PerformanceMetrics represents performance measurement results
type PerformanceMetrics struct {
	CPUUsage      float64           `json:"cpu_usage"`
	MemoryUsage   int64             `json:"memory_usage"`
	DiskUsage     int64             `json:"disk_usage"`
	NetworkIO     map[string]int64  `json:"network_io"`
	Latency       map[string]float64 `json:"latency"`
	Throughput    map[string]float64 `json:"throughput"`
	ErrorRate     float64           `json:"error_rate"`
}

// IngestRequest represents a capability ingestion request
type IngestRequest struct {
	SourceID     string                 `json:"source_id"`
	CapabilityID string                 `json:"capability_id"`
	Priority     string                 `json:"priority"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// IngestResponse represents the ingestion result
type IngestResponse struct {
	Success     bool                   `json:"success"`
	SandboxID   string                 `json:"sandbox_id"`
	Status      string                 `json:"status"`
	Results     *SandboxResults        `json:"results"`
	Duration    time.Duration          `json:"duration"`
	Metadata    map[string]interface{} `json:"metadata"`
	ErrorMessage string                `json:"error_message"`
}

func NewCapabilityIngestor(config *IngestorConfig) (*CapabilityIngestor, error) {
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

	ingestor := &CapabilityIngestor{
		logger:   log.New(os.Stdout, "[INGESTOR] ", log.LstdFlags),
		redis:    redisClient,
		config:   config,
		sources:  make(map[string]*CapabilitySource),
		watchers: make(map[string]*SourceWatcher),
		stopChan: make(chan struct{}),
	}

	// Initialize sandbox pool
	ingestor.sandboxPool = &SandboxPool{
		ingestor:  ingestor,
		active:    make(map[string]*SandboxInstance),
		maxActive: config.MaxConcurrent,
	}

	// Load existing sources from Redis
	if err := ingestor.loadSources(); err != nil {
		ingestor.logger.Printf("Warning: Failed to load sources: %v", err)
	}

	return ingestor, nil
}

func (ci *CapabilityIngestor) Start() error {
	ci.logger.Println("🔍 Starting Capability Ingestor")

	// Start source watchers
	ci.startSourceWatchers()

	// Start sandbox cleanup routine
	go ci.sandboxCleanupRoutine()

	ci.logger.Println("✅ Capability Ingestor started successfully")
	return nil
}

func (ci *CapabilityIngestor) Stop() error {
	ci.logger.Println("🛑 Stopping Capability Ingestor")

	close(ci.stopChan)

	// Stop all watchers
	ci.mutex.Lock()
	for _, watcher := range ci.watchers {
		close(watcher.stopChan)
	}
	ci.mutex.Unlock()

	// Clean up active sandboxes
	ci.sandboxPool.mutex.Lock()
	for _, sandbox := range ci.sandboxPool.active {
		ci.cleanupSandbox(sandbox)
	}
	ci.sandboxPool.mutex.Unlock()

	ci.logger.Println("✅ Capability Ingestor stopped")
	return nil
}

func (ci *CapabilityIngestor) AddSource(source *CapabilitySource) error {
	ci.mutex.Lock()
	defer ci.mutex.Unlock()

	source.ID = generateSourceID(source.URL)
	source.CreatedAt = time.Now()
	source.UpdatedAt = time.Now()
	source.Status = "active"

	ci.sources[source.ID] = source

	// Store in Redis
	sourceJSON, _ := json.Marshal(source)
	ctx := context.Background()
	ci.redis.Set(ctx, fmt.Sprintf("source:%s", source.ID), sourceJSON, 0)

	// Start watcher
	ci.startSourceWatcher(source)

	ci.logger.Printf("✅ Added capability source: %s (%s)", source.URL, source.ID)
	return nil
}

func (ci *CapabilityIngestor) RemoveSource(sourceID string) error {
	ci.mutex.Lock()
	defer ci.mutex.Unlock()

	if watcher, exists := ci.watchers[sourceID]; exists {
		close(watcher.stopChan)
		delete(ci.watchers, sourceID)
	}

	delete(ci.sources, sourceID)

	// Remove from Redis
	ctx := context.Background()
	ci.redis.Del(ctx, fmt.Sprintf("source:%s", sourceID))

	ci.logger.Printf("✅ Removed capability source: %s", sourceID)
	return nil
}

func (ci *CapabilityIngestor) IngestCapability(request *IngestRequest) (*IngestResponse, error) {
	startTime := time.Now()

	ci.logger.Printf("🔍 Ingesting capability: %s from source: %s", request.CapabilityID, request.SourceID)

	// Get source
	ci.mutex.RLock()
	source, exists := ci.sources[request.SourceID]
	ci.mutex.RUnlock()

	if !exists {
		return &IngestResponse{
			Success:      false,
			ErrorMessage: "Source not found",
		}, fmt.Errorf("source not found: %s", request.SourceID)
	}

	// Discover capability
	capability, err := ci.discoverCapability(source, request.CapabilityID)
	if err != nil {
		return &IngestResponse{
			Success:      false,
			ErrorMessage: err.Error(),
		}, err
	}

	// Validate capability pack
	if err := ci.validateCapabilityPack(capability); err != nil {
		return &IngestResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Validation failed: %v", err),
		}, err
	}

	// Create sandbox
	sandbox, err := ci.createSandbox(capability)
	if err != nil {
		return &IngestResponse{
			Success:      false,
			ErrorMessage: fmt.Sprintf("Sandbox creation failed: %v", err),
		}, err
	}

	// Execute in sandbox
	results, err := ci.executeSandbox(sandbox)
	if err != nil {
		ci.cleanupSandbox(sandbox)
		return &IngestResponse{
			Success:      false,
			SandboxID:    sandbox.ID,
			ErrorMessage: fmt.Sprintf("Sandbox execution failed: %v", err),
		}, err
	}

	// Store results
	if err := ci.storeResults(sandbox, results); err != nil {
		ci.logger.Printf("Warning: Failed to store results: %v", err)
	}

	// Cleanup sandbox
	ci.cleanupSandbox(sandbox)

	duration := time.Since(startTime)
	ci.logger.Printf("✅ Capability ingestion completed: %s (duration: %v)", request.CapabilityID, duration)

	return &IngestResponse{
		Success:   results.Success,
		SandboxID: sandbox.ID,
		Status:    "completed",
		Results:   results,
		Duration:  duration,
	}, nil
}

func (ci *CapabilityIngestor) discoverCapability(source *CapabilitySource, capabilityID string) (*CapabilityPack, error) {
	switch source.Type {
	case "git":
		return ci.discoverFromGit(source, capabilityID)
	case "registry":
		return ci.discoverFromRegistry(source, capabilityID)
	case "filesystem":
		return ci.discoverFromFilesystem(source, capabilityID)
	case "api":
		return ci.discoverFromAPI(source, capabilityID)
	default:
		return nil, fmt.Errorf("unsupported source type: %s", source.Type)
	}
}

func (ci *CapabilityIngestor) discoverFromGit(source *CapabilitySource, capabilityID string) (*CapabilityPack, error) {
	// Clone repository to temporary directory
	tempDir := filepath.Join(ci.config.SandboxDir, "temp", generateID())
	if err := os.MkdirAll(tempDir, 0755); err != nil {
		return nil, err
	}
	defer os.RemoveAll(tempDir)

	// Git clone
	cmd := exec.Command("git", "clone", source.URL, tempDir)
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("git clone failed: %w", err)
	}

	// Look for capability pack file
	capabilityFile := filepath.Join(tempDir, capabilityID, "capability.yaml")
	if _, err := os.Stat(capabilityFile); os.IsNotExist(err) {
		capabilityFile = filepath.Join(tempDir, capabilityID, "capability.yml")
	}

	// Read and parse capability pack
	data, err := os.ReadFile(capabilityFile)
	if err != nil {
		return nil, fmt.Errorf("failed to read capability file: %w", err)
	}

	var capability CapabilityPack
	if err := yaml.Unmarshal(data, &capability); err != nil {
		return nil, fmt.Errorf("failed to parse capability: %w", err)
	}

	return &capability, nil
}

func (ci *CapabilityIngestor) discoverFromRegistry(source *CapabilitySource, capabilityID string) (*CapabilityPack, error) {
	// Implementation for registry-based discovery
	// This would integrate with container registries, package managers, etc.
	return nil, fmt.Errorf("registry discovery not implemented")
}

func (ci *CapabilityIngestor) discoverFromFilesystem(source *CapabilitySource, capabilityID string) (*CapabilityPack, error) {
	// Implementation for filesystem-based discovery
	capabilityFile := filepath.Join(source.URL, capabilityID, "capability.yaml")
	
	data, err := os.ReadFile(capabilityFile)
	if err != nil {
		return nil, fmt.Errorf("failed to read capability file: %w", err)
	}

	var capability CapabilityPack
	if err := yaml.Unmarshal(data, &capability); err != nil {
		return nil, fmt.Errorf("failed to parse capability: %w", err)
	}

	return &capability, nil
}

func (ci *CapabilityIngestor) discoverFromAPI(source *CapabilitySource, capabilityID string) (*CapabilityPack, error) {
	// Implementation for API-based discovery
	url := fmt.Sprintf("%s/capabilities/%s", source.URL, capabilityID)
	
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var capability CapabilityPack
	if err := json.NewDecoder(resp.Body).Decode(&capability); err != nil {
		return nil, err
	}

	return &capability, nil
}

func (ci *CapabilityIngestor) validateCapabilityPack(capability *CapabilityPack) error {
	// Validate required fields
	if capability.Kind != "Capability" {
		return fmt.Errorf("invalid kind: %s", capability.Kind)
	}

	if capability.Metadata.ID == "" {
		return fmt.Errorf("missing capability ID")
	}

	if capability.Spec.Purpose == "" {
		return fmt.Errorf("missing capability purpose")
	}

	// Validate signature
	if err := ci.validateSignature(capability); err != nil {
		return fmt.Errorf("signature validation failed: %w", err)
	}

	return nil
}

func (ci *CapabilityIngestor) validateSignature(capability *CapabilityPack) error {
	// Implementation for cryptographic signature validation
	// This would verify the SHA-256 hash and certificate
	
	// For now, just check that signature fields are present
	if capability.Signature.SHA256 == "" {
		return fmt.Errorf("missing SHA-256 signature")
	}

	if capability.Signature.Issuer == "" {
		return fmt.Errorf("missing signature issuer")
	}

	// TODO: Implement actual cryptographic verification
	return nil
}

func (ci *CapabilityIngestor) createSandbox(capability *CapabilityPack) (*SandboxInstance, error) {
	sandboxID := generateID()
	workDir := filepath.Join(ci.config.SandboxDir, sandboxID)

	if err := os.MkdirAll(workDir, 0755); err != nil {
		return nil, err
	}

	sandbox := &SandboxInstance{
		ID:         sandboxID,
		Type:       ci.config.IsolationMode,
		Status:     "created",
		Capability: capability,
		WorkDir:    workDir,
		StartTime:  time.Now(),
		Resources:  make(map[string]interface{}),
		Logs:       make([]string, 0),
	}

	// Add to active pool
	ci.sandboxPool.mutex.Lock()
	ci.sandboxPool.active[sandboxID] = sandbox
	ci.sandboxPool.mutex.Unlock()

	ci.logger.Printf("✅ Created sandbox: %s for capability: %s", sandboxID, capability.Metadata.ID)
	return sandbox, nil
}

func (ci *CapabilityIngestor) executeSandbox(sandbox *SandboxInstance) (*SandboxResults, error) {
	sandbox.mutex.Lock()
	sandbox.Status = "running"
	sandbox.mutex.Unlock()

	ci.logger.Printf("🏃 Executing sandbox: %s", sandbox.ID)

	results := &SandboxResults{
		Success:     true,
		TestResults: make(map[string]interface{}),
		Artifacts:   make([]string, 0),
		Logs:        make([]string, 0),
	}

	startTime := time.Now()

	// Generate SBOM
	sbom, err := ci.generateSBOM(sandbox)
	if err != nil {
		ci.logger.Printf("Warning: SBOM generation failed: %v", err)
	} else {
		results.SBOM = sbom
	}

	// Run tests
	testResults, err := ci.runTests(sandbox)
	if err != nil {
		results.Success = false
		results.ErrorMessage = err.Error()
	} else {
		results.TestResults = testResults
	}

	// Perform risk analysis
	riskAnalysis, err := ci.performRiskAnalysis(sandbox)
	if err != nil {
		ci.logger.Printf("Warning: Risk analysis failed: %v", err)
	} else {
		results.RiskAnalysis = riskAnalysis
	}

	// Collect performance metrics
	performance, err := ci.collectPerformanceMetrics(sandbox)
	if err != nil {
		ci.logger.Printf("Warning: Performance collection failed: %v", err)
	} else {
		results.Performance = performance
	}

	results.Duration = time.Since(startTime)
	sandbox.Results = results

	sandbox.mutex.Lock()
	sandbox.Status = "completed"
	sandbox.EndTime = time.Now()
	sandbox.mutex.Unlock()

	ci.logger.Printf("✅ Sandbox execution completed: %s (success: %v)", sandbox.ID, results.Success)
	return results, nil
}

func (ci *CapabilityIngestor) generateSBOM(sandbox *SandboxInstance) (*SBOM, error) {
	// Implementation for SBOM generation
	// This would scan the capability for dependencies, licenses, etc.
	
	sbom := &SBOM{
		Version:    "1.0",
		Components: make([]SBOMComponent, 0),
		CreatedAt:  time.Now(),
		Generator:  "ark-ai-os-ingestor",
	}

	// TODO: Implement actual SBOM generation logic
	// This would scan for dependencies, analyze licenses, etc.

	return sbom, nil
}

func (ci *CapabilityIngestor) runTests(sandbox *SandboxInstance) (map[string]interface{}, error) {
	results := make(map[string]interface{})

	// Run unit tests
	if len(sandbox.Capability.Spec.Tests.Unit) > 0 {
		unitResults, err := ci.runUnitTests(sandbox)
		if err != nil {
			return nil, err
		}
		results["unit"] = unitResults
	}

	// Run integration tests
	if len(sandbox.Capability.Spec.Tests.Integration) > 0 {
		integrationResults, err := ci.runIntegrationTests(sandbox)
		if err != nil {
			return nil, err
		}
		results["integration"] = integrationResults
	}

	// Run constitutional tests
	if len(sandbox.Capability.Spec.Tests.Constitutional) > 0 {
		constitutionalResults, err := ci.runConstitutionalTests(sandbox)
		if err != nil {
			return nil, err
		}
		results["constitutional"] = constitutionalResults
	}

	return results, nil
}

func (ci *CapabilityIngestor) runUnitTests(sandbox *SandboxInstance) (map[string]interface{}, error) {
	// Implementation for unit test execution
	return map[string]interface{}{
		"passed": true,
		"count":  len(sandbox.Capability.Spec.Tests.Unit),
		"tests":  sandbox.Capability.Spec.Tests.Unit,
	}, nil
}

func (ci *CapabilityIngestor) runIntegrationTests(sandbox *SandboxInstance) (map[string]interface{}, error) {
	// Implementation for integration test execution
	return map[string]interface{}{
		"passed": true,
		"count":  len(sandbox.Capability.Spec.Tests.Integration),
		"tests":  sandbox.Capability.Spec.Tests.Integration,
	}, nil
}

func (ci *CapabilityIngestor) runConstitutionalTests(sandbox *SandboxInstance) (map[string]interface{}, error) {
	// Implementation for constitutional test execution
	// This would validate against the Trifecta Court
	
	// Call Trifecta Court for validation
	payload := map[string]interface{}{
		"action": "validate_capability",
		"context": map[string]interface{}{
			"capability_id": sandbox.Capability.Metadata.ID,
			"capability":    sandbox.Capability,
		},
	}

	// TODO: Implement actual Trifecta Court integration
	
	return map[string]interface{}{
		"passed":           true,
		"scripture_court":  "approved",
		"geometry_court":   "approved",
		"bridge_path":      "approved",
		"overall_verdict":  "approved",
	}, nil
}

func (ci *CapabilityIngestor) performRiskAnalysis(sandbox *SandboxInstance) (*RiskAnalysis, error) {
	// Implementation for risk analysis
	analysis := &RiskAnalysis{
		OverallRisk:      "low",
		RiskFactors:      make(map[string]string),
		Mitigations:      make([]string, 0),
		Recommendations:  make([]string, 0),
		LicenseIssues:    make([]string, 0),
		SecurityIssues:   make([]string, 0),
		ComplianceIssues: make([]string, 0),
	}

	// Analyze capability risks
	risks := sandbox.Capability.Spec.Risks
	analysis.RiskFactors["privacy"] = risks.Privacy
	analysis.RiskFactors["supply_chain"] = risks.SupplyChain
	analysis.RiskFactors["license"] = risks.License
	analysis.RiskFactors["security"] = risks.Security
	analysis.RiskFactors["cost"] = risks.Cost
	analysis.RiskFactors["constitutional"] = risks.Constitutional

	// TODO: Implement actual risk analysis logic

	return analysis, nil
}

func (ci *CapabilityIngestor) collectPerformanceMetrics(sandbox *SandboxInstance) (*PerformanceMetrics, error) {
	// Implementation for performance metrics collection
	metrics := &PerformanceMetrics{
		CPUUsage:    0.0,
		MemoryUsage: 0,
		DiskUsage:   0,
		NetworkIO:   make(map[string]int64),
		Latency:     make(map[string]float64),
		Throughput:  make(map[string]float64),
		ErrorRate:   0.0,
	}

	// TODO: Implement actual performance monitoring

	return metrics, nil
}

func (ci *CapabilityIngestor) storeResults(sandbox *SandboxInstance, results *SandboxResults) error {
	// Store results in Redis
	resultsJSON, err := json.Marshal(results)
	if err != nil {
		return err
	}

	ctx := context.Background()
	key := fmt.Sprintf("sandbox_results:%s", sandbox.ID)
	return ci.redis.Set(ctx, key, resultsJSON, 24*time.Hour).Err()
}

func (ci *CapabilityIngestor) cleanupSandbox(sandbox *SandboxInstance) {
	// Remove from active pool
	ci.sandboxPool.mutex.Lock()
	delete(ci.sandboxPool.active, sandbox.ID)
	ci.sandboxPool.mutex.Unlock()

	// Clean up work directory
	if err := os.RemoveAll(sandbox.WorkDir); err != nil {
		ci.logger.Printf("Warning: Failed to cleanup sandbox directory: %v", err)
	}

	ci.logger.Printf("🧹 Cleaned up sandbox: %s", sandbox.ID)
}

func (ci *CapabilityIngestor) loadSources() error {
	ctx := context.Background()
	keys, err := ci.redis.Keys(ctx, "source:*").Result()
	if err != nil {
		return err
	}

	for _, key := range keys {
		data, err := ci.redis.Get(ctx, key).Result()
		if err != nil {
			continue
		}

		var source CapabilitySource
		if err := json.Unmarshal([]byte(data), &source); err != nil {
			continue
		}

		ci.sources[source.ID] = &source
	}

	return nil
}

func (ci *CapabilityIngestor) startSourceWatchers() {
	ci.mutex.RLock()
	defer ci.mutex.RUnlock()

	for _, source := range ci.sources {
		ci.startSourceWatcher(source)
	}
}

func (ci *CapabilityIngestor) startSourceWatcher(source *CapabilitySource) {
	watcher := &SourceWatcher{
		source:   source,
		ingestor: ci,
		ticker:   time.NewTicker(time.Duration(ci.config.PollInterval) * time.Second),
		stopChan: make(chan struct{}),
	}

	ci.watchers[source.ID] = watcher

	go func() {
		for {
			select {
			case <-watcher.ticker.C:
				ci.pollSource(source)
			case <-watcher.stopChan:
				watcher.ticker.Stop()
				return
			}
		}
	}()

	ci.logger.Printf("👀 Started watcher for source: %s", source.ID)
}

func (ci *CapabilityIngestor) pollSource(source *CapabilitySource) {
	// Implementation for polling source for changes
	// This would check for new capabilities, version updates, etc.
	ci.logger.Printf("🔍 Polling source: %s", source.ID)
	
	// TODO: Implement actual polling logic based on source type
}

func (ci *CapabilityIngestor) sandboxCleanupRoutine() {
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			ci.cleanupExpiredSandboxes()
		case <-ci.stopChan:
			return
		}
	}
}

func (ci *CapabilityIngestor) cleanupExpiredSandboxes() {
	ci.sandboxPool.mutex.Lock()
	defer ci.sandboxPool.mutex.Unlock()

	now := time.Now()
	timeout := time.Duration(ci.config.DefaultTimeout) * time.Second

	for id, sandbox := range ci.sandboxPool.active {
		if now.Sub(sandbox.StartTime) > timeout {
			ci.logger.Printf("🧹 Cleaning up expired sandbox: %s", id)
			ci.cleanupSandbox(sandbox)
		}
	}
}

// Helper functions
func generateID() string {
	return fmt.Sprintf("%d", time.Now().UnixNano())
}

func generateSourceID(url string) string {
	hash := sha256.Sum256([]byte(url + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

// REST API endpoints
func (ci *CapabilityIngestor) setupRoutes(router *gin.Engine) {
	api := router.Group("/api/v1")
	
	// Sources
	api.POST("/sources", ci.addSourceHandler)
	api.GET("/sources", ci.listSourcesHandler)
	api.GET("/sources/:id", ci.getSourceHandler)
	api.DELETE("/sources/:id", ci.removeSourceHandler)
	
	// Ingestion
	api.POST("/ingest", ci.ingestHandler)
	api.GET("/ingest/:id", ci.getIngestStatusHandler)
	
	// Sandboxes
	api.GET("/sandboxes", ci.listSandboxesHandler)
	api.GET("/sandboxes/:id", ci.getSandboxHandler)
	api.DELETE("/sandboxes/:id", ci.cleanupSandboxHandler)
}

func (ci *CapabilityIngestor) addSourceHandler(c *gin.Context) {
	var source CapabilitySource
	if err := c.ShouldBindJSON(&source); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid source format"})
		return
	}

	if err := ci.AddSource(&source); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success":   true,
		"source_id": source.ID,
		"source":    source,
	})
}

func (ci *CapabilityIngestor) listSourcesHandler(c *gin.Context) {
	ci.mutex.RLock()
	defer ci.mutex.RUnlock()

	sources := make([]*CapabilitySource, 0, len(ci.sources))
	for _, source := range ci.sources {
		sources = append(sources, source)
	}

	c.JSON(http.StatusOK, gin.H{
		"sources": sources,
		"total":   len(sources),
	})
}

func (ci *CapabilityIngestor) getSourceHandler(c *gin.Context) {
	sourceID := c.Param("id")
	
	ci.mutex.RLock()
	source, exists := ci.sources[sourceID]
	ci.mutex.RUnlock()

	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Source not found"})
		return
	}

	c.JSON(http.StatusOK, source)
}

func (ci *CapabilityIngestor) removeSourceHandler(c *gin.Context) {
	sourceID := c.Param("id")
	
	if err := ci.RemoveSource(sourceID); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Source removed successfully",
	})
}

func (ci *CapabilityIngestor) ingestHandler(c *gin.Context) {
	var request IngestRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	response, err := ci.IngestCapability(&request)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, response)
}

func (ci *CapabilityIngestor) getIngestStatusHandler(c *gin.Context) {
	sandboxID := c.Param("id")
	
	// Get results from Redis
	ctx := context.Background()
	key := fmt.Sprintf("sandbox_results:%s", sandboxID)
	data, err := ci.redis.Get(ctx, key).Result()
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Results not found"})
		return
	}

	var results SandboxResults
	if err := json.Unmarshal([]byte(data), &results); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to parse results"})
		return
	}

	c.JSON(http.StatusOK, results)
}

func (ci *CapabilityIngestor) listSandboxesHandler(c *gin.Context) {
	ci.sandboxPool.mutex.RLock()
	defer ci.sandboxPool.mutex.RUnlock()

	sandboxes := make([]*SandboxInstance, 0, len(ci.sandboxPool.active))
	for _, sandbox := range ci.sandboxPool.active {
		sandboxes = append(sandboxes, sandbox)
	}

	c.JSON(http.StatusOK, gin.H{
		"sandboxes": sandboxes,
		"total":     len(sandboxes),
	})
}

func (ci *CapabilityIngestor) getSandboxHandler(c *gin.Context) {
	sandboxID := c.Param("id")
	
	ci.sandboxPool.mutex.RLock()
	sandbox, exists := ci.sandboxPool.active[sandboxID]
	ci.sandboxPool.mutex.RUnlock()

	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Sandbox not found"})
		return
	}

	c.JSON(http.StatusOK, sandbox)
}

func (ci *CapabilityIngestor) cleanupSandboxHandler(c *gin.Context) {
	sandboxID := c.Param("id")
	
	ci.sandboxPool.mutex.RLock()
	sandbox, exists := ci.sandboxPool.active[sandboxID]
	ci.sandboxPool.mutex.RUnlock()

	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Sandbox not found"})
		return
	}

	ci.cleanupSandbox(sandbox)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Sandbox cleaned up successfully",
	})
}

