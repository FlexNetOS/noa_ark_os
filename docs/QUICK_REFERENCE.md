# Quick Reference Guide

## Common Commands and Operations

### Getting Started

```bash
# Clone the repository
git clone https://github.com/FlexNetOS/noa_ark_os.git
cd noa_ark_os

# Start all services
./scripts/start-all-services.sh

# Stop all services
./scripts/stop-all-services.sh
```

### Working with Components

```bash
# Update a component from upstream
./scripts/update-subtree.sh MicroAgentStack

# Navigate to a component
cd repos/MicroAgentStack

# Make changes and commit
git add .
git commit -m "feat(MicroAgentStack): add new feature"
cd ../..
git push
```

### Development

```bash
# Run Python tests
cd repos/MicroAgentStack
pytest tests/ -v

# Run Rust tests
cd repos/agentaskit
cargo test

# Format Python code
black .
flake8 .

# Format Rust code
cargo fmt
cargo clippy
```

### Docker Operations

```bash
# Build all containers
docker-compose build

# Start specific service
cd repos/MicroAgentStack
docker-compose up -d

# View logs
docker-compose logs -f

# Stop and remove
docker-compose down -v
```

### Git Operations

```bash
# List all remotes
git remote -v

# Update from upstream
git fetch upstream
git rebase upstream/main

# Check status
git status

# View recent commits
git log --oneline --graph -10
```

### Component-Specific Commands

#### MicroAgentStack
```bash
cd repos/MicroAgentStack
python main.py                    # Start orchestrator
python -m pytest tests/          # Run tests
docker-compose up                # Start with Docker
```

#### ark-os-noa
```bash
cd repos/ark-os-noa
docker-compose up                # Start services
python pipeline.py               # Run pipeline
```

#### agentaskit (Rust)
```bash
cd repos/agentaskit
cargo build --release            # Build release
cargo test                       # Run tests
cargo run                        # Run application
```

#### deflex-ai-os (Rust)
```bash
cd repos/deflex-ai-os
cargo build --release            # Build release
./target/release/deflex-ai-os   # Run application
docker-compose up                # Start with Docker
```

#### deflexnet-app
```bash
cd repos/deflexnet-app
python run_pipeline.py           # Run pipeline
python -m pytest tests/          # Run tests
```

### Documentation

```bash
# View documentation
cat README.md                    # Main README
cat docs/ARCHITECTURE.md         # Architecture
cat docs/DEVELOPMENT.md          # Dev guide
cat docs/DEPLOYMENT.md           # Deployment
cat docs/API.md                  # API reference

# Generate API docs
cd repos/MicroAgentStack
pdoc --html --output-dir docs .
```

### Troubleshooting

```bash
# Clean Docker
docker system prune -a
docker volume prune

# Reset git state
git reset --hard HEAD
git clean -fd

# Check service health
curl http://localhost:8000/health

# View all running containers
docker ps

# Check logs
docker-compose logs -f service-name
```

## File Locations

```
Project Root: /
├── Main README: /README.md
├── Contributing: /CONTRIBUTING.md
├── Components: /repos/
├── Documentation: /docs/
├── Scripts: /scripts/
└── CI/CD: /.github/workflows/
```

## Important URLs

- **Repository**: https://github.com/FlexNetOS/noa_ark_os
- **Issues**: https://github.com/FlexNetOS/noa_ark_os/issues
- **PRs**: https://github.com/FlexNetOS/noa_ark_os/pulls
- **Docs** (local): http://localhost:8080/docs

## Service Endpoints

- **Orchestrator**: http://localhost:8000
- **API Docs**: http://localhost:8000/docs
- **Health Check**: http://localhost:8000/health
- **Metrics**: http://localhost:8000/metrics

## Git Subtree Commands

```bash
# Pull updates from component
git subtree pull --prefix=repos/COMPONENT COMPONENT main --squash

# Push changes to component
git subtree push --prefix=repos/COMPONENT COMPONENT main

# Add new component
git subtree add --prefix=repos/NEW_COMPONENT REMOTE main --squash
```

## Environment Variables

```bash
# Set development environment
export ENV=development
export DEBUG=true
export LOG_LEVEL=debug

# Set production environment
export ENV=production
export DEBUG=false
export LOG_LEVEL=info
```

## Quick Tips

1. **Always work from repo root**: Commands assume you're at `/`
2. **Use scripts**: Prefer `./scripts/*.sh` over manual commands
3. **Test before commit**: Run tests locally before pushing
4. **Update regularly**: Keep your branch up to date with main
5. **Document changes**: Update relevant docs with code changes

## Getting Help

```bash
# View help for scripts
./scripts/start-all-services.sh --help

# Check component README
cat repos/MicroAgentStack/README.md

# View CI/CD logs
# Visit: https://github.com/FlexNetOS/noa_ark_os/actions
```

## Cheat Sheet

| Task | Command |
|------|---------|
| Clone repo | `git clone https://github.com/FlexNetOS/noa_ark_os.git` |
| Start all | `./scripts/start-all-services.sh` |
| Stop all | `./scripts/stop-all-services.sh` |
| Update component | `./scripts/update-subtree.sh COMPONENT` |
| Run Python tests | `pytest tests/` |
| Run Rust tests | `cargo test` |
| Format Python | `black .` |
| Format Rust | `cargo fmt` |
| Build Docker | `docker-compose build` |
| View logs | `docker-compose logs -f` |
| Health check | `curl http://localhost:8000/health` |

---

**Need more help?**
- Read `/docs/DEVELOPMENT.md` for detailed guide
- Check `/docs/TROUBLESHOOTING.md` for common issues
- Open an issue on GitHub
- Contact the team
