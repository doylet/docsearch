# Docker Deployment Guide

Production-ready Docker setup for Zero-Latency Document Search.

## Quick Start

### 1. Build and Start Services

```bash
make docker-build
make docker-up
```

Or use Docker Compose directly:

```bash
docker-compose up -d
```

### 2. Access the Application

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8081

### 3. View Logs

```bash
make docker-logs
```

## Commands

| Command | Description |
|---------|-------------|
| `make docker-build` | Build all Docker images |
| `make docker-up` | Start all services |
| `make docker-down` | Stop all services |
| `make docker-logs` | View logs from all services |
| `make docker-restart` | Restart all services |
| `make docker-clean` | Remove containers, images, and volumes |

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │
│   Frontend      │────▶│    Backend      │
│   (Next.js)     │     │    (Rust)       │
│   Port 3000     │     │    Port 8081    │
│                 │     │                 │
└─────────────────┘     └─────────────────┘
                              │
                              ▼
                        ┌─────────────┐
                        │   Vector    │
                        │    Store    │
                        │  (Volumes)  │
                        └─────────────┘
```

## Configuration

### Environment Variables

Create a `.env.local` file based on `.env.example`:

```bash
cp .env.example .env.local
```

Key variables:
- `NEXT_PUBLIC_API_URL`: Frontend API endpoint (default: http://localhost:8081)
- `RUST_LOG`: Backend logging level (info, debug, trace)
- `PORT`: Backend server port (default: 8081)

### Volumes

Persistent data is stored in Docker volumes:

- `docsearch_data`: Vector store and indexed documents
- `docsearch_logs`: Application logs
- `./collections`: Shared directory for collection configurations

## Production Deployment

### 1. Update Configuration

Edit `docker-compose.yml` for your environment:

```yaml
services:
  frontend:
    environment:
      - NEXT_PUBLIC_API_URL=https://api.yourdomain.com
```

### 2. Use Production Build

The Docker images are already optimized for production:
- Frontend: Multi-stage build with standalone output
- Backend: Release build with optimizations

### 3. Add Reverse Proxy (Recommended)

For production, add nginx or Traefik:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - frontend
      - backend
```

### 4. Enable HTTPS

Mount SSL certificates in the reverse proxy:

```yaml
volumes:
  - /etc/letsencrypt:/etc/nginx/ssl:ro
```

## Health Checks

Both services include health checks:

### Backend
```bash
curl http://localhost:8081/api/collections
```

### Frontend
```bash
curl http://localhost:3000
```

### Check Status
```bash
docker-compose ps
```

## Troubleshooting

### Services Not Starting

Check logs:
```bash
docker-compose logs backend
docker-compose logs frontend
```

### Port Conflicts

Change ports in `docker-compose.yml`:
```yaml
ports:
  - "3001:3000"  # Frontend on 3001
  - "8082:8081"  # Backend on 8082
```

### Reset Everything

```bash
make docker-clean
make docker-build
make docker-up
```

### Backend Can't Connect to Storage

Ensure volumes are properly mounted:
```bash
docker-compose down -v
docker-compose up -d
```

## Scaling

### Horizontal Scaling

Run multiple backend replicas:

```yaml
services:
  backend:
    deploy:
      replicas: 3
```

### Load Balancing

Add a load balancer:

```yaml
services:
  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx-lb.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - backend
```

## Monitoring (Optional)

Enable monitoring profile:

```bash
docker-compose --profile monitoring up -d
```

This starts:
- Prometheus (metrics): http://localhost:9090
- Grafana (dashboards): http://localhost:3000

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Images
        run: make docker-build
      - name: Push to Registry
        run: |
          docker tag docsearch-backend:latest registry.example.com/docsearch-backend:latest
          docker push registry.example.com/docsearch-backend:latest
```

## Security

### Best Practices

1. **Use secrets for sensitive data**:
   ```yaml
   secrets:
     api_key:
       file: ./secrets/api_key.txt
   ```

2. **Run as non-root user** (already configured in Dockerfiles)

3. **Limit resources**:
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '2'
         memory: 2G
   ```

4. **Use Docker secrets** for production:
   ```yaml
   environment:
     - API_KEY_FILE=/run/secrets/api_key
   secrets:
     - api_key
   ```

## Performance

### Image Sizes

- Frontend: ~150MB (multi-stage build)
- Backend: ~50MB (release build)

### Startup Times

- Backend: ~2-5 seconds
- Frontend: ~1-2 seconds

### Resource Usage

- Backend: ~100-500MB RAM
- Frontend: ~50-200MB RAM

## Backup

### Backup Volumes

```bash
docker run --rm -v docsearch_data:/data -v $(pwd):/backup alpine tar czf /backup/data-backup.tar.gz /data
```

### Restore Volumes

```bash
docker run --rm -v docsearch_data:/data -v $(pwd):/backup alpine tar xzf /backup/data-backup.tar.gz -C /
```

## Support

For issues or questions:
- Check logs: `make docker-logs`
- Review documentation: `frontend/README.md`
- File an issue on GitHub
