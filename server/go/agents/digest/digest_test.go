package main

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	sqlmock "github.com/DATA-DOG/go-sqlmock"
	"github.com/alicebob/miniredis/v2"
	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"github.com/sirupsen/logrus"
)

func TestDigestEngineExtractKnowledgeFromChunk(t *testing.T) {
	var receivedPayload KnowledgeUpsertPayload

	graphServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/graph/upsert" {
			t.Fatalf("unexpected path: %s", r.URL.Path)
		}
		if err := json.NewDecoder(r.Body).Decode(&receivedPayload); err != nil {
			t.Fatalf("failed to decode payload: %v", err)
		}
		_, _ = w.Write([]byte(`{"nodes_created":1,"edges_created":1}`))
	}))
	defer graphServer.Close()

	logger := logrus.New()
	agent := &DigestAgent{
		logger: logger,
		knowledgeGraph: &KnowledgeGraphClient{
			baseURL: graphServer.URL,
			client:  graphServer.Client(),
		},
	}
	engine := &DigestEngine{agent: agent}
	agent.digestEngine = engine

	result, err := engine.extractKnowledgeFromChunk("example chunk text", "digest-123")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.NodesCreated != 1 || result.EdgesCreated != 1 {
		t.Fatalf("unexpected result counts: %+v", result)
	}

	if len(result.Provenance) != 1 {
		t.Fatalf("expected provenance entry, got %d", len(result.Provenance))
	}

	if receivedPayload.Nodes[0].DigestID != "digest-123" {
		t.Fatalf("expected digest id to propagate, got %s", receivedPayload.Nodes[0].DigestID)
	}

	if receivedPayload.Edges[0].SourceID != "digest:digest-123" {
		t.Fatalf("expected digest edge source, got %s", receivedPayload.Edges[0].SourceID)
	}
}

func TestResearchServiceCreateAndComplete(t *testing.T) {
	gin.SetMode(gin.TestMode)

	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}
	defer db.Close()

	mock.ExpectExec("CREATE TABLE IF NOT EXISTS research_insights").WillReturnResult(sqlmock.NewResult(0, 0))

	redisServer, err := miniredis.Run()
	if err != nil {
		t.Fatalf("failed to start miniredis: %v", err)
	}
	defer redisServer.Close()

	redisClient := redis.NewClient(&redis.Options{Addr: redisServer.Addr()})
	service, err := NewResearchService(db, redisClient, logrus.New())
	if err != nil {
		t.Fatalf("failed to init research service: %v", err)
	}

	mock.ExpectExec("INSERT INTO research_insights").
		WithArgs(sqlmock.AnyArg(), "test query", sqlmock.AnyArg(), sqlmock.AnyArg(), sqlmock.AnyArg(), "", sqlmock.AnyArg(), "pending", sqlmock.AnyArg()).
		WillReturnResult(sqlmock.NewResult(1, 1))

	response, err := service.CreateResearchRequest(context.Background(), ResearchQueryRequest{
		Query:       "test query",
		DataSources: []string{"alpha"},
		Metadata:    map[string]interface{}{"team": "atlas"},
	})
	if err != nil {
		t.Fatalf("failed to create research request: %v", err)
	}

	if response.Status != "pending" {
		t.Fatalf("expected pending status, got %s", response.Status)
	}

	statusRaw, err := redisServer.Get("research:status:" + response.ResearchID)
	if err != nil {
		t.Fatalf("expected status cached: %v", err)
	}
	var statusPayload map[string]interface{}
	if err := json.Unmarshal([]byte(statusRaw), &statusPayload); err != nil {
		t.Fatalf("failed to unmarshal status cache: %v", err)
	}
	if statusPayload["status"].(string) != "pending" {
		t.Fatalf("expected cached pending status, got %v", statusPayload["status"])
	}

	mock.ExpectExec("UPDATE research_insights").
		WithArgs(sqlmock.AnyArg(), "summary", sqlmock.AnyArg(), "completed", sqlmock.AnyArg(), response.ResearchID).
		WillReturnResult(sqlmock.NewResult(1, 1))

	insightsResponse, err := service.CompleteResearchRequest(context.Background(), ResearchInsightsRequest{
		ResearchID: response.ResearchID,
		Insights:   []string{"insight-1"},
		Summary:    "summary",
		Metadata:   map[string]interface{}{"team": "atlas"},
	})
	if err != nil {
		t.Fatalf("failed to complete research request: %v", err)
	}

	if insightsResponse.Status != "completed" {
		t.Fatalf("expected completed status, got %s", insightsResponse.Status)
	}

	insightsCached, err := redisServer.Get("research:insights:" + response.ResearchID)
	if err != nil {
		t.Fatalf("expected insights cached: %v", err)
	}
	var insightsPayload map[string]interface{}
	if err := json.Unmarshal([]byte(insightsCached), &insightsPayload); err != nil {
		t.Fatalf("failed to unmarshal insights cache: %v", err)
	}
	if insightsPayload["summary"].(string) != "summary" {
		t.Fatalf("expected cached summary, got %v", insightsPayload["summary"])
	}

	if err := mock.ExpectationsWereMet(); err != nil {
		t.Fatalf("unmet sql expectations: %v", err)
	}
}

func TestResearchQueryTrifectaValidation(t *testing.T) {
	gin.SetMode(gin.TestMode)

	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}
	defer db.Close()

	mock.ExpectExec("CREATE TABLE IF NOT EXISTS research_insights").WillReturnResult(sqlmock.NewResult(0, 0))

	redisServer, err := miniredis.Run()
	if err != nil {
		t.Fatalf("failed to start miniredis: %v", err)
	}
	defer redisServer.Close()

	redisClient := redis.NewClient(&redis.Options{Addr: redisServer.Addr()})
	researchService, err := NewResearchService(db, redisClient, logrus.New())
	if err != nil {
		t.Fatalf("failed to init research service: %v", err)
	}

	trifecta := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"valid":false}`))
	}))
	defer trifecta.Close()

	agent := &DigestAgent{
		logger: logrus.New(),
		redis:  redisClient,
		db:     db,
		config: &Config{ChunkSize: 10},
		trifectaCourt: &TrifectaCourtClient{
			baseURL: trifecta.URL,
			client:  trifecta.Client(),
		},
		researchService: researchService,
	}
	agent.digestEngine = &DigestEngine{agent: agent}

	recorder := httptest.NewRecorder()
	requestBody := `{"query":"blocked","data_sources":["alpha"]}`
	req, _ := http.NewRequest(http.MethodPost, "/research/query", strings.NewReader(requestBody))
	req.Header.Set("Content-Type", "application/json")

	ctx, _ := gin.CreateTestContext(recorder)
	ctx.Request = req

	agent.researchQuery(ctx)

	if recorder.Code != http.StatusForbidden {
		t.Fatalf("expected forbidden status, got %d", recorder.Code)
	}

	if err := mock.ExpectationsWereMet(); err != nil {
		t.Fatalf("unmet sql expectations: %v", err)
	}
}

func TestProcessAnalysisCachesStatus(t *testing.T) {
	redisServer, err := miniredis.Run()
	if err != nil {
		t.Fatalf("failed to start miniredis: %v", err)
	}
	defer redisServer.Close()

	redisClient := redis.NewClient(&redis.Options{Addr: redisServer.Addr()})
	agent := &DigestAgent{
		logger: logrus.New(),
		redis:  redisClient,
		config: &Config{ChunkSize: 10},
	}
	engine := &DigestEngine{agent: agent}
	agent.digestEngine = engine

	response, err := engine.ProcessAnalysis(AnalysisRequest{
		Query:        "what is the trend?",
		DataSources:  []string{"alpha", "beta"},
		AnalysisType: "trend",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	statusRaw, err := redisServer.Get("analysis:status:" + response.AnalysisID)
	if err != nil {
		t.Fatalf("expected analysis status cached: %v", err)
	}

	var statusPayload map[string]interface{}
	if err := json.Unmarshal([]byte(statusRaw), &statusPayload); err != nil {
		t.Fatalf("failed to unmarshal cached status: %v", err)
	}

	metadata := statusPayload["metadata"].(map[string]interface{})
	if metadata["status_ref"].(string) != response.StatusRef {
		t.Fatalf("expected matching status ref, got %s", metadata["status_ref"].(string))
	}
}
