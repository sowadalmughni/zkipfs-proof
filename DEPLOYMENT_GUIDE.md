# zkIPFS-Proof Deployment Guide

## Overview

This guide covers deploying zkIPFS-Proof across different environments, from local development to production-scale deployments. The system consists of multiple components that can be deployed independently or together.

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Frontend  │    │   CLI Tool      │    │  Smart Contract │
│   (React)       │    │   (Rust)        │    │  (Solidity)     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────┴───────────┐
                    │     Core Library        │
                    │     (Rust)              │
                    └─────────────┬───────────┘
                                  │
                    ┌─────────────┴───────────┐
                    │     IPFS Network        │
                    │     (Distributed)       │
                    └─────────────────────────┘
```

## Prerequisites

### System Requirements

**Minimum Requirements:**
- CPU: 2 cores, 2.0 GHz
- RAM: 4 GB
- Storage: 20 GB SSD
- Network: 10 Mbps

**Recommended Requirements:**
- CPU: 4+ cores, 3.0 GHz
- RAM: 8+ GB
- Storage: 100+ GB NVMe SSD
- Network: 100+ Mbps

### Software Dependencies

**Required:**
- Rust 1.75.0+
- Node.js 18.0+
- Docker 20.10+
- Git 2.30+

**Optional:**
- Kubernetes 1.25+
- Terraform 1.0+
- Ansible 2.9+

## Local Development Setup

### 1. Clone Repository

```bash
git clone https://github.com/sowadalmughni/zkipfs-proof.git
cd zkipfs-proof
```

### 2. Install Dependencies

```bash
# Install Rust dependencies
cargo build --workspace

# Install Node.js dependencies
cd frontend/web
npm install
cd ../..

# Install development tools
cargo install cargo-watch
npm install -g @vercel/ncc
```

### 3. Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

Example `.env` file:
```bash
# Core Configuration
RUST_LOG=info
ZKIPFS_PROOF_ENV=development
ZKIPFS_PROOF_PORT=8080

# IPFS Configuration
IPFS_API_URL=http://localhost:5001
IPFS_GATEWAY_URL=http://localhost:8080

# Database Configuration (optional)
DATABASE_URL=postgresql://user:pass@localhost/zkipfs_proof

# Security Configuration
JWT_SECRET=your_jwt_secret_here
API_KEY_SALT=your_api_key_salt_here

# ZK Configuration
ZK_SECURITY_LEVEL=128
ZK_ENABLE_GPU=false
ZK_BONSAI_API_KEY=your_bonsai_key_here
```

### 4. Start Development Services

```bash
# Start IPFS node
ipfs daemon &

# Start core service
cargo run --bin zkipfs-proof-server

# Start web frontend (in another terminal)
cd frontend/web
npm run dev
```

### 5. Verify Installation

```bash
# Test CLI tool
cargo run --bin zkipfs-proof -- --version

# Test API endpoint
curl http://localhost:8080/api/v1/health

# Test web interface
open http://localhost:3000
```

## Production Deployment

### Option 1: Docker Deployment

#### 1. Build Docker Images

```bash
# Build core service image
docker build -t zkipfs-proof-core:latest -f docker/Dockerfile.core .

# Build web frontend image
docker build -t zkipfs-proof-web:latest -f docker/Dockerfile.web .

# Build CLI tool image
docker build -t zkipfs-proof-cli:latest -f docker/Dockerfile.cli .
```

#### 2. Docker Compose Deployment

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  zkipfs-proof-core:
    image: zkipfs-proof-core:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - ZKIPFS_PROOF_ENV=production
      - DATABASE_URL=postgresql://postgres:password@db:5432/zkipfs_proof
    depends_on:
      - db
      - ipfs
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  zkipfs-proof-web:
    image: zkipfs-proof-web:latest
    ports:
      - "3000:3000"
    environment:
      - REACT_APP_API_URL=http://localhost:8080
    depends_on:
      - zkipfs-proof-core
    restart: unless-stopped

  ipfs:
    image: ipfs/go-ipfs:latest
    ports:
      - "4001:4001"
      - "5001:5001"
      - "8081:8080"
    volumes:
      - ipfs_data:/data/ipfs
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=zkipfs_proof
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - zkipfs-proof-core
      - zkipfs-proof-web
    restart: unless-stopped

volumes:
  ipfs_data:
  postgres_data:
```

#### 3. Deploy with Docker Compose

```bash
# Deploy production stack
docker-compose -f docker-compose.prod.yml up -d

# Check service status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

### Option 2: Kubernetes Deployment

#### 1. Create Kubernetes Manifests

**Namespace:**
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: zkipfs-proof
```

**ConfigMap:**
```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: zkipfs-proof-config
  namespace: zkipfs-proof
data:
  RUST_LOG: "info"
  ZKIPFS_PROOF_ENV: "production"
  IPFS_API_URL: "http://ipfs-service:5001"
```

**Core Service Deployment:**
```yaml
# k8s/core-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: zkipfs-proof-core
  namespace: zkipfs-proof
spec:
  replicas: 3
  selector:
    matchLabels:
      app: zkipfs-proof-core
  template:
    metadata:
      labels:
        app: zkipfs-proof-core
    spec:
      containers:
      - name: core
        image: zkipfs-proof-core:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: zkipfs-proof-config
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

**Service:**
```yaml
# k8s/core-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: zkipfs-proof-core-service
  namespace: zkipfs-proof
spec:
  selector:
    app: zkipfs-proof-core
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

**Ingress:**
```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: zkipfs-proof-ingress
  namespace: zkipfs-proof
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - your-domain.com  # Replace with your actual domain
    secretName: zkipfs-proof-tls
  rules:
  - host: your-domain.com  # Replace with your actual domain
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: zkipfs-proof-core-service
            port:
              number: 8080
```

#### 2. Deploy to Kubernetes

```bash
# Apply all manifests
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n zkipfs-proof

# Check service status
kubectl get services -n zkipfs-proof

# View logs
kubectl logs -f deployment/zkipfs-proof-core -n zkipfs-proof
```

### Option 3: Cloud Provider Deployment

#### AWS Deployment

**Using AWS ECS:**

1. Create ECS Cluster:
```bash
aws ecs create-cluster --cluster-name zkipfs-proof-cluster
```

2. Create Task Definition:
```json
{
  "family": "zkipfs-proof-core",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "zkipfs-proof-core",
      "image": "your-account.dkr.ecr.region.amazonaws.com/zkipfs-proof-core:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/zkipfs-proof",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

3. Create Service:
```bash
aws ecs create-service \
  --cluster zkipfs-proof-cluster \
  --service-name zkipfs-proof-core-service \
  --task-definition zkipfs-proof-core \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-12345],securityGroups=[sg-12345],assignPublicIp=ENABLED}"
```

#### Google Cloud Platform

**Using Google Cloud Run:**

```bash
# Build and push image
gcloud builds submit --tag gcr.io/PROJECT_ID/zkipfs-proof-core

# Deploy to Cloud Run
gcloud run deploy zkipfs-proof-core \
  --image gcr.io/PROJECT_ID/zkipfs-proof-core \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 10
```

#### Azure Deployment

**Using Azure Container Instances:**

```bash
# Create resource group
az group create --name zkipfs-proof-rg --location eastus

# Deploy container
az container create \
  --resource-group zkipfs-proof-rg \
  --name zkipfs-proof-core \
  --image zkipfs-proof-core:latest \
  --cpu 2 \
  --memory 4 \
  --ports 8080 \
  --environment-variables RUST_LOG=info
```

## Smart Contract Deployment

### 1. Install Foundry

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### 2. Configure Deployment

Create `contracts/.env`:
```bash
PRIVATE_KEY=your_private_key_here
RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your_key
ETHERSCAN_API_KEY=your_etherscan_key
```

### 3. Deploy Contracts

```bash
cd contracts

# Deploy to testnet
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify

# Deploy to mainnet
forge script script/Deploy.s.sol --rpc-url $MAINNET_RPC_URL --broadcast --verify
```

### 4. Verify Deployment

```bash
# Check contract on Etherscan
cast call $CONTRACT_ADDRESS "version()" --rpc-url $RPC_URL

# Test verification function
cast call $CONTRACT_ADDRESS "verifyProof(bytes,bytes32,address)" $PROOF_DATA $CONTENT_HASH $SUBMITTER --rpc-url $RPC_URL
```

## Monitoring and Observability

### 1. Prometheus Metrics

Add to `docker-compose.prod.yml`:

```yaml
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
    restart: unless-stopped
```

### 2. Logging Configuration

**Structured Logging:**
```rust
use tracing::{info, warn, error};
use tracing_subscriber;

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("zkipfs_proof=info")
        .json()
        .init();
}
```

**Log Aggregation with ELK Stack:**
```yaml
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.5.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    ports:
      - "9200:9200"

  logstash:
    image: docker.elastic.co/logstash/logstash:8.5.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf

  kibana:
    image: docker.elastic.co/kibana/kibana:8.5.0
    ports:
      - "5601:5601"
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
```

### 3. Health Checks

**Application Health Endpoint:**
```rust
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": get_uptime_seconds()
    }))
}
```

**Kubernetes Health Checks:**
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

## Security Configuration

### 1. TLS/SSL Setup

**Nginx Configuration:**
```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;  # Replace with your actual domain

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;

    location / {
        proxy_pass http://zkipfs-proof-core:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. Firewall Configuration

```bash
# UFW configuration
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
```

### 3. Secret Management

**Using Kubernetes Secrets:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: zkipfs-proof-secrets
  namespace: zkipfs-proof
type: Opaque
data:
  jwt-secret: <base64-encoded-secret>
  api-key-salt: <base64-encoded-salt>
  database-url: <base64-encoded-url>
```

**Using HashiCorp Vault:**
```bash
# Store secrets in Vault
vault kv put secret/zkipfs-proof \
  jwt_secret="your_jwt_secret" \
  api_key_salt="your_salt" \
  database_url="postgresql://..."

# Retrieve in application
vault kv get -field=jwt_secret secret/zkipfs-proof
```

## Backup and Disaster Recovery

### 1. Database Backup

```bash
# PostgreSQL backup
pg_dump -h localhost -U postgres zkipfs_proof > backup_$(date +%Y%m%d_%H%M%S).sql

# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump -h $DB_HOST -U $DB_USER $DB_NAME | gzip > $BACKUP_DIR/zkipfs_proof_$DATE.sql.gz

# Keep only last 30 days
find $BACKUP_DIR -name "zkipfs_proof_*.sql.gz" -mtime +30 -delete
```

### 2. IPFS Data Backup

```bash
# Export IPFS data
ipfs repo gc
tar -czf ipfs_backup_$(date +%Y%m%d).tar.gz ~/.ipfs

# Backup to S3
aws s3 cp ipfs_backup_$(date +%Y%m%d).tar.gz s3://your-backup-bucket/ipfs/
```

### 3. Application State Backup

```bash
# Backup application data
kubectl create backup zkipfs-proof-backup \
  --include-namespaces zkipfs-proof \
  --storage-location default

# Restore from backup
kubectl create restore zkipfs-proof-restore \
  --from-backup zkipfs-proof-backup
```

## Performance Tuning

### 1. Rust Application Optimization

**Cargo.toml optimizations:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

**Runtime optimizations:**
```rust
// Use jemalloc for better memory management
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

// Configure thread pool
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .enable_all()
    .build()?;
```

### 2. Database Optimization

**PostgreSQL configuration:**
```sql
-- Increase shared buffers
ALTER SYSTEM SET shared_buffers = '256MB';

-- Optimize for proof verification workload
ALTER SYSTEM SET work_mem = '64MB';
ALTER SYSTEM SET maintenance_work_mem = '256MB';

-- Enable parallel queries
ALTER SYSTEM SET max_parallel_workers_per_gather = 4;
```

### 3. IPFS Optimization

```bash
# Increase connection limits
ipfs config Swarm.ConnMgr.HighWater 2000
ipfs config Swarm.ConnMgr.LowWater 1000

# Enable experimental features
ipfs config --json Experimental.FilestoreEnabled true
ipfs config --json Experimental.UrlstoreEnabled true

# Optimize datastore
ipfs config Datastore.StorageMax "100GB"
```

## Troubleshooting

### Common Issues

**1. Out of Memory Errors:**
```bash
# Check memory usage
docker stats
kubectl top pods -n zkipfs-proof

# Increase memory limits
docker run -m 4g zkipfs-proof-core:latest
```

**2. IPFS Connection Issues:**
```bash
# Check IPFS connectivity
ipfs swarm peers
ipfs id

# Reset IPFS configuration
ipfs config --json Addresses.Swarm '["/ip4/0.0.0.0/tcp/4001", "/ip6/::/tcp/4001"]'
```

**3. Proof Generation Timeouts:**
```bash
# Increase timeout values
export ZKIPFS_PROOF_TIMEOUT=300

# Check ZK circuit compilation
cargo build --release --features gpu
```

### Log Analysis

**Common log patterns:**
```bash
# Proof generation errors
grep "proof generation failed" /var/log/zkipfs-proof.log

# Performance issues
grep "slow query" /var/log/zkipfs-proof.log | tail -20

# Memory issues
grep "out of memory" /var/log/zkipfs-proof.log
```

## Maintenance

### 1. Regular Updates

```bash
# Update dependencies
cargo update
npm update

# Security updates
cargo audit
npm audit

# Rebuild images
docker build --no-cache -t zkipfs-proof-core:latest .
```

### 2. Database Maintenance

```bash
# Vacuum database
psql -c "VACUUM ANALYZE;" zkipfs_proof

# Reindex tables
psql -c "REINDEX DATABASE zkipfs_proof;"

# Update statistics
psql -c "ANALYZE;" zkipfs_proof
```

### 3. IPFS Maintenance

```bash
# Garbage collection
ipfs repo gc

# Verify repository
ipfs repo verify

# Update IPFS
ipfs update install latest
```

## Support

For deployment issues or questions:

- **GitHub Repository**: [https://github.com/sowadalmughni/zkipfs-proof](https://github.com/sowadalmughni/zkipfs-proof)
- **GitHub Issues**: [https://github.com/sowadalmughni/zkipfs-proof/issues](https://github.com/sowadalmughni/zkipfs-proof/issues)
- **Email Support**: sowadalmughni@gmail.com

