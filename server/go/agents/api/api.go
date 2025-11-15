package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"os/signal"
	"syscall"
	"time"
	"strings"
	"crypto/sha256"
	"encoding/hex"
	"sync"

	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	_ "github.com/lib/pq"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
)

// APIGateway represents the central request routing and service mesh
type APIGateway struct {
	logger         *logrus.Logger
	redis          *redis.Client
	db             *sql.DB
	router         *gin.Engine
	config         *Config
	services       map[string]*ServiceEndpoint
	loadBalancer   *LoadBalancer
	rateLimiter    *RateLimiter
	circuitBreaker *CircuitBreaker
	trifectaCourt  *TrifectaCourtClient
	mutex          sync.RWMutex
}

// Config holds the API gateway configuration
type Config struct {
	ServerPort       string `yaml:"server_port"`
	RedisURL         string `yaml:"redis_url"`
	DatabaseURL      string `yaml:"database_url"`
	LogLevel         string `yaml:"log_level"`
	TrifectaCourtURL string `yaml:"trifecta_court_url"`
	RateLimitRPS     int    `yaml:"rate_limit_rps"`
	CircuitThreshold int    `yaml:"circuit_threshold"`
	HealthCheckInterval int `yaml:"health_check_interval_seconds"`
}

// ServiceEndpoint represents a backend service
type ServiceEndpoint struct {
	ID          string                 `json:"id"`
	Name        string                 `json:"name"`
	URL         string                 `json:"url"`
	Status      string                 `json:"status"`
	Weight      int                    `json:"weight"`
	Priority    int                    `json:"priority"`
	HealthCheck *HealthCheckConfig     `json:"health_check"`
	Metadata    map[string]interface{} `json:"metadata"`
	LastCheck   time.Time              `json:"last_check"`
	Failures    int                    `json:"failures"`
	CreatedAt   time.Time              `json:"created_at"`
	UpdatedAt   time.Time              `json:"updated_at"`
}

// HealthCheckConfig represents health check configuration
type HealthCheckConfig struct {
	Path     string `json:"path"`
	Interval int    `json:"interval_seconds"`
	Timeout  int    `json:"timeout_seconds"`
	Retries  int    `json:"retries"`
}

// LoadBalancer handles request distribution
type LoadBalancer struct {
	gateway   *APIGateway
	algorithm string
	counter   int64
	mutex     sync.Mutex
}

// RateLimiter handles request rate limiting
type RateLimiter struct {
	gateway *APIGateway
	limits  map[string]*RateLimit
	mutex   sync.RWMutex
}

// RateLimit represents rate limiting configuration
type RateLimit struct {
	RequestsPerSecond int       `json:"requests_per_second"`
	BurstSize         int       `json:"burst_size"`
	LastReset         time.Time `json:"last_reset"`
	RequestCount      int       `json:"request_count"`
}

// CircuitBreaker handles circuit breaking
type CircuitBreaker struct {
	gateway   *APIGateway
	circuits  map[string]*Circuit
	mutex     sync.RWMutex
}

// Circuit represents a circuit breaker state
type Circuit struct {
	State         string    `json:"state"`
	FailureCount  int       `json:"failure_count"`
	LastFailure   time.Time `json:"last_failure"`
	NextAttempt   time.Time `json:"next_attempt"`
	SuccessCount  int       `json:"success_count"`
	Threshold     int       `json:"threshold"`
	Timeout       int       `json:"timeout_seconds"`
}

// RouteRequest represents a routing request
type RouteRequest struct {
	Service     string                 `json:"service"`
	Path        string                 `json:"path"`
	Method      string                 `json:"method"`
	Headers     map[string]string      `json:"headers"`
	Body        interface{}            `json:"body"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// RouteResponse represents the routing result
type RouteResponse struct {
	Success     bool                   `json:"success"`
	ServiceID   string                 `json:"service_id"`
	Endpoint    string                 `json:"endpoint"`
	StatusCode  int                    `json:"status_code"`
	Headers     map[string]string      `json:"headers"`
	Body        interface{}            `json:"body"`
	Duration    int64                  `json:"duration_ms"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// ServiceRegistration represents service registration
type ServiceRegistration struct {
	Name        string                 `json:"name"`
	URL         string                 `json:"url"`
	Weight      int                    `json:"weight"`
	Priority    int                    `json:"priority"`
	HealthCheck *HealthCheckConfig     `json:"health_check"`
	Metadata    map[string]interface{} `json:"metadata"`
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
		ServerPort:          getEnv("SERVER_PORT", "8080"),
		RedisURL:            getEnv("REDIS_URL", "redis://localhost:6379"),
		DatabaseURL:         getEnv("DATABASE_URL", "postgres://postgres:password@localhost:5432/ark_os_noa?sslmode=disable"),
		LogLevel:            getEnv("LOG_LEVEL", "info"),
		TrifectaCourtURL:    getEnv("TRIFECTA_COURT_URL", "http://localhost:8000"),
		RateLimitRPS:        1000,
		CircuitThreshold:    5,
		HealthCheckInterval: 30,
	}

	logger.Info("🌐 Starting API Gateway Service")

	// Initialize API Gateway
	apiGateway, err := NewAPIGateway(config, logger)
	if err != nil {
		logger.Fatal("Failed to initialize API Gateway:", err)
	}

	// Start the service
	if err := apiGateway.Start(); err != nil {
		logger.Fatal("Failed to start API Gateway:", err)
	}
}

func NewAPIGateway(config *Config, logger *logrus.Logger) (*APIGateway, error) {
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
	router.Use(gin.Logger(), gin.Recovery(), CORSMiddleware())

	// Create API gateway
	ag := &APIGateway{
		logger:   logger,
		redis:    redisClient,
		db:       db,
		router:   router,
		config:   config,
		services: make(map[string]*ServiceEndpoint),
	}

	// Initialize components
	ag.loadBalancer = &LoadBalancer{
		gateway:   ag,
		algorithm: "round_robin",
	}
	ag.rateLimiter = &RateLimiter{
		gateway: ag,
		limits:  make(map[string]*RateLimit),
	}
	ag.circuitBreaker = &CircuitBreaker{
		gateway:  ag,
		circuits: make(map[string]*Circuit),
	}
	ag.trifectaCourt = &TrifectaCourtClient{
		baseURL: config.TrifectaCourtURL,
		client:  &http.Client{Timeout: 10 * time.Second},
	}

	// Setup routes
	ag.setupRoutes()

	// Register default services
	ag.registerDefaultServices()

	// Start background tasks
	go ag.startHealthChecker()
	go ag.startServiceDiscovery()

	return ag, nil
}

func (ag *APIGateway) setupRoutes() {
	// Health check
	ag.router.GET("/health", ag.healthCheck)

	// Service registration
	ag.router.POST("/services/register", ag.registerService)
	ag.router.DELETE("/services/:service_id/deregister", ag.deregisterService)
	ag.router.GET("/services", ag.listServices)
	ag.router.GET("/services/:service_id", ag.getService)
	ag.router.PUT("/services/:service_id", ag.updateService)

	// Service discovery
	ag.router.GET("/discovery/services", ag.discoverServices)
	ag.router.GET("/discovery/endpoints/:service", ag.getServiceEndpoints)

	// Load balancing
	ag.router.GET("/loadbalancer/status", ag.getLoadBalancerStatus)
	ag.router.PUT("/loadbalancer/algorithm", ag.updateLoadBalancerAlgorithm)

	// Rate limiting
	ag.router.GET("/ratelimit/status", ag.getRateLimitStatus)
	ag.router.PUT("/ratelimit/config", ag.updateRateLimitConfig)

	// Circuit breaker
	ag.router.GET("/circuitbreaker/status", ag.getCircuitBreakerStatus)
	ag.router.POST("/circuitbreaker/:service/reset", ag.resetCircuitBreaker)

	// Gateway routing - catch-all for service routing
	ag.router.Any("/api/*path", ag.routeRequest)
	ag.router.Any("/v1/*path", ag.routeRequest)

	// Direct service routing
	ag.router.Any("/noa-core/*path", ag.createServiceProxy("noa-core"))
	ag.router.Any("/digest-agent/*path", ag.createServiceProxy("digest-agent"))
	ag.router.Any("/board-agents/*path", ag.createServiceProxy("board-agents"))
	ag.router.Any("/model-selector/*path", ag.createServiceProxy("model-selector"))
	ag.router.Any("/agent-registry/*path", ag.createServiceProxy("agent-registry"))
	ag.router.Any("/security-scanner/*path", ag.createServiceProxy("security-scanner"))
	ag.router.Any("/microagent-stacks/*path", ag.createServiceProxy("microagent-stacks"))
	ag.router.Any("/capsule-orchestrator/*path", ag.createServiceProxy("capsule-orchestrator"))
	ag.router.Any("/trifecta-court/*path", ag.createServiceProxy("trifecta-court"))

	// Metrics
	ag.router.GET("/metrics", gin.WrapH(promhttp.Handler()))
}

func (ag *APIGateway) Start() error {
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start HTTP server
	server := &http.Server{
		Addr:    ":" + ag.config.ServerPort,
		Handler: ag.router,
	}

	go func() {
		ag.logger.Infof("API Gateway listening on port %s", ag.config.ServerPort)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			ag.logger.Fatal("Failed to start server:", err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	ag.logger.Info("Shutting down API Gateway...")

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		ag.logger.Error("Server forced to shutdown:", err)
		return err
	}

	ag.logger.Info("API Gateway stopped")
	return nil
}

func (ag *APIGateway) healthCheck(c *gin.Context) {
	ag.mutex.RLock()
	defer ag.mutex.RUnlock()

	status := map[string]interface{}{
		"status":    "healthy",
		"service":   "api-gateway",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
		"services": map[string]interface{}{
			"total":     len(ag.services),
			"healthy":   ag.countServicesByStatus("healthy"),
			"unhealthy": ag.countServicesByStatus("unhealthy"),
		},
		"load_balancer": map[string]interface{}{
			"algorithm": ag.loadBalancer.algorithm,
			"requests":  ag.loadBalancer.counter,
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
	if err := ag.redis.Ping(ctx).Err(); err != nil {
		dependencies["redis"] = "unhealthy"
		status["status"] = "degraded"
	}

	// Test Database
	if err := ag.db.Ping(); err != nil {
		dependencies["database"] = "unhealthy"
		status["status"] = "degraded"
	}

	status["dependencies"] = dependencies
	c.JSON(http.StatusOK, status)
}

func (ag *APIGateway) registerService(c *gin.Context) {
	var registration ServiceRegistration
	if err := c.ShouldBindJSON(&registration); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid registration format"})
		return
	}

	// Validate with Trifecta Court
	if valid, err := ag.validateWithTrifectaCourt("register_service", map[string]interface{}{
		"name": registration.Name,
		"url":  registration.URL,
	}); err != nil || !valid {
		c.JSON(http.StatusForbidden, gin.H{"error": "Constitutional validation failed"})
		return
	}

	// Register service
	serviceID, err := ag.registerServiceEndpoint(registration)
	if err != nil {
		ag.logger.Error("Failed to register service:", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Service registration failed"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success":    true,
		"service_id": serviceID,
		"message":    "Service registered successfully",
	})
}

func (ag *APIGateway) deregisterService(c *gin.Context) {
	serviceID := c.Param("service_id")
	
	ag.mutex.Lock()
	defer ag.mutex.Unlock()

	if _, exists := ag.services[serviceID]; !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Service not found"})
		return
	}

	delete(ag.services, serviceID)

	// Remove from Redis
	ctx := context.Background()
	ag.redis.Del(ctx, fmt.Sprintf("service:%s", serviceID))

	ag.logger.Infof("Service deregistered: %s", serviceID)

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"message": "Service deregistered successfully",
	})
}

func (ag *APIGateway) listServices(c *gin.Context) {
	ag.mutex.RLock()
	defer ag.mutex.RUnlock()

	services := make([]*ServiceEndpoint, 0, len(ag.services))
	for _, service := range ag.services {
		services = append(services, service)
	}

	c.JSON(http.StatusOK, gin.H{
		"services": services,
		"total":    len(services),
	})
}

func (ag *APIGateway) getService(c *gin.Context) {
	serviceID := c.Param("service_id")
	
	ag.mutex.RLock()
	defer ag.mutex.RUnlock()

	service, exists := ag.services[serviceID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Service not found"})
		return
	}

	c.JSON(http.StatusOK, service)
}

func (ag *APIGateway) updateService(c *gin.Context) {
	serviceID := c.Param("service_id")
	
	ag.mutex.Lock()
	defer ag.mutex.Unlock()

	service, exists := ag.services[serviceID]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Service not found"})
		return
	}

	var update ServiceRegistration
	if err := c.ShouldBindJSON(&update); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid update format"})
		return
	}

	// Update service
	service.Name = update.Name
	service.URL = update.URL
	service.Weight = update.Weight
	service.Priority = update.Priority
	service.HealthCheck = update.HealthCheck
	service.Metadata = update.Metadata
	service.UpdatedAt = time.Now()

	c.JSON(http.StatusOK, service)
}

func (ag *APIGateway) discoverServices(c *gin.Context) {
	ag.mutex.RLock()
	defer ag.mutex.RUnlock()

	healthyServices := make([]*ServiceEndpoint, 0)
	for _, service := range ag.services {
		if service.Status == "healthy" {
			healthyServices = append(healthyServices, service)
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"services": healthyServices,
		"total":    len(healthyServices),
	})
}

func (ag *APIGateway) getServiceEndpoints(c *gin.Context) {
	serviceName := c.Param("service")
	
	ag.mutex.RLock()
	defer ag.mutex.RUnlock()

	endpoints := make([]*ServiceEndpoint, 0)
	for _, service := range ag.services {
		if service.Name == serviceName && service.Status == "healthy" {
			endpoints = append(endpoints, service)
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"service":   serviceName,
		"endpoints": endpoints,
		"total":     len(endpoints),
	})
}

func (ag *APIGateway) getLoadBalancerStatus(c *gin.Context) {
	status := map[string]interface{}{
		"algorithm":      ag.loadBalancer.algorithm,
		"total_requests": ag.loadBalancer.counter,
		"services":       len(ag.services),
	}

	c.JSON(http.StatusOK, status)
}

func (ag *APIGateway) updateLoadBalancerAlgorithm(c *gin.Context) {
	var request struct {
		Algorithm string `json:"algorithm"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	ag.loadBalancer.algorithm = request.Algorithm

	c.JSON(http.StatusOK, gin.H{
		"success":   true,
		"algorithm": request.Algorithm,
	})
}

func (ag *APIGateway) getRateLimitStatus(c *gin.Context) {
	ag.rateLimiter.mutex.RLock()
	defer ag.rateLimiter.mutex.RUnlock()

	c.JSON(http.StatusOK, gin.H{
		"limits": ag.rateLimiter.limits,
		"total":  len(ag.rateLimiter.limits),
	})
}

func (ag *APIGateway) updateRateLimitConfig(c *gin.Context) {
	var request struct {
		Service           string `json:"service"`
		RequestsPerSecond int    `json:"requests_per_second"`
		BurstSize         int    `json:"burst_size"`
	}
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request format"})
		return
	}

	ag.rateLimiter.mutex.Lock()
	defer ag.rateLimiter.mutex.Unlock()

	ag.rateLimiter.limits[request.Service] = &RateLimit{
		RequestsPerSecond: request.RequestsPerSecond,
		BurstSize:         request.BurstSize,
		LastReset:         time.Now(),
		RequestCount:      0,
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"service": request.Service,
		"config":  ag.rateLimiter.limits[request.Service],
	})
}

func (ag *APIGateway) getCircuitBreakerStatus(c *gin.Context) {
	ag.circuitBreaker.mutex.RLock()
	defer ag.circuitBreaker.mutex.RUnlock()

	c.JSON(http.StatusOK, gin.H{
		"circuits": ag.circuitBreaker.circuits,
		"total":    len(ag.circuitBreaker.circuits),
	})
}

func (ag *APIGateway) resetCircuitBreaker(c *gin.Context) {
	serviceName := c.Param("service")
	
	ag.circuitBreaker.mutex.Lock()
	defer ag.circuitBreaker.mutex.Unlock()

	if circuit, exists := ag.circuitBreaker.circuits[serviceName]; exists {
		circuit.State = "closed"
		circuit.FailureCount = 0
		circuit.SuccessCount = 0
		circuit.LastFailure = time.Time{}
		circuit.NextAttempt = time.Time{}
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"service": serviceName,
		"state":   "closed",
	})
}

func (ag *APIGateway) routeRequest(c *gin.Context) {
	startTime := time.Now()
	
	// Extract service from path
	pathParts := strings.Split(strings.TrimPrefix(c.Request.URL.Path, "/"), "/")
	if len(pathParts) < 2 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid path format"})
		return
	}
	
	serviceName := pathParts[1]
	
	// Check rate limiting
	if !ag.rateLimiter.checkRateLimit(serviceName) {
		c.JSON(http.StatusTooManyRequests, gin.H{"error": "Rate limit exceeded"})
		return
	}
	
	// Check circuit breaker
	if !ag.circuitBreaker.allowRequest(serviceName) {
		c.JSON(http.StatusServiceUnavailable, gin.H{"error": "Circuit breaker open"})
		return
	}
	
	// Select service endpoint
	endpoint := ag.loadBalancer.selectEndpoint(serviceName)
	if endpoint == nil {
		ag.circuitBreaker.recordFailure(serviceName)
		c.JSON(http.StatusServiceUnavailable, gin.H{"error": "No healthy endpoints available"})
		return
	}
	
	// Proxy request
	targetURL, err := url.Parse(endpoint.URL)
	if err != nil {
		ag.circuitBreaker.recordFailure(serviceName)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Invalid service URL"})
		return
	}
	
	proxy := httputil.NewSingleHostReverseProxy(targetURL)
	proxy.Director = func(req *http.Request) {
		req.URL.Scheme = targetURL.Scheme
		req.URL.Host = targetURL.Host
		req.URL.Path = strings.TrimPrefix(req.URL.Path, "/"+pathParts[0])
		req.Header.Set("X-Forwarded-Host", req.Header.Get("Host"))
		req.Header.Set("X-Forwarded-For", c.ClientIP())
		req.Header.Set("X-Gateway-Service", serviceName)
		req.Header.Set("X-Gateway-Endpoint", endpoint.ID)
	}
	
	proxy.ErrorHandler = func(w http.ResponseWriter, r *http.Request, err error) {
		ag.circuitBreaker.recordFailure(serviceName)
		ag.logger.Error("Proxy error:", err)
		w.WriteHeader(http.StatusBadGateway)
		w.Write([]byte(`{"error": "Service unavailable"}`))
	}
	
	proxy.ModifyResponse = func(resp *http.Response) error {
		if resp.StatusCode >= 500 {
			ag.circuitBreaker.recordFailure(serviceName)
		} else {
			ag.circuitBreaker.recordSuccess(serviceName)
		}
		
		resp.Header.Set("X-Gateway-Duration", fmt.Sprintf("%dms", time.Since(startTime).Milliseconds()))
		resp.Header.Set("X-Gateway-Service", serviceName)
		resp.Header.Set("X-Gateway-Endpoint", endpoint.ID)
		return nil
	}
	
	ag.loadBalancer.incrementCounter()
	proxy.ServeHTTP(c.Writer, c.Request)
}

func (ag *APIGateway) createServiceProxy(serviceName string) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Rewrite path to include service name
		originalPath := c.Request.URL.Path
		c.Request.URL.Path = "/api/" + serviceName + strings.TrimPrefix(originalPath, "/"+serviceName)
		ag.routeRequest(c)
	}
}

// Service registration and management
func (ag *APIGateway) registerServiceEndpoint(registration ServiceRegistration) (string, error) {
	serviceID := generateServiceID(registration.Name, registration.URL)
	
	ag.mutex.Lock()
	defer ag.mutex.Unlock()

	service := &ServiceEndpoint{
		ID:          serviceID,
		Name:        registration.Name,
		URL:         registration.URL,
		Status:      "healthy",
		Weight:      registration.Weight,
		Priority:    registration.Priority,
		HealthCheck: registration.HealthCheck,
		Metadata:    registration.Metadata,
		LastCheck:   time.Now(),
		Failures:    0,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}

	ag.services[serviceID] = service

	// Store in Redis
	serviceJSON, _ := json.Marshal(service)
	ctx := context.Background()
	ag.redis.Set(ctx, fmt.Sprintf("service:%s", serviceID), serviceJSON, 24*time.Hour)

	ag.logger.Infof("Service registered: %s (%s)", service.Name, serviceID)

	return serviceID, nil
}

func (ag *APIGateway) registerDefaultServices() {
	defaultServices := []ServiceRegistration{
		{
			Name:     "noa-core",
			URL:      "http://localhost:8001",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "digest-agent",
			URL:      "http://localhost:8002",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "board-agents",
			URL:      "http://localhost:8003",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "model-selector",
			URL:      "http://localhost:8004",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "agent-registry",
			URL:      "http://localhost:8005",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "security-scanner",
			URL:      "http://localhost:8007",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "microagent-stacks",
			URL:      "http://localhost:8008",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "capsule-orchestrator",
			URL:      "http://localhost:8009",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "trifecta-court",
			URL:      "http://localhost:8000",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/court/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "agent-registry-enhanced",
			URL:      "http://localhost:8009",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "model-selector-enhanced",
			URL:      "http://localhost:8010",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "canary-testing",
			URL:      "http://localhost:8011",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
		{
			Name:     "unity-harmonica",
			URL:      "http://localhost:8012",
			Weight:   100,
			Priority: 1,
			HealthCheck: &HealthCheckConfig{
				Path:     "/health",
				Interval: 30,
				Timeout:  5,
				Retries:  3,
			},
		},
	}

	for _, service := range defaultServices {
		if _, err := ag.registerServiceEndpoint(service); err != nil {
			ag.logger.Error("Failed to register default service:", err)
		}
	}
}

// Load balancer methods
func (lb *LoadBalancer) selectEndpoint(serviceName string) *ServiceEndpoint {
	lb.gateway.mutex.RLock()
	defer lb.gateway.mutex.RUnlock()

	var candidates []*ServiceEndpoint
	for _, service := range lb.gateway.services {
		if service.Name == serviceName && service.Status == "healthy" {
			candidates = append(candidates, service)
		}
	}

	if len(candidates) == 0 {
		return nil
	}

	switch lb.algorithm {
	case "round_robin":
		lb.mutex.Lock()
		index := int(lb.counter) % len(candidates)
		lb.mutex.Unlock()
		return candidates[index]
	case "weighted":
		return lb.selectWeightedEndpoint(candidates)
	default:
		return candidates[0]
	}
}

func (lb *LoadBalancer) selectWeightedEndpoint(candidates []*ServiceEndpoint) *ServiceEndpoint {
	totalWeight := 0
	for _, candidate := range candidates {
		totalWeight += candidate.Weight
	}

	if totalWeight == 0 {
		return candidates[0]
	}

	lb.mutex.Lock()
	target := int(lb.counter) % totalWeight
	lb.mutex.Unlock()

	current := 0
	for _, candidate := range candidates {
		current += candidate.Weight
		if current > target {
			return candidate
		}
	}

	return candidates[0]
}

func (lb *LoadBalancer) incrementCounter() {
	lb.mutex.Lock()
	lb.counter++
	lb.mutex.Unlock()
}

// Rate limiter methods
func (rl *RateLimiter) checkRateLimit(serviceName string) bool {
	rl.mutex.Lock()
	defer rl.mutex.Unlock()

	limit, exists := rl.limits[serviceName]
	if !exists {
		// No limit configured, allow request
		return true
	}

	now := time.Now()
	if now.Sub(limit.LastReset) >= time.Second {
		limit.RequestCount = 0
		limit.LastReset = now
	}

	if limit.RequestCount >= limit.RequestsPerSecond {
		return false
	}

	limit.RequestCount++
	return true
}

// Circuit breaker methods
func (cb *CircuitBreaker) allowRequest(serviceName string) bool {
	cb.mutex.RLock()
	defer cb.mutex.RUnlock()

	circuit, exists := cb.circuits[serviceName]
	if !exists {
		return true
	}

	switch circuit.State {
	case "open":
		if time.Now().After(circuit.NextAttempt) {
			circuit.State = "half_open"
			return true
		}
		return false
	case "half_open":
		return true
	default: // closed
		return true
	}
}

func (cb *CircuitBreaker) recordSuccess(serviceName string) {
	cb.mutex.Lock()
	defer cb.mutex.Unlock()

	circuit, exists := cb.circuits[serviceName]
	if !exists {
		circuit = &Circuit{
			State:     "closed",
			Threshold: cb.gateway.config.CircuitThreshold,
			Timeout:   60,
		}
		cb.circuits[serviceName] = circuit
	}

	circuit.SuccessCount++
	circuit.FailureCount = 0

	if circuit.State == "half_open" && circuit.SuccessCount >= 3 {
		circuit.State = "closed"
	}
}

func (cb *CircuitBreaker) recordFailure(serviceName string) {
	cb.mutex.Lock()
	defer cb.mutex.Unlock()

	circuit, exists := cb.circuits[serviceName]
	if !exists {
		circuit = &Circuit{
			State:     "closed",
			Threshold: cb.gateway.config.CircuitThreshold,
			Timeout:   60,
		}
		cb.circuits[serviceName] = circuit
	}

	circuit.FailureCount++
	circuit.LastFailure = time.Now()

	if circuit.FailureCount >= circuit.Threshold {
		circuit.State = "open"
		circuit.NextAttempt = time.Now().Add(time.Duration(circuit.Timeout) * time.Second)
	}
}

// Background tasks
func (ag *APIGateway) startHealthChecker() {
	ticker := time.NewTicker(time.Duration(ag.config.HealthCheckInterval) * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		ag.performHealthChecks()
	}
}

func (ag *APIGateway) startServiceDiscovery() {
	ticker := time.NewTicker(60 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		ag.discoverNewServices()
	}
}

func (ag *APIGateway) performHealthChecks() {
	ag.mutex.Lock()
	defer ag.mutex.Unlock()

	for _, service := range ag.services {
		if service.HealthCheck != nil {
			go ag.checkServiceHealth(service)
		}
	}
}

func (ag *APIGateway) checkServiceHealth(service *ServiceEndpoint) {
	client := &http.Client{
		Timeout: time.Duration(service.HealthCheck.Timeout) * time.Second,
	}

	healthURL := service.URL + service.HealthCheck.Path
	resp, err := client.Get(healthURL)
	
	service.LastCheck = time.Now()
	
	if err != nil || resp.StatusCode >= 400 {
		service.Failures++
		if service.Failures >= service.HealthCheck.Retries {
			service.Status = "unhealthy"
		}
	} else {
		service.Failures = 0
		service.Status = "healthy"
	}
	
	if resp != nil {
		resp.Body.Close()
	}
}

func (ag *APIGateway) discoverNewServices() {
	// Implementation for automatic service discovery
	// This could integrate with service registries like Consul, etcd, etc.
}

// Helper methods
func (ag *APIGateway) countServicesByStatus(status string) int {
	count := 0
	for _, service := range ag.services {
		if service.Status == status {
			count++
		}
	}
	return count
}

func (ag *APIGateway) validateWithTrifectaCourt(action string, context map[string]interface{}) (bool, error) {
	payload := map[string]interface{}{
		"action":  action,
		"context": context,
	}
	
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return false, err
	}
	
	resp, err := ag.trifectaCourt.client.Post(
		ag.trifectaCourt.baseURL+"/court/trifecta",
		"application/json",
		strings.NewReader(string(payloadJSON)),
	)
	if err != nil {
		ag.logger.Warn("Trifecta Court validation failed, proceeding with caution:", err)
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

func generateServiceID(name, url string) string {
	hash := sha256.Sum256([]byte(name + url + time.Now().String()))
	return hex.EncodeToString(hash[:])[:16]
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

