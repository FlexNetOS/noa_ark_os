# API Reference

## Overview

The noa_ark_os unified system exposes multiple APIs across different components. This document provides a comprehensive reference for all available APIs.

## API Architecture

```
┌─────────────────────────────────────────┐
│         API Gateway (Future)            │
│     Unified Entry Point for All APIs    │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴────────┬──────────┬──────────┐
       ▼                ▼          ▼          ▼
┌──────────┐    ┌──────────┐  ┌────────┐  ┌────────┐
│Orchestr. │    │ Hive Mind│  │File Ops│  │Pipeline│
│   API    │    │   API    │  │  API   │  │  API   │
└──────────┘    └──────────┘  └────────┘  └────────┘
```

## Base URLs

- **Orchestrator (MicroAgentStack)**: `http://localhost:8000`
- **Hive Mind (ark-os-noa)**: `http://localhost:8001`
- **File Operations (deflex-ai-os)**: `http://localhost:8002`
- **Pipeline (deflexnet-app)**: `http://localhost:8003`

## Authentication

### API Keys
```bash
# Include in headers
Authorization: Bearer YOUR_API_KEY
```

### JWT Tokens
```bash
# Obtain token
POST /auth/token
Content-Type: application/json

{
  "username": "user",
  "password": "pass"
}

# Use token
Authorization: Bearer YOUR_JWT_TOKEN
```

## MicroAgentStack API

### Agent Management

#### List Agents
```http
GET /agents
```

**Response:**
```json
{
  "agents": [
    {
      "id": "agent-001",
      "name": "Task Executor",
      "status": "active",
      "created_at": "2024-10-08T10:00:00Z"
    }
  ]
}
```

#### Create Agent
```http
POST /agents
Content-Type: application/json

{
  "name": "New Agent",
  "type": "executor",
  "config": {
    "capabilities": ["code", "analysis"]
  }
}
```

#### Get Agent Details
```http
GET /agents/{agent_id}
```

#### Update Agent
```http
PUT /agents/{agent_id}
Content-Type: application/json

{
  "status": "paused"
}
```

#### Delete Agent
```http
DELETE /agents/{agent_id}
```

### Task Management

#### Submit Task
```http
POST /tasks
Content-Type: application/json

{
  "type": "code_analysis",
  "payload": {
    "repository": "https://github.com/user/repo",
    "branch": "main"
  },
  "priority": "high"
}
```

**Response:**
```json
{
  "task_id": "task-12345",
  "status": "queued",
  "created_at": "2024-10-08T10:00:00Z"
}
```

#### Get Task Status
```http
GET /tasks/{task_id}
```

**Response:**
```json
{
  "task_id": "task-12345",
  "status": "running",
  "progress": 45,
  "assigned_to": "agent-001",
  "started_at": "2024-10-08T10:01:00Z"
}
```

#### List Tasks
```http
GET /tasks?status=running&limit=10
```

### Health and Metrics

#### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": 86400
}
```

#### Metrics
```http
GET /metrics
```

**Response:**
```json
{
  "active_agents": 5,
  "queued_tasks": 10,
  "completed_tasks": 1000,
  "failed_tasks": 5
}
```

## Hive Mind API

### Swarm Coordination

#### Get Swarm Status
```http
GET /swarm/status
```

**Response:**
```json
{
  "members": 10,
  "consensus": "ready",
  "leader": "agent-001"
}
```

#### Submit Collective Task
```http
POST /swarm/tasks
Content-Type: application/json

{
  "task": "distributed_analysis",
  "requirements": {
    "agents": 5,
    "capabilities": ["analysis", "reporting"]
  }
}
```

### Knowledge Base

#### Query Knowledge
```http
GET /knowledge?query=python+optimization
```

#### Add Knowledge
```http
POST /knowledge
Content-Type: application/json

{
  "topic": "rust_optimization",
  "content": "Best practices for Rust optimization...",
  "tags": ["rust", "performance"]
}
```

## File Operations API

### File Management

#### List Files
```http
GET /files?path=/workspace
```

#### Upload File
```http
POST /files
Content-Type: multipart/form-data

file: [binary data]
path: /workspace/newfile.txt
```

#### Download File
```http
GET /files/{file_id}/download
```

#### Delete File
```http
DELETE /files/{file_id}
```

### AI Operations

#### Analyze File
```http
POST /ai/analyze
Content-Type: application/json

{
  "file_id": "file-123",
  "analysis_type": "content_classification"
}
```

#### Organize Files
```http
POST /ai/organize
Content-Type: application/json

{
  "path": "/workspace",
  "strategy": "by_type"
}
```

## Pipeline API

### Pipeline Execution

#### Start Pipeline
```http
POST /pipelines/run
Content-Type: application/json

{
  "pipeline": "digest_pipeline",
  "input": {
    "source": "/data/input",
    "format": "json"
  }
}
```

**Response:**
```json
{
  "run_id": "run-789",
  "status": "started",
  "estimated_duration": 300
}
```

#### Get Pipeline Status
```http
GET /pipelines/runs/{run_id}
```

#### Stop Pipeline
```http
POST /pipelines/runs/{run_id}/stop
```

### Pipeline Configuration

#### List Pipelines
```http
GET /pipelines
```

#### Get Pipeline Config
```http
GET /pipelines/{pipeline_id}/config
```

## WebSocket APIs

### Real-time Agent Updates
```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8000/ws/agents');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Agent update:', update);
};

// Subscribe to specific agent
ws.send(JSON.stringify({
  action: 'subscribe',
  agent_id: 'agent-001'
}));
```

### Task Progress Stream
```javascript
const ws = new WebSocket('ws://localhost:8000/ws/tasks/task-12345');

ws.onmessage = (event) => {
  const progress = JSON.parse(event.data);
  console.log('Progress:', progress.percentage);
};
```

## Error Handling

### Standard Error Response
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input parameters",
    "details": {
      "field": "agent_type",
      "issue": "Must be one of: executor, analyzer, coordinator"
    }
  }
}
```

### HTTP Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error
- `503 Service Unavailable` - Service temporarily unavailable

## Rate Limiting

- **Default**: 100 requests per minute per API key
- **Burst**: 200 requests
- **Headers**:
  ```
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 95
  X-RateLimit-Reset: 1696780800
  ```

## Pagination

List endpoints support pagination:

```http
GET /agents?page=2&per_page=20
```

**Response includes:**
```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

## Filtering and Sorting

```http
GET /tasks?status=completed&sort=-created_at&limit=50
```

Operators:
- `-` prefix for descending sort
- Multiple filters with `&`

## Webhooks

### Register Webhook
```http
POST /webhooks
Content-Type: application/json

{
  "url": "https://your-server.com/webhook",
  "events": ["task.completed", "agent.failed"],
  "secret": "your-webhook-secret"
}
```

### Webhook Payload
```json
{
  "event": "task.completed",
  "timestamp": "2024-10-08T10:00:00Z",
  "data": {
    "task_id": "task-12345",
    "result": {...}
  }
}
```

## GraphQL API (Future)

```graphql
query {
  agents(status: ACTIVE) {
    id
    name
    tasks {
      id
      status
    }
  }
}
```

## SDK Examples

### Python
```python
from noa_ark_os import Client

client = Client(api_key='YOUR_API_KEY')

# Create agent
agent = client.agents.create(
    name='Code Analyzer',
    type='analyzer'
)

# Submit task
task = client.tasks.submit(
    type='code_analysis',
    payload={'repo': 'https://github.com/user/repo'}
)

# Wait for completion
result = task.wait()
print(result)
```

### Rust
```rust
use noa_ark_os::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("YOUR_API_KEY");
    
    let agent = client.agents().create()
        .name("Code Analyzer")
        .agent_type("analyzer")
        .send()
        .await?;
        
    println!("Created agent: {}", agent.id);
}
```

### JavaScript/TypeScript
```typescript
import { NoaArkClient } from '@noa-ark-os/client';

const client = new NoaArkClient({
  apiKey: 'YOUR_API_KEY'
});

// Create agent
const agent = await client.agents.create({
  name: 'Code Analyzer',
  type: 'analyzer'
});

// Submit task
const task = await client.tasks.submit({
  type: 'code_analysis',
  payload: { repo: 'https://github.com/user/repo' }
});
```

## OpenAPI Specification

Full OpenAPI 3.0 specifications available at:
- Orchestrator: `http://localhost:8000/openapi.json`
- Documentation UI: `http://localhost:8000/docs`
- ReDoc UI: `http://localhost:8000/redoc`

## Support

For API questions:
- Check inline API documentation at `/docs`
- Review examples in `/examples`
- Open issue on GitHub
- Contact API support team

## Changelog

### v1.0.0 (2024-10-08)
- Initial unified API release
- Core endpoints for all components
- WebSocket support
- Webhook system
