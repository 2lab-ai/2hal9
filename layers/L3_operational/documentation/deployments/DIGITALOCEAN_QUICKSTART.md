# 🌊 DigitalOcean Quick Deployment

## 5-Minute Setup

### Prerequisites
- DigitalOcean account
- `doctl` CLI installed
- Domain name (optional)

### Step 1: Create Droplet

```bash
# One-command deployment
doctl compute droplet create hal9-prod \
  --image docker-20-04 \
  --size s-4vcpu-8gb \
  --region nyc3 \
  --ssh-keys $(doctl compute ssh-key list --format ID --no-header) \
  --user-data-file <(cat <<'EOF'
#!/bin/bash
# Update system
apt-get update && apt-get upgrade -y

# Install Docker
curl -fsSL https://get.docker.com | sh

# Install Docker Compose
curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# Clone HAL9
git clone https://github.com/yourusername/2hal9.git /opt/hal9
cd /opt/hal9

# Start services
docker-compose up -d
EOF
)
```

### Step 2: Setup Load Balancer

```bash
# Create load balancer
doctl compute load-balancer create \
  --name hal9-lb \
  --region nyc3 \
  --forwarding-rules entry_protocol:https,entry_port:443,target_protocol:http,target_port:8080,certificate_id:YOUR_CERT_ID \
  --health-check protocol:http,port:8080,path:/health,check_interval_seconds:10
```

### Step 3: Database Setup

```bash
# Create managed PostgreSQL
doctl databases create hal9-db \
  --engine pg \
  --region nyc3 \
  --size db-s-1vcpu-1gb \
  --version 14

# Create Redis cluster
doctl databases create hal9-redis \
  --engine redis \
  --region nyc3 \
  --size db-s-1vcpu-1gb
```

### Step 4: Configure Firewall

```bash
# Create firewall
doctl compute firewall create \
  --name hal9-firewall \
  --inbound-rules "protocol:tcp,ports:22,sources:address:0.0.0.0/0 protocol:tcp,ports:80,sources:address:0.0.0.0/0 protocol:tcp,ports:443,sources:address:0.0.0.0/0" \
  --outbound-rules "protocol:tcp,ports:all,destinations:address:0.0.0.0/0"
```

## Environment Configuration

```bash
# SSH into droplet
doctl compute ssh hal9-prod

# Create .env file
cat > /opt/hal9/.env << EOF
# Database (get from DO dashboard)
DATABASE_URL=postgresql://doadmin:XXXXX@hal9-db-do-user.db.ondigitalocean.com:25060/defaultdb?sslmode=require

# Redis
REDIS_URL=rediss://default:XXXXX@hal9-redis-do-user.db.ondigitalocean.com:25061

# Domain
DOMAIN_NAME=hal9.yourdomain.com
PUBLIC_URL=https://hal9.yourdomain.com

# Security
JWT_SECRET=$(openssl rand -base64 32)
EOF

# Restart services
cd /opt/hal9
docker-compose down
docker-compose up -d
```

## Monitoring Setup

```bash
# Enable metrics
doctl monitoring alert create \
  --name "HAL9 High CPU" \
  --type "v1/insights/droplet/cpu" \
  --description "Alert when CPU > 80%" \
  --compare GreaterThan \
  --value 80 \
  --window "5m" \
  --entities $(doctl compute droplet list --format ID --no-header)
```

## Backup Configuration

```bash
# Enable automated backups
doctl compute droplet-action enable-backups $(doctl compute droplet list --format ID --no-header)

# Create snapshot
doctl compute droplet-action snapshot $(doctl compute droplet list --format ID --no-header) --snapshot-name "hal9-$(date +%Y%m%d)"
```

## Scaling

### Vertical Scaling
```bash
# Resize droplet (requires poweroff)
doctl compute droplet-action resize $(doctl compute droplet list --format ID --no-header) --size s-8vcpu-16gb
```

### Horizontal Scaling
```bash
# Create additional droplets
for i in {2..3}; do
  doctl compute droplet create hal9-prod-$i \
    --image YOUR_SNAPSHOT_ID \
    --size s-4vcpu-8gb \
    --region nyc3
done

# Add to load balancer
doctl compute load-balancer add-droplets LB_ID \
  --droplet-ids $(doctl compute droplet list --format ID --no-header | tr '\n' ',')
```

## Costs Breakdown

| Service | Specs | Monthly Cost |
|---------|-------|-------------|
| Droplet | 4 vCPU, 8GB RAM | $48 |
| Database | 1 vCPU, 1GB RAM | $15 |
| Redis | 1 vCPU, 1GB RAM | $15 |
| Load Balancer | Standard | $12 |
| Backups | Weekly | $4.80 |
| **Total** | | **$94.80** |

## Useful Commands

```bash
# View logs
doctl compute ssh hal9-prod --ssh-command "docker logs hal9-server"

# Check status
doctl compute droplet list
doctl databases list
doctl compute load-balancer list

# Get droplet IP
doctl compute droplet get hal9-prod --format PublicIPv4 --no-header
```

## Troubleshooting

### Connection Issues
```bash
# Check firewall
doctl compute firewall list

# Test connectivity
curl -I http://$(doctl compute droplet get hal9-prod --format PublicIPv4 --no-header):8080/health
```

### Performance Issues
```bash
# Check metrics
doctl monitoring metrics droplet get cpu --droplet-ids $(doctl compute droplet list --format ID --no-header)
```

## Next Steps

1. Setup domain with DigitalOcean DNS
2. Configure SSL with Let's Encrypt
3. Enable monitoring alerts
4. Setup automated backups
5. Configure CDN with Spaces