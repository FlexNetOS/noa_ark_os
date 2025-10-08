# Deployment Guide

## Deployment Overview

This guide covers deploying the noa_ark_os unified system in various environments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development Deployment](#local-development-deployment)
3. [Production Deployment](#production-deployment)
4. [Docker Deployment](#docker-deployment)
5. [Kubernetes Deployment](#kubernetes-deployment)
6. [Cloud Deployment](#cloud-deployment)
7. [Monitoring and Maintenance](#monitoring-and-maintenance)

## Prerequisites

### System Requirements

- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 50GB+ available space
- **OS**: Linux (Ubuntu 20.04+), macOS, Windows with WSL2

### Software Requirements

- Docker 20.10+
- Docker Compose 1.29+
- Git 2.30+
- Python 3.8+
- Rust 1.70+ (for building from source)

## Local Development Deployment

### Quick Start

```bash
# Clone repository
git clone https://github.com/FlexNetOS/noa_ark_os.git
cd noa_ark_os

# Start all services
./scripts/start-all-services.sh
```

### Component-by-Component Setup

#### 1. MicroAgentStack
```bash
cd repos/MicroAgentStack
pip install -r requirements.txt
python main.py
# Access at: http://localhost:8000
```

#### 2. ark-os-noa
```bash
cd repos/ark-os-noa
docker-compose up -d
# Services available on configured ports
```

#### 3. deflex-ai-os
```bash
cd repos/deflex-ai-os
cargo build --release
./target/release/deflex-ai-os
```

## Production Deployment

### Environment Configuration

Create environment files for each component:

**`.env.production`**
```bash
# Common settings
ENV=production
DEBUG=false
LOG_LEVEL=info

# Database
DATABASE_URL=postgresql://user:pass@localhost/noa_ark_os

# Security
SECRET_KEY=your-super-secret-key
JWT_SECRET=your-jwt-secret

# Services
ORCHESTRATOR_URL=http://orchestrator:8000
AGENT_REGISTRY_URL=http://agent-registry:8001
```

### Build for Production

```bash
# Python components
cd repos/MicroAgentStack
pip install -r requirements.txt
python -m compileall .

# Rust components
cd repos/agentaskit
cargo build --release

cd ../deflex-ai-os
cargo build --release
```

### System Service Setup (Linux)

Create systemd service files:

**`/etc/systemd/system/noa-orchestrator.service`**
```ini
[Unit]
Description=noa_ark_os Orchestrator
After=network.target

[Service]
Type=simple
User=noaarkos
WorkingDirectory=/opt/noa_ark_os/repos/MicroAgentStack
ExecStart=/usr/bin/python3 main.py
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable noa-orchestrator
sudo systemctl start noa-orchestrator
sudo systemctl status noa-orchestrator
```

## Docker Deployment

### Single-Host Docker Compose

**`docker-compose.production.yml`**
```yaml
version: '3.8'

services:
  orchestrator:
    build: ./repos/MicroAgentStack
    ports:
      - "8000:8000"
    environment:
      - ENV=production
    volumes:
      - ./data:/app/data
    restart: always
    networks:
      - noa_network

  arkos-core:
    build: ./repos/ark-os-noa
    environment:
      - ENV=production
    volumes:
      - ./config:/app/config
    restart: always
    networks:
      - noa_network

  file-ops:
    build: ./repos/deflex-ai-os
    environment:
      - ENV=production
    volumes:
      - ./storage:/app/storage
    restart: always
    networks:
      - noa_network

  agent-executor:
    build: ./repos/agentaskit
    environment:
      - ENV=production
    restart: always
    networks:
      - noa_network

  pipeline:
    build: ./repos/deflexnet-app
    environment:
      - ENV=production
    restart: always
    networks:
      - noa_network

networks:
  noa_network:
    driver: bridge

volumes:
  data:
  config:
  storage:
```

Deploy:
```bash
docker-compose -f docker-compose.production.yml up -d
```

### Docker Swarm Deployment

Initialize swarm:
```bash
docker swarm init
```

Deploy stack:
```bash
docker stack deploy -c docker-compose.production.yml noa_ark_os
```

Monitor:
```bash
docker stack services noa_ark_os
docker stack ps noa_ark_os
```

## Kubernetes Deployment

### Prerequisites
- Kubernetes cluster (minikube, k3s, or cloud provider)
- kubectl configured
- Helm 3+ installed

### Namespace Setup

```bash
kubectl create namespace noa-ark-os
kubectl config set-context --current --namespace=noa-ark-os
```

### Deployment Manifests

**`k8s/orchestrator-deployment.yaml`**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orchestrator
  namespace: noa-ark-os
spec:
  replicas: 3
  selector:
    matchLabels:
      app: orchestrator
  template:
    metadata:
      labels:
        app: orchestrator
    spec:
      containers:
      - name: orchestrator
        image: ghcr.io/flexnetos/noa_ark_os/microagentstack:latest
        ports:
        - containerPort: 8000
        env:
        - name: ENV
          value: "production"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: orchestrator
  namespace: noa-ark-os
spec:
  selector:
    app: orchestrator
  ports:
  - port: 8000
    targetPort: 8000
  type: LoadBalancer
```

Deploy:
```bash
kubectl apply -f k8s/
```

### Helm Chart (Future)

```bash
# Install with Helm
helm repo add noa-ark-os https://charts.flexnetos.io
helm install noa-ark-os noa-ark-os/noa-ark-os
```

## Cloud Deployment

### AWS Deployment

#### Using ECS
```bash
# Create ECS cluster
aws ecs create-cluster --cluster-name noa-ark-os

# Register task definitions
aws ecs register-task-definition --cli-input-json file://ecs-task-def.json

# Create service
aws ecs create-service \
  --cluster noa-ark-os \
  --service-name orchestrator \
  --task-definition orchestrator:1 \
  --desired-count 2
```

#### Using EKS
```bash
# Create EKS cluster
eksctl create cluster --name noa-ark-os --region us-west-2

# Deploy application
kubectl apply -f k8s/
```

### Google Cloud Platform

```bash
# Create GKE cluster
gcloud container clusters create noa-ark-os \
  --num-nodes=3 \
  --zone=us-central1-a

# Deploy
kubectl apply -f k8s/
```

### Azure Deployment

```bash
# Create AKS cluster
az aks create \
  --resource-group noa-ark-os-rg \
  --name noa-ark-os \
  --node-count 3

# Get credentials
az aks get-credentials \
  --resource-group noa-ark-os-rg \
  --name noa-ark-os

# Deploy
kubectl apply -f k8s/
```

## Monitoring and Maintenance

### Health Checks

```bash
# Check orchestrator health
curl http://localhost:8000/health

# Check all services
./scripts/health-check-all.sh
```

### Logging

#### Centralized Logging with ELK Stack

```yaml
# docker-compose.logging.yml
version: '3.8'
services:
  elasticsearch:
    image: elasticsearch:7.14.0
    environment:
      - discovery.type=single-node
    ports:
      - "9200:9200"

  logstash:
    image: logstash:7.14.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf

  kibana:
    image: kibana:7.14.0
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_URL: http://elasticsearch:9200
```

### Monitoring with Prometheus

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'orchestrator'
    static_configs:
      - targets: ['orchestrator:8000']
  
  - job_name: 'agent-executor'
    static_configs:
      - targets: ['agent-executor:8001']
```

### Backup and Recovery

#### Backup Script
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/noa_ark_os/$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# Backup data volumes
docker run --rm -v noa_data:/data -v $BACKUP_DIR:/backup \
  alpine tar czf /backup/data.tar.gz -C /data .

# Backup configurations
tar czf $BACKUP_DIR/config.tar.gz config/

echo "Backup completed: $BACKUP_DIR"
```

#### Restore Script
```bash
#!/bin/bash
# restore.sh

BACKUP_DIR=$1

# Restore data volumes
docker run --rm -v noa_data:/data -v $BACKUP_DIR:/backup \
  alpine tar xzf /backup/data.tar.gz -C /data

# Restore configurations
tar xzf $BACKUP_DIR/config.tar.gz -C .

echo "Restore completed from: $BACKUP_DIR"
```

### Scaling

#### Horizontal Scaling
```bash
# Docker Swarm
docker service scale noa_ark_os_orchestrator=5

# Kubernetes
kubectl scale deployment orchestrator --replicas=5
```

#### Vertical Scaling
```bash
# Update resource limits in deployment manifests
kubectl apply -f k8s/orchestrator-deployment.yaml
```

### Updates and Maintenance

#### Rolling Updates
```bash
# Docker Swarm
docker service update --image ghcr.io/flexnetos/orchestrator:v2.0 \
  noa_ark_os_orchestrator

# Kubernetes
kubectl set image deployment/orchestrator \
  orchestrator=ghcr.io/flexnetos/orchestrator:v2.0
```

#### Zero-Downtime Deployment
```bash
# Blue-Green Deployment
kubectl apply -f k8s/orchestrator-deployment-green.yaml
# Test green deployment
kubectl delete -f k8s/orchestrator-deployment-blue.yaml
```

## Security Best Practices

1. **Use TLS/SSL**: Always enable HTTPS in production
2. **Secrets Management**: Use Kubernetes secrets or AWS Secrets Manager
3. **Network Policies**: Implement network segmentation
4. **Regular Updates**: Keep dependencies up to date
5. **Access Control**: Implement RBAC and least privilege
6. **Audit Logging**: Enable comprehensive audit trails

## Troubleshooting

### Common Issues

1. **Container won't start**
   ```bash
   docker logs container_name
   docker inspect container_name
   ```

2. **High memory usage**
   ```bash
   docker stats
   # Adjust resource limits
   ```

3. **Network connectivity issues**
   ```bash
   docker network inspect noa_network
   # Check firewall rules
   ```

## Support

For deployment issues:
- Check documentation in `/docs`
- Review logs in `/var/log/noa_ark_os`
- Open issue on GitHub
- Contact support team

## Next Steps

1. Set up monitoring dashboards
2. Configure automated backups
3. Implement disaster recovery plan
4. Set up alerting and notifications
5. Document runbooks for common scenarios
