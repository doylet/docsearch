# Secrets Management

This directory contains configuration for managing secrets in production deployments.

## Structure

```
docker/secrets/
├── README.md                    # This file
├── k8s-secrets.yaml            # Kubernetes secrets template
├── docker-secrets.yml          # Docker Swarm secrets template
└── env-template.txt            # Environment variables template
```

## Secrets Configuration

### Required Secrets

1. **Database Credentials** (if using external database)
   - `DB_USER`
   - `DB_PASSWORD`
   - `DB_HOST`
   - `DB_PORT`
   - `DB_NAME`

2. **Vector Database Credentials**
   - `VECTOR_DB_API_KEY`
   - `VECTOR_DB_ENDPOINT`

3. **API Keys**
   - `EMBEDDING_API_KEY`
   - `RERANKING_API_KEY`

4. **Security Tokens**
   - `JWT_SECRET`
   - `API_SECRET_KEY`

5. **Monitoring Credentials**
   - `PROMETHEUS_USER`
   - `PROMETHEUS_PASSWORD`
   - `GRAFANA_ADMIN_PASSWORD`

### Development

For development, copy `env-template.txt` to `.env` and fill in the values:

```bash
cp docker/secrets/env-template.txt .env
# Edit .env with your development values
```

### Production

For production deployments:

#### Docker Compose with Secrets

```bash
# Create secrets
echo "your_jwt_secret" | docker secret create jwt_secret -
echo "your_api_key" | docker secret create api_key -

# Deploy with secrets
docker stack deploy -c docker-compose.prod.yml docsearch
```

#### Kubernetes

```bash
# Apply secrets
kubectl apply -f docker/secrets/k8s-secrets.yaml

# Deploy application
kubectl apply -f k8s/
```

## Security Best Practices

1. **Never commit secrets to version control**
2. **Use environment-specific secret management**
3. **Rotate secrets regularly**
4. **Use least-privilege access**
5. **Audit secret access**

## Integration

The application reads secrets through environment variables with fallbacks:

1. Docker/Kubernetes secrets (mounted as files)
2. Environment variables
3. Configuration files (development only)
