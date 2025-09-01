# Docsearch Kubernetes Deployment

This directory contains Kubernetes manifests for deploying the docsearch application to a Kubernetes cluster.

## Quick Deploy

```bash
# Apply secrets and configuration
kubectl apply -f ../docker/secrets/k8s-secrets.yaml

# Deploy the application
kubectl apply -f .

# Check deployment status
kubectl get pods -l app=docsearch
kubectl get services -l app=docsearch
```

## Files Overview

- `deployment.yaml` - Main application deployment with 3 replicas
- `service.yaml` - ClusterIP and headless services
- `ingress.yaml` - External and internal ingress configurations
- `hpa.yaml` - Horizontal Pod Autoscaler and Pod Disruption Budget
- `monitoring.yaml` - Prometheus and Grafana deployments (optional)

## Configuration

### Required Setup

1. **Secrets**: Update `../docker/secrets/k8s-secrets.yaml` with your base64-encoded secrets
2. **Domain**: Replace `docsearch.example.com` in `ingress.yaml` with your actual domain
3. **TLS**: Create TLS secret for HTTPS:
   ```bash
   kubectl create secret tls docsearch-tls \
     --cert=path/to/tls.crt \
     --key=path/to/tls.key
   ```

### Optional Components

#### Monitoring Stack
```bash
# Deploy Prometheus and Grafana
kubectl apply -f monitoring.yaml
```

#### Resource Quotas
```bash
# Apply resource limits (if needed)
kubectl apply -f resource-quota.yaml
```

## Scaling

The deployment includes:
- **HPA**: Auto-scales from 2-10 pods based on CPU (70%) and memory (80%)
- **PDB**: Ensures at least 1 pod remains available during disruptions
- **Rolling Updates**: Zero-downtime deployments

## Security

Security features include:
- Non-root container execution
- Read-only root filesystem
- Security context with dropped capabilities
- Network policies (if `network-policy.yaml` is applied)
- Secrets management via Kubernetes secrets

## Monitoring

The deployment is configured for:
- **Prometheus scraping**: Annotations for metrics collection
- **Health checks**: Liveness and readiness probes
- **Observability**: Structured logging and distributed tracing

## Access

### Internal Access
```bash
# Port-forward for local access
kubectl port-forward service/docsearch-service 8080:80
curl http://localhost:8080/health
```

### External Access
- **Public API**: `https://docsearch.example.com/`
- **Doc Indexer**: `https://docsearch.example.com/indexer/`
- **Internal API**: `http://docsearch-internal.cluster.local/`

## Troubleshooting

### Common Commands
```bash
# Check pod status
kubectl get pods -l app=docsearch

# View logs
kubectl logs -l app=docsearch -f

# Describe deployment
kubectl describe deployment docsearch

# Check HPA status
kubectl get hpa docsearch-hpa

# View events
kubectl get events --sort-by=.metadata.creationTimestamp
```

### Performance Tuning

Adjust resource requests/limits in `deployment.yaml`:
```yaml
resources:
  requests:
    memory: "512Mi"  # Increase for higher load
    cpu: "200m"
  limits:
    memory: "2Gi"    # Adjust based on usage
    cpu: "1000m"
```

### Debugging

Enable debug logging:
```bash
kubectl set env deployment/docsearch RUST_LOG=docsearch=debug,zero_latency=debug
```

## Production Checklist

- [ ] Update secrets with production values
- [ ] Configure TLS certificates
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy
- [ ] Set resource quotas and limits
- [ ] Enable network policies
- [ ] Configure log aggregation
- [ ] Set up CI/CD pipeline
- [ ] Test disaster recovery procedures
