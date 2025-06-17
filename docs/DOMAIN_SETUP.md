# 🌐 HAL9 Domain Setup Guide

## Overview

This guide covers setting up a custom domain for your HAL9 deployment.

## Prerequisites

- Domain name registered with a domain registrar
- Access to DNS management
- Server with public IP address
- Ports 80 and 443 open

## Step 1: Choose Your Domain Structure

### Option A: Root Domain
- `hal9.ai`
- `yourdomain.com`

### Option B: Subdomain
- `api.hal9.ai`
- `hal9.yourdomain.com`

### Option C: Multiple Subdomains
- `api.hal9.ai` - API endpoints
- `ws.hal9.ai` - WebSocket connections
- `admin.hal9.ai` - Admin panel

## Step 2: Configure DNS Records

### For Root Domain

```
Type    Name    Value           TTL
A       @       YOUR.IP.HERE    300
A       www     YOUR.IP.HERE    300
```

### For Subdomain

```
Type    Name    Value           TTL
A       api     YOUR.IP.HERE    300
A       ws      YOUR.IP.HERE    300
```

### For Load Balancing (Multiple IPs)

```
Type    Name    Value           TTL
A       @       IP1.HERE        300
A       @       IP2.HERE        300
A       @       IP3.HERE        300
```

## Step 3: Update HAL9 Configuration

### Update .env file

```bash
# Domain configuration
DOMAIN_NAME=hal9.yourdomain.com
ALLOWED_ORIGINS=https://hal9.yourdomain.com,https://www.hal9.yourdomain.com
```

### Update nginx configuration

```bash
# Edit nginx/nginx.conf
server_name hal9.yourdomain.com www.hal9.yourdomain.com;
```

### Update docker-compose

```yaml
# docker-compose.production.yml
services:
  hal9-server:
    environment:
      - PUBLIC_URL=https://hal9.yourdomain.com
      - ALLOWED_ORIGINS=${ALLOWED_ORIGINS}
```

## Step 4: SSL Certificate Setup

Once DNS is configured and propagated:

```bash
# Setup SSL with Let's Encrypt
./scripts/enable_ssl_prod.sh hal9.yourdomain.com admin@yourdomain.com
```

## Step 5: Test Domain Configuration

### DNS Propagation Check

```bash
# Check if DNS has propagated
nslookup hal9.yourdomain.com
dig hal9.yourdomain.com

# Check from multiple locations
curl -I https://dns-checker.org/hal9.yourdomain.com
```

### Domain Connectivity Test

```bash
# Test HTTP redirect
curl -I http://hal9.yourdomain.com
# Should return 301 redirect to HTTPS

# Test HTTPS
curl https://hal9.yourdomain.com/health
# Should return {"status":"healthy"}

# Test WebSocket
wscat -c wss://hal9.yourdomain.com/ws
```

## Step 6: Configure CDN (Optional)

### Cloudflare Setup

1. Add site to Cloudflare
2. Update nameservers at registrar
3. Configure SSL/TLS mode: "Full (strict)"
4. Page Rules:
   ```
   *api.hal9.yourdomain.com/*
   - Cache Level: Bypass
   - Disable Performance
   ```

### AWS CloudFront Setup

```yaml
# cloudfront-distribution.yml
Origins:
  - DomainName: hal9.yourdomain.com
    OriginPath: ""
    CustomOriginConfig:
      HTTPPort: 80
      HTTPSPort: 443
      OriginProtocolPolicy: https-only
```

## Multi-Region Setup

### Geographic DNS Routing

```
# Route 53 Geolocation Routing
US East: us-east.hal9.yourdomain.com -> US-EAST-IP
EU West: eu-west.hal9.yourdomain.com -> EU-WEST-IP
Asia Pacific: ap.hal9.yourdomain.com -> AP-IP

# Main domain with latency routing
hal9.yourdomain.com -> Closest region
```

### Health Checks

```bash
# Create health check endpoint
curl https://hal9.yourdomain.com/health

# Monitor with external service
- Uptime Robot
- Pingdom
- AWS Route 53 Health Checks
```

## Security Considerations

### 1. DNSSEC

Enable DNSSEC at your registrar for protection against DNS spoofing.

### 2. CAA Records

```
Type    Name    Value                           TTL
CAA     @       0 issue "letsencrypt.org"       300
CAA     @       0 issuewild "letsencrypt.org"   300
```

### 3. SPF/DKIM/DMARC (If sending emails)

```
Type    Name        Value                               TTL
TXT     @           "v=spf1 ip4:YOUR.IP.HERE -all"     300
TXT     _dmarc      "v=DMARC1; p=reject;"               300
```

## Monitoring and Alerts

### DNS Monitoring Script

```bash
#!/bin/bash
# scripts/monitor_dns.sh

DOMAIN="hal9.yourdomain.com"
EXPECTED_IP="YOUR.IP.HERE"

CURRENT_IP=$(dig +short $DOMAIN)

if [ "$CURRENT_IP" != "$EXPECTED_IP" ]; then
    echo "DNS mismatch! Expected: $EXPECTED_IP, Got: $CURRENT_IP"
    # Send alert
fi
```

### SSL Certificate Monitoring

```bash
# Check certificate expiry
openssl s_client -connect hal9.yourdomain.com:443 -servername hal9.yourdomain.com < /dev/null 2>/dev/null | openssl x509 -noout -enddate
```

## Troubleshooting

### DNS Not Resolving

1. Check propagation time (can take up to 48 hours)
2. Verify DNS records at registrar
3. Clear local DNS cache:
   ```bash
   # macOS
   sudo dscacheutil -flushcache
   
   # Linux
   sudo systemctl restart systemd-resolved
   
   # Windows
   ipconfig /flushdns
   ```

### SSL Certificate Issues

1. Ensure ports 80/443 are open
2. Check DNS is fully propagated
3. Verify domain ownership

### Connection Timeouts

1. Check firewall rules
2. Verify server is accessible
3. Test from different networks

## Production Checklist

- [ ] DNS records configured
- [ ] DNS propagated (all regions)
- [ ] SSL certificate installed
- [ ] HTTP to HTTPS redirect working
- [ ] WebSocket connections working
- [ ] API endpoints accessible
- [ ] Monitoring configured
- [ ] Backup DNS configured
- [ ] CDN configured (optional)
- [ ] DNSSEC enabled
- [ ] CAA records set
- [ ] Health checks configured

## Next Steps

1. Configure DNS records at your registrar
2. Wait for propagation (5-30 minutes usually)
3. Run SSL setup script
4. Test all endpoints
5. Set up monitoring
6. Configure CDN if needed