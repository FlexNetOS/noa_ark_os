# Caddy Web Server Configuration

Modern web server for TLS termination, reverse proxy, and service fronting.

## Overview

This directory contains production-grade Caddy configurations adapted from WSL development environment. Caddy provides:

- **Automatic HTTPS** via ACME (Let's Encrypt)
- **TLS Termination** for internal services
- **Reverse Proxy** with health checks and load balancing
- **Security Headers** (HSTS, X-Frame-Options, etc.)
- **Compression** (Brotli, Gzip, Zstandard)
- **Structured Logging** with rotation policies
- **Zero Downtime Reloads**

## Files

- `Caddyfile` - Main configuration with three disabled-by-default examples
- `caddy.env` - Environment variables (VAULT_UPSTREAM_ADDR, CLIENT_CA)

## Installation

### Windows

1. Download Caddy from https://caddyserver.com/download
2. Extract to `server/tools/bin/caddy.exe` (or add to PATH)
3. Verify installation:
   ```powershell
   caddy version
   ```

### WSL/Linux

```bash
# Via package manager (Ubuntu/Debian)
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

## Usage

### Validate Configuration

Before running, validate the Caddyfile syntax:

```powershell
# Windows
caddy validate --config server/caddy/Caddyfile

# WSL
wsl caddy validate --config /mnt/d/dev/workspaces/noa_ark_os/server/caddy/Caddyfile
```

### Run (Foreground)

Useful for testing and development:

```powershell
# Windows (from workspace root)
caddy run --config server/caddy/Caddyfile

# WSL
wsl caddy run --config /mnt/d/dev/workspaces/noa_ark_os/server/caddy/Caddyfile
```

### Run (Background Service)

#### Windows Service

Create `server/caddy/install-service.ps1`:

```powershell
# Install Caddy as Windows service (requires admin)
$serviceName = "CaddyWeb"
$caddyPath = "D:\dev\workspaces\noa_ark_os\server\tools\bin\caddy.exe"
$configPath = "D:\dev\workspaces\noa_ark_os\server\caddy\Caddyfile"

# Create service
nssm install $serviceName $caddyPath "run" "--config" $configPath

# Configure service
nssm set $serviceName AppDirectory "D:\dev\workspaces\noa_ark_os"
nssm set $serviceName DisplayName "Caddy Web Server"
nssm set $serviceName Description "Modern web server for NoaArkOS services"
nssm set $serviceName Start SERVICE_AUTO_START

# Start service
Start-Service $serviceName
```

#### Linux systemd (WSL)

For persistent service in WSL, use the provided systemd unit:

```bash
# Copy service file
sudo cp server/caddy/caddy.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable caddy
sudo systemctl start caddy

# Check status
sudo systemctl status caddy
```

## Configuration Examples

The `Caddyfile` includes three disabled-by-default examples. To enable, uncomment the desired block.

### Example 1: Local Vault with TLS

Front local Vault (port 8200) with TLS termination on port 8443:

```caddyfile
https://localhost:8443 {
  tls internal  # Self-signed cert for local dev
  
  encode zstd gzip
  
  header {
    Strict-Transport-Security "max-age=31536000"
    X-Content-Type-Options "nosniff"
    X-Frame-Options "DENY"
  }
  
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

**Test:**
```powershell
# Trust local CA cert (first time only)
caddy trust

# Start Caddy
caddy run --config server/caddy/Caddyfile

# Access Vault through Caddy
curl https://localhost:8443/v1/sys/health
```

### Example 2: Public Domain with ACME

For production deployment with automatic Let's Encrypt certificates:

```caddyfile
https://vault.example.com {
  # Automatic HTTPS via ACME HTTP-01
  # For DNS-01 (wildcard certs), configure DNS provider
  
  encode zstd gzip
  
  reverse_proxy 127.0.0.1:8200 {
    lb_policy least_conn
  }
}
```

**Prerequisites:**
- Domain DNS pointing to server IP
- Port 80 and 443 open (for ACME challenge)
- Or DNS provider credentials (for DNS-01 challenge)

### Example 3: Static File Server

Serve static files (useful for documentation, UI builds):

```caddyfile
http://localhost:8080 {
  root * ./ui/web/dist
  file_server browse
  encode zstd gzip
}
```

## Architecture Integration

### With Vault

```
Client (HTTPS/8443)
    ↓ [TLS Termination]
Caddy Reverse Proxy
    ↓ [HTTP/Loopback]
Vault (127.0.0.1:8200)
```

**Benefits:**
- Single TLS certificate (Caddy handles all TLS)
- Security headers applied uniformly
- Health checks prevent routing to unhealthy instances
- Compression reduces bandwidth
- Structured logging for all requests

### With Agent Registry (Cycle 2)

```
Client (HTTPS)
    ↓
Caddy (Port 8443)
    ├→ /vault/* → Vault (8200)
    ├→ /agents/* → Agent Registry (8080)
    └→ /metrics → Prometheus (9090)
```

**Caddyfile Addition:**
```caddyfile
https://localhost:8443 {
  tls internal
  
  encode zstd gzip
  
  # Route to different services
  handle /vault/* {
    reverse_proxy 127.0.0.1:8200
  }
  
  handle /agents/* {
    reverse_proxy 127.0.0.1:8080
  }
  
  handle /metrics {
    reverse_proxy 127.0.0.1:9090
  }
  
  # Default: serve UI
  handle {
    root * ./ui/web/dist
    file_server
  }
}
```

## Security Best Practices

### For Local Development
- ✅ Use `tls internal` for self-signed certs
- ✅ Trust Caddy's local CA: `caddy trust`
- ✅ Bind services to loopback (127.0.0.1) only
- ✅ Use Caddy as the only public-facing service

### For Production
- ✅ Use ACME with DNS-01 for wildcard certs
- ✅ Enable mTLS for client authentication:
  ```caddyfile
  tls {
    client_auth {
      mode require_and_verify
      trusted_ca_cert_file config/ca/clients.pem
    }
  }
  ```
- ✅ Set proper CORS headers
- ✅ Enable rate limiting (via Caddy plugins)
- ✅ Use strong cipher suites (Caddy defaults are good)
- ❌ Never expose services directly (always proxy through Caddy)
- ❌ Never use `tls_disable` in production

## Logging

### Configuration

Logs are written to `server/caddy/logs/applications/caddy/`:

- `app.log` - Caddy application logs (errors, warnings, info)
- `access.log` - HTTP access logs in JSON format

**Rotation Policy:**
- Application logs: 50 MiB per file, keep 5 files, 120 days retention
- Access logs: 100 MiB per file, keep 7 files, 30 days retention

### Log Format

Access logs use structured JSON:

```json
{
  "level": "info",
  "ts": 1704067200,
  "logger": "http.log.access",
  "msg": "handled request",
  "request": {
    "remote_ip": "127.0.0.1",
    "proto": "HTTP/2.0",
    "method": "GET",
    "host": "localhost:8443",
    "uri": "/v1/sys/health",
    "headers": { ... }
  },
  "duration": 0.003,
  "size": 234,
  "status": 200
}
```

### Viewing Logs

```powershell
# Tail application log
Get-Content server/caddy/logs/applications/caddy/app.log -Wait -Tail 20

# Parse JSON access log (requires jq)
wsl cat server/caddy/logs/applications/caddy/access.log | jq -r '[.ts, .request.method, .request.uri, .status] | @tsv'
```

## Troubleshooting

### Port Already in Use

```powershell
# Find process using port 8443
netstat -ano | findstr :8443

# Kill process (use PID from above)
taskkill /PID <PID> /F
```

### Certificate Trust Issues

```powershell
# Windows: Trust Caddy's local CA
caddy trust

# View trusted certificate
caddy untrust --help  # Shows CA cert location
```

### Configuration Errors

```powershell
# Validate before running
caddy validate --config server/caddy/Caddyfile

# Run with debug logging
caddy run --config server/caddy/Caddyfile --debug
```

### Health Check Failures

If health checks fail for Vault:

1. Verify Vault is running: `vault status`
2. Check Vault health endpoint: `curl http://127.0.0.1:8200/v1/sys/health`
3. Review Vault logs: `Get-Content server/vault/logs/server.log -Tail 20`

## Advanced Configuration

### Custom Storage Path

By default, Caddy stores certificates in OS data directory. For portable setup:

```caddyfile
{
  storage file_system {
    root D:/dev/workspaces/noa_ark_os/server/data/storage/caddy
  }
}
```

### Environment Variables

Use `caddy.env` for configuration:

```env
VAULT_UPSTREAM_ADDR=127.0.0.1:8200
CLIENT_CA=D:/dev/workspaces/noa_ark_os/config/system/ca/clients.pem
ACME_EMAIL=admin@example.com
```

Load in PowerShell:

```powershell
Get-Content server/caddy/caddy.env | ForEach-Object {
  if ($_ -match '^([^=]+)=(.*)$') {
    [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
  }
}

caddy run --config server/caddy/Caddyfile
```

### DNS Providers (for DNS-01 ACME)

For wildcard certificates or when port 80/443 are unavailable:

```caddyfile
https://*.example.com {
  tls {
    dns cloudflare {env.CLOUDFLARE_API_TOKEN}
  }
  # ...
}
```

Supported providers: Cloudflare, Route53, Google Cloud DNS, Azure DNS, etc.

## Integration with NoaArkOS

### Server Architecture

```
NoaArkOS Workspace (D:\dev\workspaces\noa_ark_os\)
├── server/
│   ├── tools/
│   │   ├── bin/caddy.exe         # Caddy binary
│   │   ├── cargo-portable/        # Rust toolchain
│   │   └── rustup-portable/       # Rust installer
│   ├── caddy/
│   │   ├── Caddyfile              # This configuration
│   │   ├── caddy.env              # Environment config
│   │   └── logs/                  # Log files
│   ├── vault/                     # Vault configuration
│   └── data/
│       ├── storage/caddy/         # Certificates & keys
│       └── database/              # Application data
├── services/
│   └── agent-registry/            # Microservices (Cycle 2)
└── core/                          # NoaArkOS kernel
```

### Relationship to Existing Server/ Directory

The existing `server/` directory contains:
- Portable development toolchains (Rust, Cargo)
- Build utilities and setup scripts

This Caddy configuration **complements** by adding:
- Service fronting and TLS termination
- Reverse proxy for microservices
- Production deployment infrastructure

### Cross-References

- **SECRETS_MANAGEMENT.md** - Vault integration for secret storage
- **services/agent-registry/README.md** - Microservice that can be fronted by Caddy
- **SERVER_WSL_DROP_INTEGRATION_COMPLETE.md** - Integration summary (Cycle 3)

## Resources

- **Official Docs:** https://caddyserver.com/docs/
- **Caddyfile Syntax:** https://caddyserver.com/docs/caddyfile
- **Community Forum:** https://caddy.community/
- **GitHub:** https://github.com/caddyserver/caddy
- **Docker Image:** https://hub.docker.com/_/caddy

## Next Steps

1. Install Caddy binary (see Installation section)
2. Customize `Caddyfile` for your services (uncomment examples)
3. Update `caddy.env` with your environment variables
4. Validate configuration: `caddy validate --config server/caddy/Caddyfile`
5. Run in foreground: `caddy run --config server/caddy/Caddyfile`
6. Test endpoints: `curl https://localhost:8443/v1/sys/health`
7. Install as service (see Run Background Service section)

---

**Source:** WSL `/home/deflex/workspace/server/caddy/`  
**Integrated:** Cycle 3 (Server-WSL Drop)  
**Status:** ✅ Production-Ready Configuration  
**Version:** 1.0
