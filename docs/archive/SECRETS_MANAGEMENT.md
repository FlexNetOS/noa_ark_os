# Environment Variables and Secrets Management

## Overview

NOA ARK OS uses a layered approach to environment configuration and secrets management, ensuring security, flexibility, and ease of deployment.

## Storage Locations

### 1. Environment Files

```
noa_ark_os/
├── .env.example .......................... Template (committed)
├── .env.local ............................ Local overrides (gitignored)
├── .env.development ...................... Development (gitignored)
├── .env.staging .......................... Staging (gitignored)
├── .env.production ....................... Production (gitignored)
│
├── server/
│   ├── .env.example ...................... Server template
│   └── config/
│       ├── default.toml .................. Default config
│       ├── dev.toml ...................... Dev config
│       ├── staging.toml .................. Staging config
│       └── prod.toml ..................... Production config
│
└── .secrets/ ............................. Encrypted secrets (gitignored)
    ├── development/
    │   ├── jwt.key ....................... JWT signing key
    │   ├── tls.crt ....................... TLS certificate
    │   ├── tls.key ....................... TLS private key
    │   └── api.keys ...................... API keys
    ├── staging/
    │   └── ... (same structure)
    └── production/
        └── ... (same structure)
```

### 2. Secrets Management Tools

```
.secrets/
├── manager.yaml .......................... Secrets manager config
├── vault/ ................................ HashiCorp Vault integration
│   ├── config.hcl ........................ Vault configuration
│   └── policies/ ......................... Vault policies
├── aws/ .................................. AWS Secrets Manager
│   └── config.json ....................... AWS SM configuration
├── azure/ ................................ Azure Key Vault
│   └── config.json ....................... Azure KV configuration
└── docker/ ............................... Docker secrets
    └── secrets.yaml ...................... Docker secret definitions
```

## Environment Variable Structure

### Root `.env.example`

```bash
# NOA ARK OS - Environment Configuration Template
# Copy to .env.local and customize

# ============================================================================
# CORE SYSTEM
# ============================================================================
NOA_ENV=development                      # development | staging | production
NOA_LOG_LEVEL=info                       # debug | info | warn | error
NOA_LOG_FORMAT=json                      # json | text

# ============================================================================
# WORKSPACE
# ============================================================================
NOA_WORKSPACE_PATH=./
NOA_WORKSPACE_BACKUP_ENABLED=true
NOA_WORKSPACE_CLEANUP_SCHEDULE="0 2 * * *"  # Daily at 2 AM

# ============================================================================
# CRC SYSTEM
# ============================================================================
NOA_CRC_DROP_IN_PATH=./crc/drop-in
NOA_CRC_ARCHIVE_PATH=./crc/archive
NOA_CRC_AUTO_APPROVE_THRESHOLD=0.95
NOA_CRC_COMPRESSION_LEVEL=3

# ============================================================================
# CI/CD
# ============================================================================
NOA_CICD_ENABLED=true
NOA_CICD_AUTO_DEPLOY_STAGING=true
NOA_CICD_AUTO_DEPLOY_PRODUCTION=false
NOA_CICD_HEALTH_CHECK_TIMEOUT=300        # seconds
NOA_CICD_ROLLBACK_ENABLED=true

# ============================================================================
# SERVER
# ============================================================================
NOA_SERVER_HOST=0.0.0.0
NOA_SERVER_PORT=8080
NOA_SERVER_WORKERS=4
NOA_SERVER_TLS_ENABLED=false

# ============================================================================
# DATABASE
# ============================================================================
NOA_DATABASE_URL=postgresql://localhost:5432/noa
NOA_DATABASE_MAX_CONNECTIONS=20
NOA_DATABASE_MIN_CONNECTIONS=5

# ============================================================================
# CACHE
# ============================================================================
NOA_CACHE_URL=redis://localhost:6379
NOA_CACHE_POOL_SIZE=10
NOA_CACHE_TTL=3600                       # seconds

# ============================================================================
# VECTOR DATABASE
# ============================================================================
NOA_QDRANT_URL=http://localhost:6333
NOA_QDRANT_API_KEY=                      # Set in .env.local
NOA_QDRANT_COLLECTION=noa_embeddings

# ============================================================================
# AI/INFERENCE
# ============================================================================
NOA_INFERENCE_DEVICE=auto                # cpu | cuda | auto
NOA_INFERENCE_MODEL_PATH=./ai/models
NOA_INFERENCE_MAX_BATCH_SIZE=32

# ============================================================================
# OBSERVABILITY
# ============================================================================
NOA_METRICS_ENABLED=true
NOA_METRICS_PORT=9090
NOA_TRACING_ENABLED=true
NOA_TRACING_ENDPOINT=http://localhost:4317  # OTLP endpoint

# ============================================================================
# EXTERNAL INTEGRATIONS
# ============================================================================

# GitHub
NOA_GITHUB_ENABLED=false
NOA_GITHUB_TOKEN=                        # Set in .env.local
NOA_GITHUB_ORG=

# AWS
NOA_AWS_ENABLED=false
NOA_AWS_ACCESS_KEY_ID=                   # Set in .env.local
NOA_AWS_SECRET_ACCESS_KEY=               # Set in .env.local
NOA_AWS_REGION=us-east-1

# Azure
NOA_AZURE_ENABLED=false
NOA_AZURE_CLIENT_ID=                     # Set in .env.local
NOA_AZURE_CLIENT_SECRET=                 # Set in .env.local
NOA_AZURE_TENANT_ID=                     # Set in .env.local

# Docker
NOA_DOCKER_ENABLED=true
NOA_DOCKER_REGISTRY=
NOA_DOCKER_USERNAME=
NOA_DOCKER_PASSWORD=                     # Set in .env.local

# Kubernetes
NOA_K8S_ENABLED=false
NOA_K8S_CONTEXT=
NOA_K8S_NAMESPACE=noa-ark-os

# NPM
NOA_NPM_ENABLED=false
NOA_NPM_TOKEN=                           # Set in .env.local

# Cloudflare
NOA_CLOUDFLARE_ENABLED=false
NOA_CLOUDFLARE_TUNNEL_TOKEN=             # Set in .env.local
NOA_CLOUDFLARE_ZONE_ID=

# Caddy
NOA_CADDY_ENABLED=false
NOA_CADDY_ADMIN_ENDPOINT=http://localhost:2019
NOA_CADDY_AUTO_HTTPS=true

# Llama.cpp
NOA_LLAMA_CPP_ENABLED=false
NOA_LLAMA_CPP_MODEL_PATH=./ai/models
NOA_LLAMA_CPP_CONTEXT_SIZE=4096
NOA_LLAMA_CPP_GPU_LAYERS=35

# VS Code
NOA_VSCODE_EXTENSION_ENABLED=false
NOA_VSCODE_EXTENSION_PORT=9000

# ============================================================================
# SECURITY
# ============================================================================
NOA_JWT_SECRET=                          # Set in .env.local (REQUIRED)
NOA_JWT_EXPIRY=3600                      # seconds
NOA_ENCRYPTION_KEY=                      # Set in .env.local (REQUIRED)
NOA_TLS_CERT_PATH=./.secrets/tls.crt
NOA_TLS_KEY_PATH=./.secrets/tls.key

# ============================================================================
# FEATURE FLAGS
# ============================================================================
NOA_FEATURE_CRC_ENABLED=true
NOA_FEATURE_SANDBOX_MODELS=true
NOA_FEATURE_AUTO_APPROVE=true
NOA_FEATURE_HIVE_MIND=true
NOA_FEATURE_SWARM_COORDINATION=true
NOA_FEATURE_SELF_HOSTED_PRIORITY=true
```

### Production `.env.production` (Example)

```bash
# Production Environment - DO NOT COMMIT
NOA_ENV=production
NOA_LOG_LEVEL=warn
NOA_LOG_FORMAT=json

# Server
NOA_SERVER_HOST=0.0.0.0
NOA_SERVER_PORT=443
NOA_SERVER_TLS_ENABLED=true

# Database (Use secrets manager)
NOA_DATABASE_URL=${SECRET_DATABASE_URL}
NOA_DATABASE_MAX_CONNECTIONS=100

# Cache
NOA_CACHE_URL=${SECRET_REDIS_URL}

# Security
NOA_JWT_SECRET=${SECRET_JWT_SECRET}
NOA_ENCRYPTION_KEY=${SECRET_ENCRYPTION_KEY}

# AWS (Production)
NOA_AWS_ENABLED=true
NOA_AWS_ACCESS_KEY_ID=${SECRET_AWS_KEY}
NOA_AWS_SECRET_ACCESS_KEY=${SECRET_AWS_SECRET}

# GitHub
NOA_GITHUB_ENABLED=true
NOA_GITHUB_TOKEN=${SECRET_GITHUB_TOKEN}
```

## Secrets Manager Configuration

### HashiCorp Vault

**`.secrets/vault/config.hcl`**:
```hcl
storage "file" {
  path = ".secrets/vault/data"
}

listener "tcp" {
  address     = "127.0.0.1:8200"
  tls_disable = 1
}

api_addr = "http://127.0.0.1:8200"
cluster_addr = "https://127.0.0.1:8201"
ui = true

default_lease_ttl = "168h"
max_lease_ttl = "720h"
```

**Vault Integration**:
```rust
// server/src/secrets/vault.rs
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

pub struct VaultSecretManager {
    client: VaultClient,
}

impl VaultSecretManager {
    pub async fn new() -> Result<Self, Error> {
        let settings = VaultClientSettingsBuilder::default()
            .address("http://127.0.0.1:8200")
            .token(std::env::var("VAULT_TOKEN")?)
            .build()?;
        
        let client = VaultClient::new(settings)?;
        Ok(Self { client })
    }
    
    pub async fn get_secret(&self, path: &str) -> Result<String, Error> {
        let secret = self.client
            .kv2("secret", Some("noa"))
            .read(path)
            .await?;
        
        Ok(secret.data["value"].as_str().unwrap().to_string())
    }
}
```

### AWS Secrets Manager

**`.secrets/aws/config.json`**:
```json
{
  "region": "us-east-1",
  "secrets": {
    "database_url": "noa/prod/database/url",
    "jwt_secret": "noa/prod/jwt/secret",
    "encryption_key": "noa/prod/encryption/key",
    "github_token": "noa/prod/github/token"
  }
}
```

**AWS SM Integration**:
```rust
// server/src/secrets/aws.rs
use aws_sdk_secretsmanager as secretsmanager;

pub struct AWSSecretManager {
    client: secretsmanager::Client,
}

impl AWSSecretManager {
    pub async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let client = secretsmanager::Client::new(&config);
        Ok(Self { client })
    }
    
    pub async fn get_secret(&self, secret_id: &str) -> Result<String, Error> {
        let response = self.client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await?;
        
        Ok(response.secret_string().unwrap().to_string())
    }
}
```

### Azure Key Vault

**`.secrets/azure/config.json`**:
```json
{
  "vaultUrl": "https://noa-vault.vault.azure.net/",
  "secrets": {
    "database-url": "DatabaseUrl",
    "jwt-secret": "JwtSecret",
    "encryption-key": "EncryptionKey"
  }
}
```

## Docker Secrets

**`.secrets/docker/secrets.yaml`**:
```yaml
version: '3.8'

secrets:
  database_url:
    external: true
  jwt_secret:
    external: true
  encryption_key:
    external: true
  github_token:
    external: true
  aws_access_key:
    external: true
  aws_secret_key:
    external: true

services:
  noa-server:
    image: noa-unified-server:latest
    secrets:
      - database_url
      - jwt_secret
      - encryption_key
    environment:
      NOA_DATABASE_URL_FILE: /run/secrets/database_url
      NOA_JWT_SECRET_FILE: /run/secrets/jwt_secret
      NOA_ENCRYPTION_KEY_FILE: /run/secrets/encryption_key
```

## Kubernetes Secrets

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: noa-secrets
  namespace: noa-ark-os
type: Opaque
stringData:
  database-url: postgresql://...
  jwt-secret: ...
  encryption-key: ...
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: noa-config
  namespace: noa-ark-os
data:
  NOA_ENV: "production"
  NOA_LOG_LEVEL: "info"
  NOA_SERVER_PORT: "8080"
```

## Environment Loading Priority

1. **Environment variables** (highest priority)
2. **`.env.{environment}` file** (e.g., `.env.production`)
3. **`.env.local` file**
4. **`.env` file**
5. **Default values in code** (lowest priority)

## Security Best Practices

### 1. Never Commit Secrets

**`.gitignore`** entries:
```
.env.local
.env.development
.env.staging
.env.production
.secrets/
*.key
*.pem
*.crt
*.p12
```

### 2. Rotate Secrets Regularly

```bash
# Rotate JWT secret
noa secrets rotate jwt-secret

# Rotate encryption key
noa secrets rotate encryption-key

# Rotate database password
noa secrets rotate database-password
```

### 3. Use Secret Scanning

```yaml
# .github/workflows/secret-scan.yml
name: Secret Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run gitleaks
        uses: gitleaks/gitleaks-action@v2
```

### 4. Encrypt Secrets at Rest

```rust
// server/src/secrets/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub fn encrypt_secret(plaintext: &str, key: &[u8]) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| Error::EncryptionFailed(e.to_string()))
}
```

## Access Control

### Secrets Manager Roles

```yaml
roles:
  developer:
    secrets:
      read:
        - development/*
      write: []
  
  devops:
    secrets:
      read:
        - development/*
        - staging/*
      write:
        - staging/*
  
  admin:
    secrets:
      read:
        - "*"
      write:
        - "*"
```

## Monitoring and Auditing

```rust
// Log all secret access
#[instrument(skip(secret_manager))]
pub async fn access_secret(
    secret_manager: &SecretManager,
    secret_name: &str,
    user: &str,
) -> Result<String, Error> {
    info!(
        secret = secret_name,
        user = user,
        "Accessing secret"
    );
    
    let secret = secret_manager.get(secret_name).await?;
    
    // Audit log
    audit::log_secret_access(secret_name, user).await;
    
    Ok(secret)
}
```

## CLI Commands

```bash
# Set secret
noa secrets set <name> <value>

# Get secret
noa secrets get <name>

# List secrets
noa secrets list

# Delete secret
noa secrets delete <name>

# Rotate secret
noa secrets rotate <name>

# Sync secrets (from Vault/AWS/Azure)
noa secrets sync

# Validate secrets
noa secrets validate
```

## Integration Examples

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
env:
  NOA_DATABASE_URL: ${{ secrets.DATABASE_URL }}
  NOA_JWT_SECRET: ${{ secrets.JWT_SECRET }}
  NOA_GITHUB_TOKEN: ${{ secrets.GH_TOKEN }}
```

### Docker Compose

```yaml
services:
  noa-server:
    environment:
      - NOA_DATABASE_URL=${DATABASE_URL}
      - NOA_JWT_SECRET=${JWT_SECRET}
    env_file:
      - .env.production
```

### Kubernetes

```yaml
spec:
  containers:
    - name: noa-server
      envFrom:
        - configMapRef:
            name: noa-config
        - secretRef:
            name: noa-secrets
```

## Troubleshooting

### Secret Not Found

```bash
# Check if secret exists
noa secrets get <name>

# Sync from secrets manager
noa secrets sync

# Recreate secret
noa secrets set <name> <value>
```

### Permission Denied

```bash
# Check permissions
noa secrets permissions <name>

# Request access
noa secrets request-access <name>
```

### Decryption Failed

```bash
# Verify encryption key
noa secrets validate-key

# Rotate encryption key
noa secrets rotate encryption-key
```
