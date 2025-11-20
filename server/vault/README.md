# HashiCorp Vault Secret Management

Production-grade secret storage and management using Vault's Integrated Storage (Raft) mode.

## Overview

This directory contains HashiCorp Vault configurations for secure secret management in NoaArkOS. Vault provides:

- **Secret Storage** - Encrypted key-value storage for credentials, API keys, certificates
- **Dynamic Secrets** - Generate credentials on-demand for databases, cloud providers
- **Encryption as a Service** - Encrypt/decrypt data without storing it
- **Access Control** - Fine-grained policies for secret access
- **Audit Logging** - Complete audit trail of all secret access
- **High Availability** - Raft consensus for multi-node deployments

## Files

- `vault.hcl` - Main Vault configuration (Raft storage, single-node)
- `start.sh` - Startup script (Linux/WSL)
- `status.sh` - Status checker (Linux/WSL)
- `stop.sh` - Graceful shutdown script (Linux/WSL)

## Installation

### Windows

Download Vault from https://www.vaultproject.io/downloads

```powershell
# Download and extract
$vaultVersion = "1.15.4"
$vaultUrl = "https://releases.hashicorp.com/vault/${vaultVersion}/vault_${vaultVersion}_windows_amd64.zip"
Invoke-WebRequest -Uri $vaultUrl -OutFile "vault.zip"
Expand-Archive -Path "vault.zip" -DestinationPath "server/tools/bin/"
Remove-Item "vault.zip"

# Verify installation
& server/tools/bin/vault.exe version
```

### WSL/Linux

```bash
# Via package manager (Ubuntu/Debian)
wget -O- https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt update && sudo apt install vault

# Verify
vault version
```

## Configuration

### vault.hcl

Single-node Raft configuration optimized for local development:

```hcl
ui = true                # Enable web UI
disable_mlock = true     # For WSL/container compatibility

listener "tcp" {
  address     = "127.0.0.1:8200"  # Loopback only
  tls_disable = 1                  # Disable TLS (use Caddy for TLS termination)
}

storage "raft" {
  path    = "{{env \"NOA_VAULT_HOME\"}}/data"
  node_id = "vault-node-1"
}

api_addr     = "http://127.0.0.1:8200"
cluster_addr = "https://127.0.0.1:8201"

telemetry {
  disable_hostname = true
}
```

**Key Settings:**
- **NOA_VAULT_HOME:** Gateway-managed base directory for Vault data/config (defaults to `~/.noa/vault`). Overrides allow each operator to keep data outside the repo while keeping manifests consistent.
- **UI:** Enabled for browser-based management
- **Listener:** Bound to loopback (127.0.0.1:8200) - NOT exposed externally
- **TLS:** Disabled (use Caddy reverse proxy for TLS termination)
- **Storage:** Raft-based integrated storage (no external database needed)
- **mlock:** Disabled for WSL compatibility (enable on hardened hosts)

Before starting Vault, export/confirm `NOA_VAULT_HOME` and create its directories:

```bash
export NOA_VAULT_HOME=${NOA_VAULT_HOME:-"$HOME/.noa/vault"}
mkdir -p "$NOA_VAULT_HOME/data" "$NOA_VAULT_HOME/runtime"
```

### Production Adjustments

For production deployment, modify `vault.hcl`:

```hcl
ui = false  # Disable UI in production
disable_mlock = false  # Enable mlock for memory protection

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 0
  tls_cert_file = "/path/to/cert.pem"
  tls_key_file  = "/path/to/key.pem"
}

# Multi-node cluster
storage "raft" {
  path    = "D:/dev/workspaces/noa_ark_os/server/vault/data"
  node_id = "vault-node-1"
  
  retry_join {
    leader_api_addr = "https://vault-node-2:8200"
  }
  retry_join {
    leader_api_addr = "https://vault-node-3:8200"
  }
}
```

## Usage

### First-Time Setup

#### 1. Start Vault Server

**Windows PowerShell:**
```powershell
# Create data directory
New-Item -ItemType Directory -Force -Path "server/vault/data"

# Start Vault in development mode (NOT for production)
& server/tools/bin/vault.exe server -dev

# OR: Start with production config
& server/tools/bin/vault.exe server -config=server/vault/vault.hcl
```

**WSL/Linux:**
```bash
# Using provided scripts
export NOA_VAULT_HOME=${NOA_VAULT_HOME:-"$HOME/.noa/vault"}
./server/vault/start.sh

# OR: Direct command
vault server -config="$(pwd)/server/vault/vault.hcl"
```

#### 1b. Configure Gateway Auth Runtime

The helper script keeps `GATEWAY_AUTH_CONFIG` pointed at the managed runtime directory and seeds it from the repository copy if missing:

```bash
export NOA_VAULT_HOME=${NOA_VAULT_HOME:-"$HOME/.noa/vault"}
./server/vault/configure-gateway-auth-simple.sh
```

Override the managed location by exporting `NOA_VAULT_HOME=/custom/path` before running the script.

#### 2. Initialize Vault (First Time Only)

```powershell
# Set Vault address
$env:VAULT_ADDR = "http://127.0.0.1:8200"

# Initialize (generates root token and unseal keys)
& server/tools/bin/vault.exe operator init

# OUTPUT (SAVE THIS SECURELY):
# Unseal Key 1: <key1>
# Unseal Key 2: <key2>
# Unseal Key 3: <key3>
# Unseal Key 4: <key4>
# Unseal Key 5: <key5>
# Initial Root Token: <root-token>
```

**⚠️ CRITICAL: Store unseal keys and root token offline (encrypted USB, password manager, etc.)**

#### 3. Unseal Vault

Vault starts sealed - requires 3 of 5 unseal keys:

```powershell
# Unseal (repeat 3 times with different keys)
& server/tools/bin/vault.exe operator unseal <key1>
& server/tools/bin/vault.exe operator unseal <key2>
& server/tools/bin/vault.exe operator unseal <key3>

# Verify unsealed
& server/tools/bin/vault.exe status
```

#### 4. Authenticate

```powershell
# Login with root token
& server/tools/bin/vault.exe login <root-token>

# Verify authentication
& server/tools/bin/vault.exe token lookup
```

### Daily Operations

#### Start Vault

**Windows Service (see Installation as Service section):**
```powershell
Start-Service VaultService
```

**Manual:**
```powershell
& server/tools/bin/vault.exe server -config=server/vault/vault.hcl
```

#### Check Status

**PowerShell:**
```powershell
$env:VAULT_ADDR = "http://127.0.0.1:8200"
& server/tools/bin/vault.exe status
```

**WSL:**
```bash
./server/vault/status.sh
```

#### Stop Vault

**Windows Service:**
```powershell
Stop-Service VaultService
```

**Manual (Ctrl+C in server terminal)**

**WSL:**
```bash
./server/vault/stop.sh
```

## Secret Management

### Enable KV Secrets Engine

```powershell
# Enable version 2 key-value store
& server/tools/bin/vault.exe secrets enable -path=secret kv-v2

# Verify
& server/tools/bin/vault.exe secrets list
```

### Store Secrets

```powershell
# Store database credentials
& server/tools/bin/vault.exe kv put secret/database/postgres `
  username=noaark `
  password=SecurePassword123

# Store API keys
& server/tools/bin/vault.exe kv put secret/api/github `
  token=ghp_xxxxxxxxxxxxxxxxxxxx

# Store with metadata
& server/tools/bin/vault.exe kv put secret/agent-registry/config `
  redis_url=redis://localhost:6379 `
  postgres_dsn="postgres://user:pass@localhost:5432/agents"
```

### Retrieve Secrets

```powershell
# Get secret (JSON output)
& server/tools/bin/vault.exe kv get -format=json secret/database/postgres

# Get specific field
& server/tools/bin/vault.exe kv get -field=password secret/database/postgres

# List secrets
& server/tools/bin/vault.exe kv list secret/
```

### Update Secrets

```powershell
# Update creates new version
& server/tools/bin/vault.exe kv put secret/database/postgres `
  username=noaark `
  password=NewPassword456

# View version history
& server/tools/bin/vault.exe kv metadata get secret/database/postgres

# Rollback to previous version
& server/tools/bin/vault.exe kv rollback -version=1 secret/database/postgres
```

### Delete Secrets

```powershell
# Soft delete (can be undeleted)
& server/tools/bin/vault.exe kv delete secret/database/postgres

# Undelete
& server/tools/bin/vault.exe kv undelete -versions=2 secret/database/postgres

# Permanently destroy
& server/tools/bin/vault.exe kv destroy -versions=2 secret/database/postgres
```

## Access Control

### Create Policy

```powershell
# Create policy file (policies/agent-registry-policy.hcl)
$policyContent = @"
# Read-only access to agent-registry secrets
path "secret/data/agent-registry/*" {
  capabilities = ["read", "list"]
}

# Read-write access to agent runtime secrets
path "secret/data/agents/runtime/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
"@

New-Item -ItemType Directory -Force -Path "server/vault/policies"
Set-Content -Path "server/vault/policies/agent-registry-policy.hcl" -Value $policyContent

# Apply policy
& server/tools/bin/vault.exe policy write agent-registry server/vault/policies/agent-registry-policy.hcl

# List policies
& server/tools/bin/vault.exe policy list
```

### Create Token with Policy

```powershell
# Create token for agent-registry service
& server/tools/bin/vault.exe token create `
  -policy=agent-registry `
  -ttl=768h `
  -renewable `
  -display-name="agent-registry-service"

# OUTPUT:
# Key                  Value
# ---                  -----
# token                hvs.CAESXXXXXXXXXXXXXXXXXX
# token_accessor       XXXXXXXXXXXXXXXXXX
# token_duration       768h
# token_renewable      true
# token_policies       ["agent-registry" "default"]
```

### AppRole Authentication (for Services)

```powershell
# Enable AppRole auth
& server/tools/bin/vault.exe auth enable approle

# Create role for agent-registry
& server/tools/bin/vault.exe write auth/approle/role/agent-registry `
  token_policies="agent-registry" `
  token_ttl=1h `
  token_max_ttl=4h `
  bind_secret_id=true

# Get Role ID
& server/tools/bin/vault.exe read auth/approle/role/agent-registry/role-id

# Generate Secret ID
& server/tools/bin/vault.exe write -f auth/approle/role/agent-registry/secret-id

# Service authenticates with role-id + secret-id
& server/tools/bin/vault.exe write auth/approle/login `
  role_id=<role-id> `
  secret_id=<secret-id>
```

## Integration with NoaArkOS

### Agent Registry Service (Cycle 2)

Update `services/agent-registry/go/main.go` to use Vault:

```go
import (
    vault "github.com/hashicorp/vault/api"
)

func initVault() (*vault.Client, error) {
    config := vault.DefaultConfig()
    config.Address = "http://127.0.0.1:8200"
    
    client, err := vault.NewClient(config)
    if err != nil {
        return nil, err
    }
    
    // Authenticate using AppRole
    data := map[string]interface{}{
        "role_id":   os.Getenv("VAULT_ROLE_ID"),
        "secret_id": os.Getenv("VAULT_SECRET_ID"),
    }
    
    resp, err := client.Logical().Write("auth/approle/login", data)
    if err != nil {
        return nil, err
    }
    
    client.SetToken(resp.Auth.ClientToken)
    return client, nil
}

func getSecret(client *vault.Client, path string) (string, error) {
    secret, err := client.Logical().Read(path)
    if err != nil {
        return "", err
    }
    
    data := secret.Data["data"].(map[string]interface{})
    return data["value"].(string), nil
}

// Usage
vaultClient, _ := initVault()
redisPassword, _ := getSecret(vaultClient, "secret/data/agent-registry/redis-password")
```

### Caddy Integration (Cycle 3)

Front Vault with Caddy for TLS termination:

**Caddyfile:**
```caddyfile
https://localhost:8443 {
  tls internal
  
  # Route to Vault
  reverse_proxy 127.0.0.1:8200 {
    health_checks {
      active {
        path /v1/sys/health
        interval 10s
      }
    }
  }
}
```

**Access Vault through Caddy:**
```powershell
$env:VAULT_ADDR = "https://localhost:8443"
& server/tools/bin/vault.exe status
```

### Environment Variables

Store Vault credentials securely:

```powershell
# .env file (DO NOT COMMIT)
VAULT_ADDR=http://127.0.0.1:8200
VAULT_TOKEN=hvs.CAESXXXXXXXXXXXXXXXXXX
VAULT_ROLE_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
VAULT_SECRET_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

Load in PowerShell:
```powershell
Get-Content .env | ForEach-Object {
  if ($_ -match '^([^=]+)=(.*)$') {
    [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
  }
}
```

## Web UI

Access Vault's web interface:

1. Start Vault with `ui = true` in config
2. Navigate to: http://127.0.0.1:8200/ui/
3. Login with root token or AppRole credentials

**Features:**
- Browse secrets
- Create/update/delete secrets
- Manage policies
- View audit logs
- Monitor cluster health

## Backup & Recovery

### Backup Vault Data

```powershell
# Stop Vault first
Stop-Service VaultService

# Backup Raft data
Copy-Item -Recurse -Path "server/vault/data" -Destination "backup/vault-$(Get-Date -Format 'yyyyMMdd-HHmmss')"

# Start Vault
Start-Service VaultService
```

### Snapshot (Online Backup)

```powershell
# Take snapshot (Vault remains running)
& server/tools/bin/vault.exe operator raft snapshot save backup/vault-snapshot-$(Get-Date -Format 'yyyyMMdd-HHmmss').snap

# Restore from snapshot
& server/tools/bin/vault.exe operator raft snapshot restore backup/vault-snapshot-YYYYMMDD-HHMMSS.snap
```

### Disaster Recovery

If Vault becomes unavailable:

1. **Lost Unseal Keys:** Cannot recover - must re-initialize (LOSE ALL DATA)
2. **Corrupted Data:** Restore from snapshot
3. **Node Failure:** Raft quorum handles automatically (multi-node)

**Prevention:**
- ✅ Store unseal keys offline in multiple secure locations
- ✅ Automate snapshots (daily/weekly)
- ✅ Test restore procedure regularly
- ✅ Use Vault's built-in replication for HA

## Troubleshooting

### Vault Sealed After Restart

Vault automatically seals on restart - must unseal:

```powershell
& server/tools/bin/vault.exe operator unseal <key1>
& server/tools/bin/vault.exe operator unseal <key2>
& server/tools/bin/vault.exe operator unseal <key3>
```

**Auto-Unseal (Production):**
Use cloud KMS (AWS, Azure, GCP) for automatic unsealing:

```hcl
seal "azurekeyvault" {
  tenant_id      = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  client_id      = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  client_secret  = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  vault_name     = "noaarkos-vault"
  key_name       = "vault-unseal-key"
}
```

### Permission Denied Errors

```powershell
# Check token capabilities
& server/tools/bin/vault.exe token capabilities secret/data/mypath

# View token info
& server/tools/bin/vault.exe token lookup

# Renew token
& server/tools/bin/vault.exe token renew
```

### Connection Refused

```powershell
# Check if Vault is running
Get-Process vault

# Check port 8200
netstat -ano | findstr :8200

# Review logs
Get-Content server/vault/logs/server.log -Tail 50
```

### High Memory Usage

Vault caches secrets in memory - expected behavior. If excessive:

```hcl
# Adjust cache size in vault.hcl
cache {
  max_lease_ttl = "1h"
}
```

## Installation as Windows Service

Create `server/vault/install-service.ps1`:

```powershell
# Install Vault as Windows service (requires NSSM and admin rights)
$serviceName = "VaultService"
$vaultPath = "D:\dev\workspaces\noa_ark_os\server\tools\bin\vault.exe"
$configPath = "D:\dev\workspaces\noa_ark_os\server\vault\vault.hcl"

# Download NSSM if not present
if (-not (Test-Path "server/tools/bin/nssm.exe")) {
    Write-Host "Downloading NSSM..."
    $nssmUrl = "https://nssm.cc/release/nssm-2.24.zip"
    Invoke-WebRequest -Uri $nssmUrl -OutFile "nssm.zip"
    Expand-Archive -Path "nssm.zip" -DestinationPath "temp/"
    Copy-Item "temp/nssm-2.24/win64/nssm.exe" -Destination "server/tools/bin/"
    Remove-Item -Recurse "temp/", "nssm.zip"
}

# Install service
& server/tools/bin/nssm.exe install $serviceName $vaultPath "server" "-config=$configPath"

# Configure service
& server/tools/bin/nssm.exe set $serviceName AppDirectory "D:\dev\workspaces\noa_ark_os"
& server/tools/bin/nssm.exe set $serviceName DisplayName "HashiCorp Vault"
& server/tools/bin/nssm.exe set $serviceName Description "Secret management for NoaArkOS"
& server/tools/bin/nssm.exe set $serviceName Start SERVICE_DEMAND_START  # Manual start

Write-Host "Vault service installed. Start with: Start-Service $serviceName"
```

## Security Best Practices

### Development
- ✅ Bind to loopback (127.0.0.1) only
- ✅ Use Caddy for TLS termination
- ✅ Rotate root token regularly
- ✅ Use AppRole for service authentication
- ⚠️ `disable_mlock = true` acceptable for WSL

### Production
- ✅ Enable mlock (`disable_mlock = false`)
- ✅ Use TLS for all listeners
- ✅ Enable audit logging
- ✅ Use cloud auto-unseal (KMS)
- ✅ Multi-node cluster (3+ nodes)
- ✅ Regular backups/snapshots
- ❌ Never use dev mode
- ❌ Never expose port 8200 directly
- ❌ Never commit tokens/keys to git

## Cross-References

- **SECRETS_MANAGEMENT.md** - Overall secrets strategy for NoaArkOS
- **server/caddy/README.md** - TLS termination and reverse proxy
- **services/agent-registry/README.md** - Service integration example
- **SERVER_WSL_DROP_INTEGRATION_COMPLETE.md** - Integration summary (Cycle 3)

## Resources

- **Official Docs:** https://www.vaultproject.io/docs
- **API Reference:** https://www.vaultproject.io/api
- **Learn Vault:** https://learn.hashicorp.com/vault
- **Community Forum:** https://discuss.hashicorp.com/c/vault
- **GitHub:** https://github.com/hashicorp/vault

## Next Steps

1. Install Vault binary (see Installation section)
2. Customize `vault.hcl` for your environment (update paths)
3. Start Vault: `vault server -config=server/vault/vault.hcl`
4. Initialize: `vault operator init` (SAVE KEYS SECURELY)
5. Unseal: `vault operator unseal` (3 times with different keys)
6. Login: `vault login <root-token>`
7. Enable KV engine: `vault secrets enable -path=secret kv-v2`
8. Store first secret: `vault kv put secret/test value=hello`
9. Integrate with agent-registry (see Integration section)
10. Install as service (see Installation as Windows Service section)

---

**Source:** WSL `/home/deflex/workspace/server/vault/`  
**Integrated:** Cycle 3 (Server-WSL Drop)  
**Status:** ✅ Production-Ready Configuration  
**Version:** 1.0
