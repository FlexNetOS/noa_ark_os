package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// CoordinatorCluster manages the control plane for capability promotion
type CoordinatorCluster struct {
	logger           *logrus.Logger
	redis            *redis.Client
	db               *sql.DB
	config           *CoordinatorConfig
	capabilityRegistry *CapabilityRegistry
	promotionController *PromotionController
	scorecardSystem    *ScorecardSystem
	metrics          *CoordinatorMetrics
	mutex            sync.RWMutex
	shutdownChan     chan struct{}
}

// CoordinatorConfig holds configuration for the coordinator cluster
type CoordinatorConfig struct {
	ServerPort           string                 `yaml:"server_port"`
	RedisURL             string                 `yaml:"redis_url"`
	DatabaseURL          string                 `yaml:"database_url"`
	SandboxClusterURL    string                 `yaml:"sandbox_cluster_url"`
	DeployedAppClusterURL string                `yaml:"deployed_app_cluster_url"`
	TrifectaCourtURL     string                 `yaml:"trifecta_court_url"`
	PromotionPolicies    *PromotionPolicies     `yaml:"promotion_policies"`
	ScorecardConfig      *ScorecardConfig       `yaml:"scorecard_config"`
	ConstitutionalConfig *ConstitutionalConfig  `yaml:"constitutional_config"`
}

// PromotionPolicies defines promotion decision policies
type PromotionPolicies struct {
	AutoPromotionEnabled bool                   `yaml:"auto_promotion_enabled"`
	RiskTiers           map[string]*RiskTier   `yaml:"risk_tiers"`
	DefaultRiskTier     string                 `yaml:"default_risk_tier"`
	PromotionGates      []*PromotionGate       `yaml:"promotion_gates"`
	RolloutStrategies   map[string]*RolloutStrategy `yaml:"rollout_strategies"`
	BudgetConstraints   *BudgetConstraints     `yaml:"budget_constraints"`
}

// RiskTier defines risk-based promotion policies
type RiskTier struct {
	Name                string        `yaml:"name"`
	RequiredScore       float64       `yaml:"required_score"`
	TestingDuration     time.Duration `yaml:"testing_duration"`
	ManualApproval      bool          `yaml:"manual_approval"`
	CanaryPercent       float64       `yaml:"canary_percent"`
	RolloutDuration     time.Duration `yaml:"rollout_duration"`
	MonitoringDuration  time.Duration `yaml:"monitoring_duration"`
	AutoRollback        bool          `yaml:"auto_rollback"`
	ConstitutionalCheck bool          `yaml:"constitutional_check"`
}

// PromotionGate defines gates that must pass for promotion
type PromotionGate struct {
	ID          string                 `yaml:"id"`
	Name        string                 `yaml:"name"`
	Type        string                 `yaml:"type"`
	Required    bool                   `yaml:"required"`
	Criteria    map[string]interface{} `yaml:"criteria"`
	Timeout     time.Duration          `yaml:"timeout"`
	RetryPolicy *RetryPolicy           `yaml:"retry_policy"`
}

// RolloutStrategy defines how capabilities are rolled out
type RolloutStrategy struct {
	Name            string                 `yaml:"name"`
	Type            string                 `yaml:"type"`
	Phases          []*RolloutPhase        `yaml:"phases"`
	HealthChecks    *HealthCheckConfig     `yaml:"health_checks"`
	SLOThresholds   *SLOThresholds         `yaml:"slo_thresholds"`
	RollbackTriggers []*RollbackTrigger    `yaml:"rollback_triggers"`
}

// RolloutPhase defines a phase in the rollout strategy
type RolloutPhase struct {
	Name            string        `yaml:"name"`
	TrafficPercent  float64       `yaml:"traffic_percent"`
	Duration        time.Duration `yaml:"duration"`
	SuccessCriteria map[string]interface{} `yaml:"success_criteria"`
	FailureCriteria map[string]interface{} `yaml:"failure_criteria"`
}

// BudgetConstraints defines budget limits for promotions
type BudgetConstraints struct {
	MaxCostPerPromotion   float64 `yaml:"max_cost_per_promotion"`
	MaxCostPerHour        float64 `yaml:"max_cost_per_hour"`
	MaxCostPerDay         float64 `yaml:"max_cost_per_day"`
	MaxCostPerMonth       float64 `yaml:"max_cost_per_month"`
	CostCenters          map[string]float64 `yaml:"cost_centers"`
	BudgetAlerts         bool    `yaml:"budget_alerts"`
}

// ScorecardConfig defines scorecard evaluation configuration
type ScorecardConfig struct {
	EvaluationInterval  time.Duration          `yaml:"evaluation_interval"`
	Dimensions          []*ScorecardDimension  `yaml:"dimensions"`
	Weights             map[string]float64     `yaml:"weights"`
	Thresholds          map[string]float64     `yaml:"thresholds"`
	BenchmarkSuites     []*BenchmarkSuite      `yaml:"benchmark_suites"`
	ExternalIntegrations []*ExternalIntegration `yaml:"external_integrations"`
}

// ScorecardDimension defines a dimension for capability evaluation
type ScorecardDimension struct {
	Name        string                 `yaml:"name"`
	Weight      float64                `yaml:"weight"`
	Metrics     []string               `yaml:"metrics"`
	Aggregation string                 `yaml:"aggregation"`
	Thresholds  map[string]float64     `yaml:"thresholds"`
	Enabled     bool                   `yaml:"enabled"`
}

// BenchmarkSuite defines a suite of benchmarks
type BenchmarkSuite struct {
	Name        string                 `yaml:"name"`
	Type        string                 `yaml:"type"`
	Tests       []string               `yaml:"tests"`
	Timeout     time.Duration          `yaml:"timeout"`
	Parallel    bool                   `yaml:"parallel"`
	Enabled     bool                   `yaml:"enabled"`
}

// ExternalIntegration defines integration with external systems
type ExternalIntegration struct {
	Name        string                 `yaml:"name"`
	Type        string                 `yaml:"type"`
	URL         string                 `yaml:"url"`
	Credentials map[string]string      `yaml:"credentials"`
	Timeout     time.Duration          `yaml:"timeout"`
	Enabled     bool                   `yaml:"enabled"`
}

// ConstitutionalConfig defines constitutional governance settings
type ConstitutionalConfig struct {
	TrifectaCourtURL    string  `yaml:"trifecta_court_url"`
	ValidationRequired  bool    `yaml:"validation_required"`
	EthicsCheckEnabled  bool    `yaml:"ethics_check_enabled"`
	GovernanceEnabled   bool    `yaml:"governance_enabled"`
	AuditTrailRequired  bool    `yaml:"audit_trail_required"`
	ComplianceThreshold float64 `yaml:"compliance_threshold"`
}

// Supporting structures

type RetryPolicy struct {
	MaxAttempts int           `yaml:"max_attempts"`
	Delay       time.Duration `yaml:"delay"`
	Backoff     string        `yaml:"backoff"`
}

type HealthCheckConfig struct {
	HTTPPath            string        `yaml:"http_path"`
	HTTPPort            int           `yaml:"http_port"`
	InitialDelaySeconds int           `yaml:"initial_delay_seconds"`
	PeriodSeconds       int           `yaml:"period_seconds"`
	TimeoutSeconds      int           `yaml:"timeout_seconds"`
	FailureThreshold    int           `yaml:"failure_threshold"`
	SuccessThreshold    int           `yaml:"success_threshold"`
}

type SLOThresholds struct {
	MaxLatencyMs        float64 `yaml:"max_latency_ms"`
	MinAvailability     float64 `yaml:"min_availability"`
	MinSuccessRate      float64 `yaml:"min_success_rate"`
	MaxErrorRate        float64 `yaml:"max_error_rate"`
	MaxMemoryUsage      float64 `yaml:"max_memory_usage"`
	MaxCPUUsage         float64 `yaml:"max_cpu_usage"`
	MinThroughputRPS    float64 `yaml:"min_throughput_rps"`
}

type RollbackTrigger struct {
	ID          string                 `yaml:"id"`
	Type        string                 `yaml:"type"`
	Condition   string                 `yaml:"condition"`
	Threshold   float64                `yaml:"threshold"`
	Duration    time.Duration          `yaml:"duration"`
	Enabled     bool                   `yaml:"enabled"`
	Priority    int                    `yaml:"priority"`
	Actions     []string               `yaml:"actions"`
}

// Core data structures

type RegisteredCapability struct {
	ID                  string                 `json:"id"`
	Name                string                 `json:"name"`
	Version             string                 `json:"version"`
	Type                string                 `json:"type"`
	Description         string                 `json:"description"`
	ContentHash         string                 `json:"content_hash"`
	Signature           string                 `json:"signature"`
	SourceURL           string                 `json:"source_url"`
	Dependencies        []string               `json:"dependencies"`
	Dependents          []string               `json:"dependents"`
	CompatibilityMatrix map[string]interface{} `json:"compatibility_matrix"`
	Metadata            map[string]interface{} `json:"metadata"`
	Status              string                 `json:"status"`
	RiskTier            string                 `json:"risk_tier"`
	Scorecard           *CapabilityScorecard   `json:"scorecard"`
	PromotionHistory    []*PromotionRecord     `json:"promotion_history"`
	ConstitutionalCheck *ConstitutionalCheck   `json:"constitutional_check"`
	ProvenanceRecord    *ProvenanceRecord      `json:"provenance_record"`
	RegisteredAt        time.Time              `json:"registered_at"`
	UpdatedAt           time.Time              `json:"updated_at"`
	ExpiresAt           time.Time              `json:"expires_at,omitempty"`
}

type CapabilityScorecard struct {
	ID                  string                 `json:"id"`
	CapabilityID        string                 `json:"capability_id"`
	OverallScore        float64                `json:"overall_score"`
	Grade               string                 `json:"grade"`
	DimensionScores     map[string]float64     `json:"dimension_scores"`
	SubScores           map[string]float64     `json:"sub_scores"`
	Trends              map[string]*TrendData  `json:"trends"`
	Benchmarks          map[string]*BenchmarkResult `json:"benchmarks"`
	Recommendations     []*Recommendation      `json:"recommendations"`
	Alerts              []*ScorecardAlert      `json:"alerts"`
	LastEvaluated       time.Time              `json:"last_evaluated"`
	NextEvaluation      time.Time              `json:"next_evaluation"`
	EvaluationHistory   []*EvaluationRecord    `json:"evaluation_history"`
}

type PromotionRecord struct {
	ID              string                 `json:"id"`
	CapabilityID    string                 `json:"capability_id"`
	FromStage       string                 `json:"from_stage"`
	ToStage         string                 `json:"to_stage"`
	Status          string                 `json:"status"`
	Strategy        string                 `json:"strategy"`
	RiskTier        string                 `json:"risk_tier"`
	Score           float64                `json:"score"`
	Gates           []*GateResult          `json:"gates"`
	Approvals       []*ApprovalRecord      `json:"approvals"`
	RolloutProgress *RolloutProgress       `json:"rollout_progress"`
	ConstitutionalCheck *ConstitutionalCheck `json:"constitutional_check"`
	StartedAt       time.Time              `json:"started_at"`
	CompletedAt     time.Time              `json:"completed_at,omitempty"`
	Duration        time.Duration          `json:"duration"`
	Metadata        map[string]interface{} `json:"metadata"`
}

type ConstitutionalCheck struct {
	ID              string                 `json:"id"`
	Status          string                 `json:"status"`
	ScriptureScore  float64                `json:"scripture_score"`
	GeometryScore   float64                `json:"geometry_score"`
	BridgePathScore float64                `json:"bridge_path_score"`
	OverallScore    float64                `json:"overall_score"`
	Verdict         string                 `json:"verdict"`
	Reasoning       string                 `json:"reasoning"`
	Conditions      []string               `json:"conditions"`
	Violations      []string               `json:"violations"`
	AuditTrail      []*ConstitutionalAudit `json:"audit_trail"`
	CheckedAt       time.Time              `json:"checked_at"`
	ValidUntil      time.Time              `json:"valid_until"`
}

type ConstitutionalAudit struct {
	Timestamp   time.Time              `json:"timestamp"`
	Action      string                 `json:"action"`
	Decision    string                 `json:"decision"`
	Reasoning   string                 `json:"reasoning"`
	Evidence    map[string]interface{} `json:"evidence"`
	Auditor     string                 `json:"auditor"`
	Hash        string                 `json:"hash"`
}

type ProvenanceRecord struct {
	ID              string                 `json:"id"`
	CapabilityID    string                 `json:"capability_id"`
	SourceChain     []*ProvenanceLink      `json:"source_chain"`
	BuildInfo       *BuildInfo             `json:"build_info"`
	TestResults     []*TestResult          `json:"test_results"`
	SecurityScans   []*SecurityScan        `json:"security_scans"`
	Signatures      []*DigitalSignature    `json:"signatures"`
	Attestations    []*Attestation         `json:"attestations"`
	CreatedAt       time.Time              `json:"created_at"`
	UpdatedAt       time.Time              `json:"updated_at"`
}

type ProvenanceLink struct {
	Type        string                 `json:"type"`
	Source      string                 `json:"source"`
	Hash        string                 `json:"hash"`
	Timestamp   time.Time              `json:"timestamp"`
	Metadata    map[string]interface{} `json:"metadata"`
}

type BuildInfo struct {
	BuildID         string                 `json:"build_id"`
	BuildSystem     string                 `json:"build_system"`
	BuildTime       time.Time              `json:"build_time"`
	BuildEnvironment map[string]string     `json:"build_environment"`
	Dependencies    []*DependencyInfo      `json:"dependencies"`
	Artifacts       []*ArtifactInfo        `json:"artifacts"`
}

type TestResult struct {
	TestSuite   string                 `json:"test_suite"`
	TestType    string                 `json:"test_type"`
	Status      string                 `json:"status"`
	Score       float64                `json:"score"`
	Results     map[string]interface{} `json:"results"`
	Duration    time.Duration          `json:"duration"`
	Timestamp   time.Time              `json:"timestamp"`
}

type SecurityScan struct {
	ScannerType     string                 `json:"scanner_type"`
	ScannerVersion  string                 `json:"scanner_version"`
	Status          string                 `json:"status"`
	Vulnerabilities []*Vulnerability       `json:"vulnerabilities"`
	Score           float64                `json:"score"`
	Timestamp       time.Time              `json:"timestamp"`
}

type Vulnerability struct {
	ID          string  `json:"id"`
	Type        string  `json:"type"`
	Severity    string  `json:"severity"`
	Score       float64 `json:"score"`
	Description string  `json:"description"`
	Fix         string  `json:"fix,omitempty"`
}

type DigitalSignature struct {
	Algorithm   string    `json:"algorithm"`
	Signature   string    `json:"signature"`
	PublicKey   string    `json:"public_key"`
	Signer      string    `json:"signer"`
	Timestamp   time.Time `json:"timestamp"`
}

type Attestation struct {
	Type        string                 `json:"type"`
	Attester    string                 `json:"attester"`
	Statement   string                 `json:"statement"`
	Evidence    map[string]interface{} `json:"evidence"`
	Signature   string                 `json:"signature"`
	Timestamp   time.Time              `json:"timestamp"`
}

type DependencyInfo struct {
	Name        string `json:"name"`
	Version     string `json:"version"`
	Type        string `json:"type"`
	Hash        string `json:"hash"`
	License     string `json:"license"`
}

type ArtifactInfo struct {
	Name        string `json:"name"`
	Type        string `json:"type"`
	Hash        string `json:"hash"`
	Size        int64  `json:"size"`
	Path        string `json:"path"`
}

type TrendData struct {
	Values      []float64 `json:"values"`
	Timestamps  []time.Time `json:"timestamps"`
	Trend       string    `json:"trend"`
	Slope       float64   `json:"slope"`
	Correlation float64   `json:"correlation"`
}

type BenchmarkResult struct {
	BenchmarkID string                 `json:"benchmark_id"`
	Score       float64                `json:"score"`
	Results     map[string]interface{} `json:"results"`
	Baseline    float64                `json:"baseline"`
	Percentile  float64                `json:"percentile"`
	Timestamp   time.Time              `json:"timestamp"`
}

type Recommendation struct {
	ID          string                 `json:"id"`
	Type        string                 `json:"type"`
	Priority    string                 `json:"priority"`
	Title       string                 `json:"title"`
	Description string                 `json:"description"`
	Actions     []string               `json:"actions"`
	Impact      string                 `json:"impact"`
	Effort      string                 `json:"effort"`
	ROI         float64                `json:"roi"`
	Metadata    map[string]interface{} `json:"metadata"`
}

type ScorecardAlert struct {
	ID          string                 `json:"id"`
	Type        string                 `json:"type"`
	Severity    string                 `json:"severity"`
	Message     string                 `json:"message"`
	Dimension   string                 `json:"dimension"`
	Threshold   float64                `json:"threshold"`
	Current     float64                `json:"current"`
	Actions     []string               `json:"actions"`
	Timestamp   time.Time              `json:"timestamp"`
}

type EvaluationRecord struct {
	ID              string                 `json:"id"`
	OverallScore    float64                `json:"overall_score"`
	Grade           string                 `json:"grade"`
	DimensionScores map[string]float64     `json:"dimension_scores"`
	Changes         map[string]float64     `json:"changes"`
	Timestamp       time.Time              `json:"timestamp"`
	Duration        time.Duration          `json:"duration"`
}

type GateResult struct {
	GateID      string                 `json:"gate_id"`
	Status      string                 `json:"status"`
	Score       float64                `json:"score"`
	Results     map[string]interface{} `json:"results"`
	Duration    time.Duration          `json:"duration"`
	Timestamp   time.Time              `json:"timestamp"`
	Error       string                 `json:"error,omitempty"`
}

type ApprovalRecord struct {
	ID          string                 `json:"id"`
	Type        string                 `json:"type"`
	Approver    string                 `json:"approver"`
	Status      string                 `json:"status"`
	Comments    string                 `json:"comments"`
	Conditions  []string               `json:"conditions"`
	Timestamp   time.Time              `json:"timestamp"`
}

type RolloutProgress struct {
	CurrentPhase    int                    `json:"current_phase"`
	TotalPhases     int                    `json:"total_phases"`
	Progress        float64                `json:"progress"`
	Status          string                 `json:"status"`
	TrafficPercent  float64                `json:"traffic_percent"`
	Metrics         map[string]interface{} `json:"metrics"`
	Issues          []string               `json:"issues"`
	StartedAt       time.Time              `json:"started_at"`
	EstimatedCompletion time.Time          `json:"estimated_completion"`
}

// Component interfaces

type CapabilityRegistry struct {
	cluster      *CoordinatorCluster
	capabilities map[string]*RegisteredCapability
	dependencies map[string][]string
	dependents   map[string][]string
	mutex        sync.RWMutex
}

type PromotionController struct {
	cluster         *CoordinatorCluster
	activePromotions map[string]*PromotionRecord
	promotionQueue  []*PromotionRequest
	policies        *PromotionPolicies
	mutex           sync.RWMutex
}

type PromotionRequest struct {
	ID           string                 `json:"id"`
	CapabilityID string                 `json:"capability_id"`
	FromStage    string                 `json:"from_stage"`
	ToStage      string                 `json:"to_stage"`
	Priority     int                    `json:"priority"`
	Requester    string                 `json:"requester"`
	Reason       string                 `json:"reason"`
	Metadata     map[string]interface{} `json:"metadata"`
	RequestedAt  time.Time              `json:"requested_at"`
}

type ScorecardSystem struct {
	cluster     *CoordinatorCluster
	scorecards  map[string]*CapabilityScorecard
	evaluators  map[string]*ScorecardEvaluator
	config      *ScorecardConfig
	mutex       sync.RWMutex
}

type ScorecardEvaluator struct {
	ID          string                 `json:"id"`
	Type        string                 `json:"type"`
	Enabled     bool                   `json:"enabled"`
	Config      map[string]interface{} `json:"config"`
	LastRun     time.Time              `json:"last_run"`
	NextRun     time.Time              `json:"next_run"`
}

// CoordinatorMetrics tracks coordinator system metrics
type CoordinatorMetrics struct {
	CapabilitiesRegistered  prometheus.Counter
	PromotionsTotal         prometheus.Counter
	PromotionsSuccessful    prometheus.Counter
	PromotionsFailed        prometheus.Counter
	ScorecardEvaluations    prometheus.Counter
	ConstitutionalChecks    prometheus.Counter
	ActivePromotions        prometheus.Gauge
	RegistrySize            prometheus.Gauge
	AveragePromotionTime    prometheus.Histogram
	ScorecardScores         prometheus.Histogram
}

func NewCoordinatorCluster(config *CoordinatorConfig) (*CoordinatorCluster, error) {
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
	metrics := &CoordinatorMetrics{
		CapabilitiesRegistered: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "capabilities_registered_total",
			Help: "Total number of capabilities registered",
		}),
		PromotionsTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "promotions_total",
			Help: "Total number of promotions attempted",
		}),
		PromotionsSuccessful: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "promotions_successful_total",
			Help: "Total number of successful promotions",
		}),
		PromotionsFailed: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "promotions_failed_total",
			Help: "Total number of failed promotions",
		}),
		ScorecardEvaluations: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "scorecard_evaluations_total",
			Help: "Total number of scorecard evaluations",
		}),
		ConstitutionalChecks: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "constitutional_checks_total",
			Help: "Total number of constitutional checks",
		}),
		ActivePromotions: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "active_promotions",
			Help: "Number of active promotions",
		}),
		RegistrySize: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "registry_size",
			Help: "Number of capabilities in registry",
		}),
		AveragePromotionTime: prometheus.NewHistogram(prometheus.HistogramOpts{
			Name:    "promotion_duration_seconds",
			Help:    "Duration of promotions",
			Buckets: prometheus.ExponentialBuckets(60, 2, 10),
		}),
		ScorecardScores: prometheus.NewHistogram(prometheus.HistogramOpts{
			Name:    "scorecard_scores",
			Help:    "Distribution of scorecard scores",
			Buckets: prometheus.LinearBuckets(0, 0.1, 11),
		}),
	}

	// Register metrics
	prometheus.MustRegister(
		metrics.CapabilitiesRegistered,
		metrics.PromotionsTotal,
		metrics.PromotionsSuccessful,
		metrics.PromotionsFailed,
		metrics.ScorecardEvaluations,
		metrics.ConstitutionalChecks,
		metrics.ActivePromotions,
		metrics.RegistrySize,
		metrics.AveragePromotionTime,
		metrics.ScorecardScores,
	)

	cluster := &CoordinatorCluster{
		logger:       logger,
		redis:        redisClient,
		db:           db,
		config:       config,
		metrics:      metrics,
		shutdownChan: make(chan struct{}),
	}

	// Initialize components
	cluster.capabilityRegistry = &CapabilityRegistry{
		cluster:      cluster,
		capabilities: make(map[string]*RegisteredCapability),
		dependencies: make(map[string][]string),
		dependents:   make(map[string][]string),
	}

	cluster.promotionController = &PromotionController{
		cluster:          cluster,
		activePromotions: make(map[string]*PromotionRecord),
		promotionQueue:   []*PromotionRequest{},
		policies:         config.PromotionPolicies,
	}

	cluster.scorecardSystem = &ScorecardSystem{
		cluster:    cluster,
		scorecards: make(map[string]*CapabilityScorecard),
		evaluators: make(map[string]*ScorecardEvaluator),
		config:     config.ScorecardConfig,
	}

	// Initialize database schema
	if err := cluster.initializeSchema(); err != nil {
		return nil, fmt.Errorf("failed to initialize schema: %w", err)
	}

	// Load existing data
	if err := cluster.loadData(); err != nil {
		logger.Warn("Failed to load existing data:", err)
	}

	return cluster, nil
}

func (cc *CoordinatorCluster) Start() error {
	cc.logger.Info("🚀 Starting Coordinator Cluster")

	// Start background routines
	go cc.promotionProcessingRoutine()
	go cc.scorecardEvaluationRoutine()
	go cc.constitutionalValidationRoutine()
	go cc.registryMaintenanceRoutine()
	go cc.metricsRoutine()

	cc.logger.Info("✅ Coordinator Cluster started successfully")
	return nil
}

func (cc *CoordinatorCluster) Stop() error {
	cc.logger.Info("🛑 Stopping Coordinator Cluster")

	// Signal shutdown
	close(cc.shutdownChan)

	// Close database connection
	if err := cc.db.Close(); err != nil {
		cc.logger.Error("Failed to close database:", err)
	}

	// Close Redis connection
	if err := cc.redis.Close(); err != nil {
		cc.logger.Error("Failed to close Redis:", err)
	}

	cc.logger.Info("✅ Coordinator Cluster stopped")
	return nil
}

func (cc *CoordinatorCluster) RegisterCapability(capability *RegisteredCapability) error {
	cc.logger.Infof("📝 Registering capability: %s:%s", capability.Name, capability.Version)

	// Validate constitutional compliance
	constitutionalCheck, err := cc.validateConstitutionalCompliance(capability)
	if err != nil {
		return fmt.Errorf("constitutional validation failed: %w", err)
	}

	if constitutionalCheck.Verdict != "APPROVED" {
		return fmt.Errorf("capability rejected by constitutional governance: %s", constitutionalCheck.Reasoning)
	}

	capability.ConstitutionalCheck = constitutionalCheck
	capability.RegisteredAt = time.Now()
	capability.UpdatedAt = time.Now()
	capability.Status = "registered"

	// Store capability
	cc.capabilityRegistry.mutex.Lock()
	cc.capabilityRegistry.capabilities[capability.ID] = capability
	cc.capabilityRegistry.mutex.Unlock()

	// Update dependencies
	cc.updateDependencyGraph(capability)

	// Initialize scorecard
	scorecard := cc.initializeScorecard(capability)
	cc.scorecardSystem.mutex.Lock()
	cc.scorecardSystem.scorecards[capability.ID] = scorecard
	cc.scorecardSystem.mutex.Unlock()

	// Update metrics
	cc.metrics.CapabilitiesRegistered.Inc()
	cc.metrics.RegistrySize.Inc()

	cc.logger.Infof("✅ Capability registered: %s", capability.ID)
	return nil
}

func (cc *CoordinatorCluster) RequestPromotion(request *PromotionRequest) (*PromotionRecord, error) {
	cc.logger.Infof("🚀 Promotion requested: %s from %s to %s", request.CapabilityID, request.FromStage, request.ToStage)

	// Validate capability exists
	capability, err := cc.GetCapability(request.CapabilityID)
	if err != nil {
		return nil, fmt.Errorf("capability not found: %w", err)
	}

	// Check if promotion is allowed
	if err := cc.validatePromotionRequest(request, capability); err != nil {
		return nil, fmt.Errorf("promotion validation failed: %w", err)
	}

	// Create promotion record
	promotion := &PromotionRecord{
		ID:           fmt.Sprintf("promo-%s-%d", request.CapabilityID, time.Now().Unix()),
		CapabilityID: request.CapabilityID,
		FromStage:    request.FromStage,
		ToStage:      request.ToStage,
		Status:       "pending",
		Strategy:     cc.determinePromotionStrategy(capability),
		RiskTier:     capability.RiskTier,
		Score:        capability.Scorecard.OverallScore,
		Gates:        []*GateResult{},
		Approvals:    []*ApprovalRecord{},
		StartedAt:    time.Now(),
		Metadata:     request.Metadata,
	}

	// Store promotion
	cc.promotionController.mutex.Lock()
	cc.promotionController.activePromotions[promotion.ID] = promotion
	cc.promotionController.mutex.Unlock()

	// Start promotion process
	go cc.processPromotion(promotion)

	// Update metrics
	cc.metrics.PromotionsTotal.Inc()
	cc.metrics.ActivePromotions.Inc()

	cc.logger.Infof("✅ Promotion initiated: %s", promotion.ID)
	return promotion, nil
}

func (cc *CoordinatorCluster) GetCapability(capabilityID string) (*RegisteredCapability, error) {
	cc.capabilityRegistry.mutex.RLock()
	capability, exists := cc.capabilityRegistry.capabilities[capabilityID]
	cc.capabilityRegistry.mutex.RUnlock()

	if !exists {
		return nil, fmt.Errorf("capability not found: %s", capabilityID)
	}

	return capability, nil
}

func (cc *CoordinatorCluster) GetPromotion(promotionID string) (*PromotionRecord, error) {
	cc.promotionController.mutex.RLock()
	promotion, exists := cc.promotionController.activePromotions[promotionID]
	cc.promotionController.mutex.RUnlock()

	if !exists {
		return nil, fmt.Errorf("promotion not found: %s", promotionID)
	}

	return promotion, nil
}

func (cc *CoordinatorCluster) GetScorecard(capabilityID string) (*CapabilityScorecard, error) {
	cc.scorecardSystem.mutex.RLock()
	scorecard, exists := cc.scorecardSystem.scorecards[capabilityID]
	cc.scorecardSystem.mutex.RUnlock()

	if !exists {
		return nil, fmt.Errorf("scorecard not found: %s", capabilityID)
	}

	return scorecard, nil
}

func (cc *CoordinatorCluster) GetAllCapabilities() ([]*RegisteredCapability, error) {
	cc.capabilityRegistry.mutex.RLock()
	defer cc.capabilityRegistry.mutex.RUnlock()

	capabilities := make([]*RegisteredCapability, 0, len(cc.capabilityRegistry.capabilities))
	for _, capability := range cc.capabilityRegistry.capabilities {
		capabilities = append(capabilities, capability)
	}

	return capabilities, nil
}

func (cc *CoordinatorCluster) GetSystemStats() map[string]interface{} {
	cc.mutex.RLock()
	defer cc.mutex.RUnlock()

	stats := map[string]interface{}{
		"total_capabilities":    len(cc.capabilityRegistry.capabilities),
		"active_promotions":     len(cc.promotionController.activePromotions),
		"promotion_queue_size":  len(cc.promotionController.promotionQueue),
		"scorecard_evaluators":  len(cc.scorecardSystem.evaluators),
		"constitutional_enabled": cc.config.ConstitutionalConfig.ValidationRequired,
		"auto_promotion_enabled": cc.config.PromotionPolicies.AutoPromotionEnabled,
	}

	return stats
}

// Internal methods

func (cc *CoordinatorCluster) validateConstitutionalCompliance(capability *RegisteredCapability) (*ConstitutionalCheck, error) {
	if !cc.config.ConstitutionalConfig.ValidationRequired {
		return &ConstitutionalCheck{
			ID:              fmt.Sprintf("const-check-%d", time.Now().Unix()),
			Status:          "approved",
			ScriptureScore:  1.0,
			GeometryScore:   1.0,
			BridgePathScore: 1.0,
			OverallScore:    1.0,
			Verdict:         "APPROVED",
			Reasoning:       "Constitutional validation disabled",
			CheckedAt:       time.Now(),
			ValidUntil:      time.Now().Add(24 * time.Hour),
		}, nil
	}

	// Call Trifecta Court for validation
	request := map[string]interface{}{
		"action":        "register_capability",
		"capability_id": capability.ID,
		"capability":    capability,
		"context": map[string]interface{}{
			"stage":       "coordinator",
			"environment": "control_plane",
			"timestamp":   time.Now().Unix(),
		},
	}

	// Simulate Trifecta Court response (in real implementation, this would be an HTTP call)
	check := &ConstitutionalCheck{
		ID:              fmt.Sprintf("const-check-%d", time.Now().Unix()),
		Status:          "approved",
		ScriptureScore:  0.94,
		GeometryScore:   0.91,
		BridgePathScore: 0.87,
		OverallScore:    0.91,
		Verdict:         "APPROVED",
		Reasoning:       "Capability meets constitutional requirements with optimization recommendations",
		Conditions:      []string{"Monitor resource usage", "Implement audit logging"},
		Violations:      []string{},
		AuditTrail: []*ConstitutionalAudit{
			{
				Timestamp: time.Now(),
				Action:    "constitutional_validation",
				Decision:  "APPROVED",
				Reasoning: "All courts approved with conditions",
				Evidence:  map[string]interface{}{"request": request},
				Auditor:   "trifecta-court",
				Hash:      fmt.Sprintf("hash-%d", time.Now().Unix()),
			},
		},
		CheckedAt:  time.Now(),
		ValidUntil: time.Now().Add(24 * time.Hour),
	}

	cc.metrics.ConstitutionalChecks.Inc()
	return check, nil
}

func (cc *CoordinatorCluster) updateDependencyGraph(capability *RegisteredCapability) {
	cc.capabilityRegistry.mutex.Lock()
	defer cc.capabilityRegistry.mutex.Unlock()

	// Update dependencies
	cc.capabilityRegistry.dependencies[capability.ID] = capability.Dependencies

	// Update dependents
	for _, dep := range capability.Dependencies {
		if cc.capabilityRegistry.dependents[dep] == nil {
			cc.capabilityRegistry.dependents[dep] = []string{}
		}
		cc.capabilityRegistry.dependents[dep] = append(cc.capabilityRegistry.dependents[dep], capability.ID)
	}
}

func (cc *CoordinatorCluster) initializeScorecard(capability *RegisteredCapability) *CapabilityScorecard {
	scorecard := &CapabilityScorecard{
		ID:              fmt.Sprintf("scorecard-%s", capability.ID),
		CapabilityID:    capability.ID,
		OverallScore:    0.0,
		Grade:           "F",
		DimensionScores: make(map[string]float64),
		SubScores:       make(map[string]float64),
		Trends:          make(map[string]*TrendData),
		Benchmarks:      make(map[string]*BenchmarkResult),
		Recommendations: []*Recommendation{},
		Alerts:          []*ScorecardAlert{},
		LastEvaluated:   time.Time{},
		NextEvaluation:  time.Now().Add(cc.config.ScorecardConfig.EvaluationInterval),
		EvaluationHistory: []*EvaluationRecord{},
	}

	// Initialize dimension scores
	for _, dimension := range cc.config.ScorecardConfig.Dimensions {
		if dimension.Enabled {
			scorecard.DimensionScores[dimension.Name] = 0.0
		}
	}

	return scorecard
}

func (cc *CoordinatorCluster) validatePromotionRequest(request *PromotionRequest, capability *RegisteredCapability) error {
	// Check if capability is eligible for promotion
	if capability.Status != "registered" && capability.Status != "tested" {
		return fmt.Errorf("capability not eligible for promotion: status=%s", capability.Status)
	}

	// Check scorecard requirements
	riskTier := cc.config.PromotionPolicies.RiskTiers[capability.RiskTier]
	if riskTier != nil && capability.Scorecard.OverallScore < riskTier.RequiredScore {
		return fmt.Errorf("capability score too low: required=%.2f, actual=%.2f", 
			riskTier.RequiredScore, capability.Scorecard.OverallScore)
	}

	// Check constitutional compliance
	if cc.config.ConstitutionalConfig.ValidationRequired {
		if capability.ConstitutionalCheck == nil || 
		   capability.ConstitutionalCheck.Verdict != "APPROVED" ||
		   time.Now().After(capability.ConstitutionalCheck.ValidUntil) {
			return fmt.Errorf("constitutional compliance required")
		}
	}

	return nil
}

func (cc *CoordinatorCluster) determinePromotionStrategy(capability *RegisteredCapability) string {
	riskTier := cc.config.PromotionPolicies.RiskTiers[capability.RiskTier]
	if riskTier == nil {
		return "standard"
	}

	if riskTier.CanaryPercent > 0 {
		return "canary"
	}

	if riskTier.ManualApproval {
		return "manual"
	}

	return "standard"
}

func (cc *CoordinatorCluster) processPromotion(promotion *PromotionRecord) {
	cc.logger.Infof("🔧 Processing promotion: %s", promotion.ID)

	startTime := time.Now()
	promotion.Status = "processing"

	// Execute promotion gates
	if err := cc.executePromotionGates(promotion); err != nil {
		cc.logger.Error("Promotion gates failed:", err)
		promotion.Status = "failed"
		cc.metrics.PromotionsFailed.Inc()
		return
	}

	// Check for manual approval requirement
	if cc.requiresManualApproval(promotion) {
		promotion.Status = "awaiting_approval"
		cc.logger.Infof("⏳ Promotion awaiting manual approval: %s", promotion.ID)
		return
	}

	// Execute rollout
	if err := cc.executeRollout(promotion); err != nil {
		cc.logger.Error("Rollout failed:", err)
		promotion.Status = "failed"
		cc.metrics.PromotionsFailed.Inc()
		return
	}

	// Complete promotion
	promotion.Status = "completed"
	promotion.CompletedAt = time.Now()
	promotion.Duration = promotion.CompletedAt.Sub(promotion.StartedAt)

	// Update metrics
	cc.metrics.PromotionsSuccessful.Inc()
	cc.metrics.ActivePromotions.Dec()
	cc.metrics.AveragePromotionTime.Observe(promotion.Duration.Seconds())

	cc.logger.Infof("✅ Promotion completed: %s", promotion.ID)
}

func (cc *CoordinatorCluster) executePromotionGates(promotion *PromotionRecord) error {
	cc.logger.Infof("🚪 Executing promotion gates for: %s", promotion.ID)

	for _, gate := range cc.config.PromotionPolicies.PromotionGates {
		if !gate.Required {
			continue
		}

		result := &GateResult{
			GateID:    gate.ID,
			Status:    "running",
			Timestamp: time.Now(),
		}

		// Execute gate
		if err := cc.executeGate(gate, promotion, result); err != nil {
			result.Status = "failed"
			result.Error = err.Error()
			promotion.Gates = append(promotion.Gates, result)
			return fmt.Errorf("gate %s failed: %w", gate.ID, err)
		}

		result.Status = "passed"
		result.Duration = time.Since(result.Timestamp)
		promotion.Gates = append(promotion.Gates, result)

		cc.logger.Infof("✅ Gate passed: %s", gate.ID)
	}

	return nil
}

func (cc *CoordinatorCluster) executeGate(gate *PromotionGate, promotion *PromotionRecord, result *GateResult) error {
	switch gate.Type {
	case "scorecard":
		return cc.executeScoreCardGate(gate, promotion, result)
	case "constitutional":
		return cc.executeConstitutionalGate(gate, promotion, result)
	case "security":
		return cc.executeSecurityGate(gate, promotion, result)
	case "performance":
		return cc.executePerformanceGate(gate, promotion, result)
	default:
		return fmt.Errorf("unknown gate type: %s", gate.Type)
	}
}

func (cc *CoordinatorCluster) executeScoreCardGate(gate *PromotionGate, promotion *PromotionRecord, result *GateResult) error {
	capability, err := cc.GetCapability(promotion.CapabilityID)
	if err != nil {
		return err
	}

	scorecard := capability.Scorecard
	if scorecard == nil {
		return fmt.Errorf("scorecard not found")
	}

	threshold, ok := gate.Criteria["min_score"].(float64)
	if !ok {
		threshold = 0.8
	}

	if scorecard.OverallScore < threshold {
		return fmt.Errorf("scorecard score too low: %.2f < %.2f", scorecard.OverallScore, threshold)
	}

	result.Score = scorecard.OverallScore
	result.Results = map[string]interface{}{
		"overall_score":    scorecard.OverallScore,
		"grade":           scorecard.Grade,
		"dimension_scores": scorecard.DimensionScores,
	}

	return nil
}

func (cc *CoordinatorCluster) executeConstitutionalGate(gate *PromotionGate, promotion *PromotionRecord, result *GateResult) error {
	capability, err := cc.GetCapability(promotion.CapabilityID)
	if err != nil {
		return err
	}

	check := capability.ConstitutionalCheck
	if check == nil || check.Verdict != "APPROVED" {
		return fmt.Errorf("constitutional validation required")
	}

	if time.Now().After(check.ValidUntil) {
		return fmt.Errorf("constitutional validation expired")
	}

	threshold, ok := gate.Criteria["min_score"].(float64)
	if !ok {
		threshold = cc.config.ConstitutionalConfig.ComplianceThreshold
	}

	if check.OverallScore < threshold {
		return fmt.Errorf("constitutional score too low: %.2f < %.2f", check.OverallScore, threshold)
	}

	result.Score = check.OverallScore
	result.Results = map[string]interface{}{
		"scripture_score":   check.ScriptureScore,
		"geometry_score":    check.GeometryScore,
		"bridge_path_score": check.BridgePathScore,
		"overall_score":     check.OverallScore,
		"verdict":           check.Verdict,
	}

	return nil
}

func (cc *CoordinatorCluster) executeSecurityGate(gate *PromotionGate, promotion *PromotionRecord, result *GateResult) error {
	// Simulate security gate execution
	result.Score = 0.95
	result.Results = map[string]interface{}{
		"vulnerabilities": 0,
		"security_score":  0.95,
		"scan_status":     "passed",
	}
	return nil
}

func (cc *CoordinatorCluster) executePerformanceGate(gate *PromotionGate, promotion *PromotionRecord, result *GateResult) error {
	// Simulate performance gate execution
	result.Score = 0.88
	result.Results = map[string]interface{}{
		"latency_p99":     150.0,
		"throughput_rps":  1000.0,
		"cpu_usage":       45.0,
		"memory_usage":    60.0,
		"performance_score": 0.88,
	}
	return nil
}

func (cc *CoordinatorCluster) requiresManualApproval(promotion *PromotionRecord) bool {
	capability, err := cc.GetCapability(promotion.CapabilityID)
	if err != nil {
		return true // Default to requiring approval on error
	}

	riskTier := cc.config.PromotionPolicies.RiskTiers[capability.RiskTier]
	if riskTier != nil {
		return riskTier.ManualApproval
	}

	return false
}

func (cc *CoordinatorCluster) executeRollout(promotion *PromotionRecord) error {
	cc.logger.Infof("🚀 Executing rollout for promotion: %s", promotion.ID)

	// Get rollout strategy
	strategy := cc.getRolloutStrategy(promotion)
	if strategy == nil {
		return fmt.Errorf("rollout strategy not found")
	}

	// Initialize rollout progress
	promotion.RolloutProgress = &RolloutProgress{
		CurrentPhase:    0,
		TotalPhases:     len(strategy.Phases),
		Progress:        0.0,
		Status:          "starting",
		TrafficPercent:  0.0,
		Metrics:         make(map[string]interface{}),
		Issues:          []string{},
		StartedAt:       time.Now(),
	}

	// Execute rollout phases
	for i, phase := range strategy.Phases {
		cc.logger.Infof("📋 Executing rollout phase %d/%d: %s", i+1, len(strategy.Phases), phase.Name)

		promotion.RolloutProgress.CurrentPhase = i
		promotion.RolloutProgress.Status = "running"
		promotion.RolloutProgress.TrafficPercent = phase.TrafficPercent

		// Execute phase
		if err := cc.executeRolloutPhase(promotion, phase); err != nil {
			promotion.RolloutProgress.Status = "failed"
			promotion.RolloutProgress.Issues = append(promotion.RolloutProgress.Issues, err.Error())
			return fmt.Errorf("rollout phase %s failed: %w", phase.Name, err)
		}

		promotion.RolloutProgress.Progress = float64(i+1) / float64(len(strategy.Phases)) * 100
		cc.logger.Infof("✅ Rollout phase completed: %s", phase.Name)
	}

	promotion.RolloutProgress.Status = "completed"
	promotion.RolloutProgress.EstimatedCompletion = time.Now()

	cc.logger.Infof("✅ Rollout completed for promotion: %s", promotion.ID)
	return nil
}

func (cc *CoordinatorCluster) getRolloutStrategy(promotion *PromotionRecord) *RolloutStrategy {
	strategyName := promotion.Strategy
	if strategyName == "" {
		strategyName = "standard"
	}

	strategy := cc.config.PromotionPolicies.RolloutStrategies[strategyName]
	if strategy == nil {
		// Return default strategy
		return &RolloutStrategy{
			Name: "default",
			Type: "rolling",
			Phases: []*RolloutPhase{
				{
					Name:           "canary",
					TrafficPercent: 5.0,
					Duration:       10 * time.Minute,
				},
				{
					Name:           "partial",
					TrafficPercent: 50.0,
					Duration:       30 * time.Minute,
				},
				{
					Name:           "full",
					TrafficPercent: 100.0,
					Duration:       0,
				},
			},
		}
	}

	return strategy
}

func (cc *CoordinatorCluster) executeRolloutPhase(promotion *PromotionRecord, phase *RolloutPhase) error {
	// Simulate rollout phase execution
	cc.logger.Infof("🔧 Executing rollout phase: %s (%.1f%% traffic)", phase.Name, phase.TrafficPercent)

	// Wait for phase duration
	if phase.Duration > 0 {
		time.Sleep(time.Duration(float64(phase.Duration) * 0.1)) // Simulate with 10% of actual duration
	}

	// Check success criteria
	if phase.SuccessCriteria != nil {
		if err := cc.checkSuccessCriteria(promotion, phase); err != nil {
			return err
		}
	}

	return nil
}

func (cc *CoordinatorCluster) checkSuccessCriteria(promotion *PromotionRecord, phase *RolloutPhase) error {
	// Simulate success criteria checking
	// In real implementation, this would check metrics from monitoring systems
	return nil
}

// Background routines

func (cc *CoordinatorCluster) promotionProcessingRoutine() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			cc.processPromotionQueue()
		case <-cc.shutdownChan:
			return
		}
	}
}

func (cc *CoordinatorCluster) processPromotionQueue() {
	cc.promotionController.mutex.Lock()
	defer cc.promotionController.mutex.Unlock()

	// Process pending promotion requests
	for i, request := range cc.promotionController.promotionQueue {
		if cc.config.PromotionPolicies.AutoPromotionEnabled {
			// Auto-process promotion
			promotion, err := cc.RequestPromotion(request)
			if err != nil {
				cc.logger.Error("Auto-promotion failed:", err)
				continue
			}

			cc.logger.Infof("🤖 Auto-promotion initiated: %s", promotion.ID)
		}

		// Remove processed request
		cc.promotionController.promotionQueue = append(
			cc.promotionController.promotionQueue[:i],
			cc.promotionController.promotionQueue[i+1:]...)
		break // Process one at a time
	}
}

func (cc *CoordinatorCluster) scorecardEvaluationRoutine() {
	ticker := time.NewTicker(cc.config.ScorecardConfig.EvaluationInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			cc.evaluateScorecards()
		case <-cc.shutdownChan:
			return
		}
	}
}

func (cc *CoordinatorCluster) evaluateScorecards() {
	cc.logger.Info("📊 Evaluating scorecards")

	cc.scorecardSystem.mutex.RLock()
	scorecards := make([]*CapabilityScorecard, 0, len(cc.scorecardSystem.scorecards))
	for _, scorecard := range cc.scorecardSystem.scorecards {
		if time.Now().After(scorecard.NextEvaluation) {
			scorecards = append(scorecards, scorecard)
		}
	}
	cc.scorecardSystem.mutex.RUnlock()

	for _, scorecard := range scorecards {
		if err := cc.evaluateScorecard(scorecard); err != nil {
			cc.logger.Error("Scorecard evaluation failed:", err)
		}
	}
}

func (cc *CoordinatorCluster) evaluateScorecard(scorecard *CapabilityScorecard) error {
	cc.logger.Infof("📈 Evaluating scorecard: %s", scorecard.CapabilityID)

	startTime := time.Now()

	// Simulate scorecard evaluation
	dimensionScores := make(map[string]float64)
	totalScore := 0.0
	totalWeight := 0.0

	for _, dimension := range cc.config.ScorecardConfig.Dimensions {
		if !dimension.Enabled {
			continue
		}

		// Simulate dimension evaluation
		score := 0.7 + (float64(time.Now().Unix()%30) / 100.0) // 0.7-0.99
		dimensionScores[dimension.Name] = score
		
		weight := cc.config.ScorecardConfig.Weights[dimension.Name]
		if weight == 0 {
			weight = dimension.Weight
		}
		
		totalScore += score * weight
		totalWeight += weight
	}

	overallScore := totalScore / totalWeight
	grade := cc.calculateGrade(overallScore)

	// Update scorecard
	cc.scorecardSystem.mutex.Lock()
	scorecard.OverallScore = overallScore
	scorecard.Grade = grade
	scorecard.DimensionScores = dimensionScores
	scorecard.LastEvaluated = time.Now()
	scorecard.NextEvaluation = time.Now().Add(cc.config.ScorecardConfig.EvaluationInterval)

	// Add evaluation record
	record := &EvaluationRecord{
		ID:              fmt.Sprintf("eval-%d", time.Now().Unix()),
		OverallScore:    overallScore,
		Grade:           grade,
		DimensionScores: dimensionScores,
		Timestamp:       time.Now(),
		Duration:        time.Since(startTime),
	}
	scorecard.EvaluationHistory = append(scorecard.EvaluationHistory, record)

	// Keep only last 100 evaluations
	if len(scorecard.EvaluationHistory) > 100 {
		scorecard.EvaluationHistory = scorecard.EvaluationHistory[len(scorecard.EvaluationHistory)-100:]
	}
	cc.scorecardSystem.mutex.Unlock()

	// Update capability scorecard reference
	cc.capabilityRegistry.mutex.Lock()
	if capability, exists := cc.capabilityRegistry.capabilities[scorecard.CapabilityID]; exists {
		capability.Scorecard = scorecard
		capability.UpdatedAt = time.Now()
	}
	cc.capabilityRegistry.mutex.Unlock()

	// Update metrics
	cc.metrics.ScorecardEvaluations.Inc()
	cc.metrics.ScorecardScores.Observe(overallScore)

	cc.logger.Infof("✅ Scorecard evaluated: %s (score: %.2f, grade: %s)", 
		scorecard.CapabilityID, overallScore, grade)
	return nil
}

func (cc *CoordinatorCluster) calculateGrade(score float64) string {
	if score >= 0.9 {
		return "A"
	} else if score >= 0.8 {
		return "B"
	} else if score >= 0.7 {
		return "C"
	} else if score >= 0.6 {
		return "D"
	} else {
		return "F"
	}
}

func (cc *CoordinatorCluster) constitutionalValidationRoutine() {
	ticker := time.NewTicker(1 * time.Hour)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			cc.performPeriodicConstitutionalValidation()
		case <-cc.shutdownChan:
			return
		}
	}
}

func (cc *CoordinatorCluster) performPeriodicConstitutionalValidation() {
	if !cc.config.ConstitutionalConfig.ValidationRequired {
		return
	}

	cc.logger.Info("🏛️ Performing periodic constitutional validation")

	cc.capabilityRegistry.mutex.RLock()
	capabilities := make([]*RegisteredCapability, 0, len(cc.capabilityRegistry.capabilities))
	for _, capability := range cc.capabilityRegistry.capabilities {
		if capability.ConstitutionalCheck != nil && 
		   time.Since(capability.ConstitutionalCheck.CheckedAt) > 24*time.Hour {
			capabilities = append(capabilities, capability)
		}
	}
	cc.capabilityRegistry.mutex.RUnlock()

	for _, capability := range capabilities {
		check, err := cc.validateConstitutionalCompliance(capability)
		if err != nil {
			cc.logger.Error("Constitutional re-validation failed:", err)
			continue
		}

		capability.ConstitutionalCheck = check
		capability.UpdatedAt = time.Now()

		if check.Verdict != "APPROVED" {
			cc.logger.Warnf("🚨 Constitutional compliance violation detected for capability %s", capability.ID)
			// Mark capability for review or suspension
			capability.Status = "suspended"
		}
	}
}

func (cc *CoordinatorCluster) registryMaintenanceRoutine() {
	ticker := time.NewTicker(6 * time.Hour)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			cc.performRegistryMaintenance()
		case <-cc.shutdownChan:
			return
		}
	}
}

func (cc *CoordinatorCluster) performRegistryMaintenance() {
	cc.logger.Info("🧹 Performing registry maintenance")

	// Clean up expired capabilities
	cc.capabilityRegistry.mutex.Lock()
	for id, capability := range cc.capabilityRegistry.capabilities {
		if !capability.ExpiresAt.IsZero() && time.Now().After(capability.ExpiresAt) {
			delete(cc.capabilityRegistry.capabilities, id)
			cc.logger.Infof("🗑️ Removed expired capability: %s", id)
		}
	}
	cc.capabilityRegistry.mutex.Unlock()

	// Clean up completed promotions
	cc.promotionController.mutex.Lock()
	for id, promotion := range cc.promotionController.activePromotions {
		if promotion.Status == "completed" || promotion.Status == "failed" {
			if time.Since(promotion.CompletedAt) > 24*time.Hour {
				delete(cc.promotionController.activePromotions, id)
				cc.logger.Infof("🗑️ Archived promotion: %s", id)
			}
		}
	}
	cc.promotionController.mutex.Unlock()
}

func (cc *CoordinatorCluster) metricsRoutine() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			cc.updateSystemMetrics()
		case <-cc.shutdownChan:
			return
		}
	}
}

func (cc *CoordinatorCluster) updateSystemMetrics() {
	cc.mutex.RLock()
	defer cc.mutex.RUnlock()

	cc.metrics.RegistrySize.Set(float64(len(cc.capabilityRegistry.capabilities)))
	cc.metrics.ActivePromotions.Set(float64(len(cc.promotionController.activePromotions)))
}

// Database schema and data loading

func (cc *CoordinatorCluster) initializeSchema() error {
	schema := `
		CREATE TABLE IF NOT EXISTS capabilities (
			id VARCHAR(255) PRIMARY KEY,
			name VARCHAR(255) NOT NULL,
			version VARCHAR(50) NOT NULL,
			type VARCHAR(50) NOT NULL,
			description TEXT,
			content_hash VARCHAR(64) NOT NULL,
			signature TEXT,
			source_url TEXT,
			dependencies JSONB,
			dependents JSONB,
			compatibility_matrix JSONB,
			metadata JSONB,
			status VARCHAR(50) NOT NULL,
			risk_tier VARCHAR(50) NOT NULL,
			scorecard JSONB,
			promotion_history JSONB,
			constitutional_check JSONB,
			provenance_record JSONB,
			registered_at TIMESTAMP NOT NULL,
			updated_at TIMESTAMP NOT NULL,
			expires_at TIMESTAMP
		);

		CREATE TABLE IF NOT EXISTS promotions (
			id VARCHAR(255) PRIMARY KEY,
			capability_id VARCHAR(255) NOT NULL,
			from_stage VARCHAR(50) NOT NULL,
			to_stage VARCHAR(50) NOT NULL,
			status VARCHAR(50) NOT NULL,
			strategy VARCHAR(50) NOT NULL,
			risk_tier VARCHAR(50) NOT NULL,
			score FLOAT NOT NULL,
			gates JSONB,
			approvals JSONB,
			rollout_progress JSONB,
			constitutional_check JSONB,
			started_at TIMESTAMP NOT NULL,
			completed_at TIMESTAMP,
			duration INTERVAL,
			metadata JSONB
		);

		CREATE TABLE IF NOT EXISTS scorecards (
			id VARCHAR(255) PRIMARY KEY,
			capability_id VARCHAR(255) NOT NULL,
			overall_score FLOAT NOT NULL,
			grade VARCHAR(2) NOT NULL,
			dimension_scores JSONB,
			sub_scores JSONB,
			trends JSONB,
			benchmarks JSONB,
			recommendations JSONB,
			alerts JSONB,
			last_evaluated TIMESTAMP NOT NULL,
			next_evaluation TIMESTAMP NOT NULL,
			evaluation_history JSONB
		);

		CREATE INDEX IF NOT EXISTS idx_capabilities_name ON capabilities(name);
		CREATE INDEX IF NOT EXISTS idx_capabilities_type ON capabilities(type);
		CREATE INDEX IF NOT EXISTS idx_capabilities_status ON capabilities(status);
		CREATE INDEX IF NOT EXISTS idx_promotions_capability ON promotions(capability_id);
		CREATE INDEX IF NOT EXISTS idx_promotions_status ON promotions(status);
		CREATE INDEX IF NOT EXISTS idx_scorecards_capability ON scorecards(capability_id);
		CREATE INDEX IF NOT EXISTS idx_scorecards_score ON scorecards(overall_score);
	`

	_, err := cc.db.Exec(schema)
	return err
}

func (cc *CoordinatorCluster) loadData() error {
	// Load existing capabilities
	if err := cc.loadCapabilities(); err != nil {
		cc.logger.Warn("Failed to load capabilities:", err)
	}

	// Load existing promotions
	if err := cc.loadPromotions(); err != nil {
		cc.logger.Warn("Failed to load promotions:", err)
	}

	// Load existing scorecards
	if err := cc.loadScorecards(); err != nil {
		cc.logger.Warn("Failed to load scorecards:", err)
	}

	return nil
}

func (cc *CoordinatorCluster) loadCapabilities() error {
	// Load capabilities from database
	return nil
}

func (cc *CoordinatorCluster) loadPromotions() error {
	// Load promotions from database
	return nil
}

func (cc *CoordinatorCluster) loadScorecards() error {
	// Load scorecards from database
	return nil
}

// REST API endpoints

func (cc *CoordinatorCluster) setupRoutes(router *gin.Engine) {
	api := router.Group("/api/v1")

	// Capability operations
	api.POST("/capabilities", cc.registerCapabilityHandler)
	api.GET("/capabilities", cc.getAllCapabilitiesHandler)
	api.GET("/capabilities/:capability_id", cc.getCapabilityHandler)
	api.PUT("/capabilities/:capability_id", cc.updateCapabilityHandler)
	api.DELETE("/capabilities/:capability_id", cc.deleteCapabilityHandler)

	// Promotion operations
	api.POST("/promotions", cc.requestPromotionHandler)
	api.GET("/promotions", cc.getAllPromotionsHandler)
	api.GET("/promotions/:promotion_id", cc.getPromotionHandler)
	api.POST("/promotions/:promotion_id/approve", cc.approvePromotionHandler)
	api.POST("/promotions/:promotion_id/reject", cc.rejectPromotionHandler)

	// Scorecard operations
	api.GET("/scorecards/:capability_id", cc.getScorecardHandler)
	api.POST("/scorecards/:capability_id/evaluate", cc.evaluateScorecardHandler)

	// System information
	api.GET("/stats", cc.getSystemStatsHandler)
	api.GET("/health", cc.getHealthHandler)

	// Metrics
	api.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (cc *CoordinatorCluster) registerCapabilityHandler(c *gin.Context) {
	var capability RegisteredCapability

	if err := c.ShouldBindJSON(&capability); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := cc.RegisterCapability(&capability); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, capability)
}

func (cc *CoordinatorCluster) getAllCapabilitiesHandler(c *gin.Context) {
	capabilities, err := cc.GetAllCapabilities()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"capabilities": capabilities,
		"count":        len(capabilities),
	})
}

func (cc *CoordinatorCluster) getCapabilityHandler(c *gin.Context) {
	capabilityID := c.Param("capability_id")

	capability, err := cc.GetCapability(capabilityID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, capability)
}

func (cc *CoordinatorCluster) updateCapabilityHandler(c *gin.Context) {
	capabilityID := c.Param("capability_id")

	var updates map[string]interface{}
	if err := c.ShouldBindJSON(&updates); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	capability, err := cc.GetCapability(capabilityID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	// Apply updates
	if status, ok := updates["status"].(string); ok {
		capability.Status = status
	}
	if riskTier, ok := updates["risk_tier"].(string); ok {
		capability.RiskTier = riskTier
	}
	capability.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, capability)
}

func (cc *CoordinatorCluster) deleteCapabilityHandler(c *gin.Context) {
	capabilityID := c.Param("capability_id")

	cc.capabilityRegistry.mutex.Lock()
	delete(cc.capabilityRegistry.capabilities, capabilityID)
	cc.capabilityRegistry.mutex.Unlock()

	c.JSON(http.StatusOK, gin.H{"message": "Capability deleted", "capability_id": capabilityID})
}

func (cc *CoordinatorCluster) requestPromotionHandler(c *gin.Context) {
	var request PromotionRequest

	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	request.ID = fmt.Sprintf("req-%d", time.Now().Unix())
	request.RequestedAt = time.Now()

	promotion, err := cc.RequestPromotion(&request)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, promotion)
}

func (cc *CoordinatorCluster) getAllPromotionsHandler(c *gin.Context) {
	cc.promotionController.mutex.RLock()
	promotions := make([]*PromotionRecord, 0, len(cc.promotionController.activePromotions))
	for _, promotion := range cc.promotionController.activePromotions {
		promotions = append(promotions, promotion)
	}
	cc.promotionController.mutex.RUnlock()

	c.JSON(http.StatusOK, gin.H{
		"promotions": promotions,
		"count":      len(promotions),
	})
}

func (cc *CoordinatorCluster) getPromotionHandler(c *gin.Context) {
	promotionID := c.Param("promotion_id")

	promotion, err := cc.GetPromotion(promotionID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, promotion)
}

func (cc *CoordinatorCluster) approvePromotionHandler(c *gin.Context) {
	promotionID := c.Param("promotion_id")

	var request struct {
		Approver string   `json:"approver" binding:"required"`
		Comments string   `json:"comments"`
		Conditions []string `json:"conditions"`
	}

	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	promotion, err := cc.GetPromotion(promotionID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	approval := &ApprovalRecord{
		ID:         fmt.Sprintf("approval-%d", time.Now().Unix()),
		Type:       "manual",
		Approver:   request.Approver,
		Status:     "approved",
		Comments:   request.Comments,
		Conditions: request.Conditions,
		Timestamp:  time.Now(),
	}

	promotion.Approvals = append(promotion.Approvals, approval)
	promotion.Status = "approved"

	// Continue promotion process
	go cc.executeRollout(promotion)

	c.JSON(http.StatusOK, gin.H{"message": "Promotion approved", "promotion_id": promotionID})
}

func (cc *CoordinatorCluster) rejectPromotionHandler(c *gin.Context) {
	promotionID := c.Param("promotion_id")

	var request struct {
		Approver string `json:"approver" binding:"required"`
		Comments string `json:"comments" binding:"required"`
	}

	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	promotion, err := cc.GetPromotion(promotionID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	approval := &ApprovalRecord{
		ID:        fmt.Sprintf("approval-%d", time.Now().Unix()),
		Type:      "manual",
		Approver:  request.Approver,
		Status:    "rejected",
		Comments:  request.Comments,
		Timestamp: time.Now(),
	}

	promotion.Approvals = append(promotion.Approvals, approval)
	promotion.Status = "rejected"
	promotion.CompletedAt = time.Now()

	cc.metrics.PromotionsFailed.Inc()
	cc.metrics.ActivePromotions.Dec()

	c.JSON(http.StatusOK, gin.H{"message": "Promotion rejected", "promotion_id": promotionID})
}

func (cc *CoordinatorCluster) getScorecardHandler(c *gin.Context) {
	capabilityID := c.Param("capability_id")

	scorecard, err := cc.GetScorecard(capabilityID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, scorecard)
}

func (cc *CoordinatorCluster) evaluateScorecardHandler(c *gin.Context) {
	capabilityID := c.Param("capability_id")

	scorecard, err := cc.GetScorecard(capabilityID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	if err := cc.evaluateScorecard(scorecard); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Scorecard evaluation initiated", "capability_id": capabilityID})
}

func (cc *CoordinatorCluster) getSystemStatsHandler(c *gin.Context) {
	stats := cc.GetSystemStats()
	c.JSON(http.StatusOK, stats)
}

func (cc *CoordinatorCluster) getHealthHandler(c *gin.Context) {
	health := map[string]interface{}{
		"status":    "healthy",
		"timestamp": time.Now(),
		"version":   "1.0.0",
		"uptime":    time.Since(time.Now().Add(-1 * time.Hour)).String(),
	}

	c.JSON(http.StatusOK, health)
}

func main() {
	// Load configuration
	config := &CoordinatorConfig{
		ServerPort:            getEnv("SERVER_PORT", "8001"),
		RedisURL:              getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:           getEnv("DATABASE_URL", "postgres://user:password@localhost/arkos?sslmode=disable"),
		SandboxClusterURL:     getEnv("SANDBOX_CLUSTER_URL", "http://localhost:8004"),
		DeployedAppClusterURL: getEnv("DEPLOYED_APP_CLUSTER_URL", "http://localhost:8003"),
		TrifectaCourtURL:      getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		PromotionPolicies: &PromotionPolicies{
			AutoPromotionEnabled: true,
			DefaultRiskTier:      "medium",
			RiskTiers: map[string]*RiskTier{
				"low": {
					Name:                "Low Risk",
					RequiredScore:       0.7,
					TestingDuration:     30 * time.Minute,
					ManualApproval:      false,
					CanaryPercent:       5.0,
					RolloutDuration:     1 * time.Hour,
					MonitoringDuration:  24 * time.Hour,
					AutoRollback:        true,
					ConstitutionalCheck: true,
				},
				"medium": {
					Name:                "Medium Risk",
					RequiredScore:       0.8,
					TestingDuration:     2 * time.Hour,
					ManualApproval:      false,
					CanaryPercent:       10.0,
					RolloutDuration:     4 * time.Hour,
					MonitoringDuration:  48 * time.Hour,
					AutoRollback:        true,
					ConstitutionalCheck: true,
				},
				"high": {
					Name:                "High Risk",
					RequiredScore:       0.9,
					TestingDuration:     8 * time.Hour,
					ManualApproval:      true,
					CanaryPercent:       5.0,
					RolloutDuration:     24 * time.Hour,
					MonitoringDuration:  7 * 24 * time.Hour,
					AutoRollback:        true,
					ConstitutionalCheck: true,
				},
			},
			PromotionGates: []*PromotionGate{
				{
					ID:       "scorecard",
					Name:     "Scorecard Gate",
					Type:     "scorecard",
					Required: true,
					Criteria: map[string]interface{}{"min_score": 0.8},
					Timeout:  10 * time.Minute,
				},
				{
					ID:       "constitutional",
					Name:     "Constitutional Gate",
					Type:     "constitutional",
					Required: true,
					Criteria: map[string]interface{}{"min_score": 0.8},
					Timeout:  5 * time.Minute,
				},
				{
					ID:       "security",
					Name:     "Security Gate",
					Type:     "security",
					Required: true,
					Criteria: map[string]interface{}{"max_vulnerabilities": 0},
					Timeout:  15 * time.Minute,
				},
			},
			RolloutStrategies: map[string]*RolloutStrategy{
				"canary": {
					Name: "Canary Rollout",
					Type: "canary",
					Phases: []*RolloutPhase{
						{Name: "canary", TrafficPercent: 5.0, Duration: 30 * time.Minute},
						{Name: "partial", TrafficPercent: 50.0, Duration: 1 * time.Hour},
						{Name: "full", TrafficPercent: 100.0, Duration: 0},
					},
				},
				"blue-green": {
					Name: "Blue-Green Rollout",
					Type: "blue-green",
					Phases: []*RolloutPhase{
						{Name: "prepare", TrafficPercent: 0.0, Duration: 15 * time.Minute},
						{Name: "switch", TrafficPercent: 100.0, Duration: 0},
					},
				},
			},
			BudgetConstraints: &BudgetConstraints{
				MaxCostPerPromotion: 1000.0,
				MaxCostPerHour:      100.0,
				MaxCostPerDay:       2000.0,
				MaxCostPerMonth:     50000.0,
				BudgetAlerts:        true,
			},
		},
		ScorecardConfig: &ScorecardConfig{
			EvaluationInterval: 1 * time.Hour,
			Dimensions: []*ScorecardDimension{
				{Name: "performance", Weight: 0.2, Enabled: true},
				{Name: "reliability", Weight: 0.2, Enabled: true},
				{Name: "security", Weight: 0.15, Enabled: true},
				{Name: "quality", Weight: 0.15, Enabled: true},
				{Name: "usability", Weight: 0.1, Enabled: true},
				{Name: "maintainability", Weight: 0.1, Enabled: true},
				{Name: "constitutional", Weight: 0.1, Enabled: true},
			},
			Weights: map[string]float64{
				"performance":      0.2,
				"reliability":      0.2,
				"security":         0.15,
				"quality":          0.15,
				"constitutional":   0.1,
			},
			Thresholds: map[string]float64{
				"promotion": 0.8,
				"warning":   0.6,
				"critical":  0.4,
			},
		},
		ConstitutionalConfig: &ConstitutionalConfig{
			TrifectaCourtURL:    getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
			ValidationRequired:  true,
			EthicsCheckEnabled:  true,
			GovernanceEnabled:   true,
			AuditTrailRequired:  true,
			ComplianceThreshold: 0.8,
		},
	}

	// Create coordinator cluster
	cluster, err := NewCoordinatorCluster(config)
	if err != nil {
		log.Fatal("Failed to create coordinator cluster:", err)
	}

	// Start cluster
	if err := cluster.Start(); err != nil {
		log.Fatal("Failed to start coordinator cluster:", err)
	}

	// Setup HTTP server
	router := gin.Default()
	cluster.setupRoutes(router)

	// Start server
	log.Printf("🚀 Coordinator Cluster starting on port %s", config.ServerPort)
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

