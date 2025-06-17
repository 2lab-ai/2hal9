# ☁️ Cloud Deployment Quick Start

## Choose Your Provider

### 💰 Budget Option: DigitalOcean ($95/month)
```bash
# 5-minute deployment
doctl compute droplet create hal9 \
  --image docker-20-04 \
  --size s-4vcpu-8gb \
  --region nyc3
```
[📖 Full Guide](./deployments/DIGITALOCEAN_QUICKSTART.md)

### 🚀 Scale Option: AWS ($315/month)
```bash
# Using AWS CLI
aws ec2 run-instances \
  --image-id ami-0c55b159cbfafe1f0 \
  --instance-type t3.large \
  --key-name your-key
```
[📖 Full Guide](./deployments/AWS_DEPLOYMENT.md)

### 🤖 AI Option: Google Cloud ($275/month)
```bash
# Using gcloud
gcloud compute instances create hal9 \
  --machine-type=n2-standard-2 \
  --zone=us-central1-a
```
[📖 Full Guide](./deployments/GCP_DEPLOYMENT.md)

## Universal Quick Deploy Script

```bash
#!/bin/bash
# save as deploy-hal9.sh

# Install Docker
curl -fsSL https://get.docker.com | sh

# Clone repository
git clone https://github.com/yourusername/2hal9.git
cd 2hal9

# Setup environment
cp .env.example .env
# Edit .env with your settings

# Start HAL9
docker-compose up -d

# Enable SSL
./scripts/enable_ssl_prod.sh your-domain.com your-email@example.com
```

## Provider Comparison

| Feature | DO | AWS | GCP | Azure |
|---------|----|----|-----|-------|
| Setup Time | 5 min | 30 min | 20 min | 30 min |
| Monthly Cost | $95 | $315 | $275 | $350 |
| Complexity | ★ | ★★★★★ | ★★★ | ★★★★ |
| Best For | Startups | Enterprise | AI/ML | Microsoft |

## Pre-configured Templates

### Terraform (All Providers)
```bash
# Clone templates
git clone https://github.com/yourusername/hal9-terraform

# Deploy to any provider
cd hal9-terraform/digitalocean
terraform init
terraform apply
```

### Kubernetes (Any Provider)
```bash
# Deploy to existing cluster
kubectl apply -f https://raw.githubusercontent.com/yourusername/2hal9/main/k8s/
```

## Post-Deployment Checklist

- [ ] Domain configured
- [ ] SSL certificate installed
- [ ] Monitoring enabled
- [ ] Backups configured
- [ ] Firewall rules set
- [ ] Health checks passing

## Get Help

- 📚 [Full Comparison Guide](./CLOUD_PROVIDER_COMPARISON.md)
- 🔧 [Troubleshooting Guide](./TROUBLESHOOTING.md)
- 💬 [Community Discord](https://discord.gg/hal9)