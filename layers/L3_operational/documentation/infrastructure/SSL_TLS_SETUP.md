# 🔒 HAL9 SSL/TLS Certificate Setup Guide

## Overview

This guide covers setting up SSL/TLS certificates for secure HTTPS communication with HAL9 server.

## Options

### 1. Let's Encrypt (Free, Recommended for Production)

#### Prerequisites
- Domain name pointing to your server
- Port 80 accessible from internet
- Docker installed

#### Setup with Certbot

```bash
# Create directory for certificates
mkdir -p ./ssl/certbot

# Use certbot Docker image
docker run -it --rm \
  -v ./ssl/certbot:/etc/letsencrypt \
  -p 80:80 \
  certbot/certbot certonly \
  --standalone \
  -d your-domain.com \
  -d www.your-domain.com \
  --email your-email@example.com \
  --agree-tos \
  --non-interactive
```

### 2. Self-Signed Certificate (Development/Testing)

#### Generate Certificate

```bash
# Create SSL directory
mkdir -p ./ssl/self-signed

# Generate private key
openssl genrsa -out ./ssl/self-signed/privkey.pem 2048

# Generate certificate signing request
openssl req -new -key ./ssl/self-signed/privkey.pem \
  -out ./ssl/self-signed/cert.csr \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

# Generate self-signed certificate (valid for 365 days)
openssl x509 -req -days 365 \
  -in ./ssl/self-signed/cert.csr \
  -signkey ./ssl/self-signed/privkey.pem \
  -out ./ssl/self-signed/fullchain.pem
```

### 3. Using Nginx as SSL Termination Proxy

#### docker-compose.ssl.yml

```yaml
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl:ro
    depends_on:
      - hal9-server
    networks:
      - hal9-network

  hal9-server:
    # Remove port mapping as nginx will proxy
    # ports:
    #   - "8080:8080"
    expose:
      - "8080"
```

#### nginx/nginx.conf

```nginx
events {
    worker_connections 1024;
}

http {
    # Redirect HTTP to HTTPS
    server {
        listen 80;
        server_name your-domain.com;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name your-domain.com;

        # SSL certificate files
        ssl_certificate /etc/ssl/certbot/live/your-domain.com/fullchain.pem;
        ssl_certificate_key /etc/ssl/certbot/live/your-domain.com/privkey.pem;

        # SSL configuration
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;
        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;

        # Security headers
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;

        # Proxy to HAL9 server
        location / {
            proxy_pass http://hal9-server:8080;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # WebSocket support
            proxy_read_timeout 86400;
        }

        # API endpoints
        location /api/ {
            proxy_pass http://hal9-server:8080/api/;
            proxy_http_version 1.1;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket endpoint
        location /ws {
            proxy_pass http://hal9-server:8080/ws;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

## Quick Start Scripts

### 1. Development (Self-Signed)

```bash
#!/bin/bash
# scripts/enable_ssl_dev.sh

echo "🔒 Setting up SSL for development"

# Generate self-signed certificate
mkdir -p ./ssl/self-signed

openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout ./ssl/self-signed/privkey.pem \
  -out ./ssl/self-signed/fullchain.pem \
  -subj "/C=US/ST=Dev/L=Dev/O=HAL9/CN=localhost"

echo "✅ Self-signed certificate created"

# Start with SSL
docker-compose -f docker-compose.yml -f docker-compose.ssl.yml up -d

echo "🚀 HAL9 running with SSL at https://localhost"
echo "⚠️  Browser will show security warning (expected for self-signed)"
```

### 2. Production (Let's Encrypt)

```bash
#!/bin/bash
# scripts/enable_ssl_prod.sh

DOMAIN=${1:-"your-domain.com"}
EMAIL=${2:-"admin@your-domain.com"}

echo "🔒 Setting up Let's Encrypt SSL for $DOMAIN"

# Get certificate
docker run -it --rm \
  -v $(pwd)/ssl/certbot:/etc/letsencrypt \
  -p 80:80 \
  certbot/certbot certonly \
  --standalone \
  -d $DOMAIN \
  -d www.$DOMAIN \
  --email $EMAIL \
  --agree-tos \
  --non-interactive

if [ $? -eq 0 ]; then
    echo "✅ Certificate obtained successfully"
    
    # Update nginx config with domain
    sed -i "s/your-domain.com/$DOMAIN/g" ./nginx/nginx.conf
    
    # Start with SSL
    docker-compose -f docker-compose.yml -f docker-compose.ssl.yml up -d
    
    echo "🚀 HAL9 running with SSL at https://$DOMAIN"
else
    echo "❌ Failed to obtain certificate"
    exit 1
fi
```

## Certificate Renewal

### Automatic Renewal with Cron

```bash
# Add to crontab
0 2 * * * /usr/bin/docker run --rm -v /path/to/ssl/certbot:/etc/letsencrypt certbot/certbot renew --quiet && docker-compose restart nginx
```

## Testing SSL Configuration

### 1. Test with curl
```bash
curl -v https://your-domain.com/health
```

### 2. Test SSL grade
```bash
# Using SSL Labs (online)
# Visit: https://www.ssllabs.com/ssltest/analyze.html?d=your-domain.com

# Using testssl.sh (local)
docker run --rm -ti drwetter/testssl.sh https://your-domain.com
```

## Troubleshooting

### Common Issues

1. **Certificate not trusted**
   - Self-signed certificates will always show warnings
   - For production, ensure domain DNS is properly configured

2. **Port 80/443 already in use**
   ```bash
   # Check what's using the ports
   sudo lsof -i :80
   sudo lsof -i :443
   ```

3. **WebSocket not working over SSL**
   - Ensure nginx proxy headers are correct
   - Check that `proxy_read_timeout` is set

4. **Certificate renewal fails**
   - Ensure port 80 is accessible
   - Check certbot logs in `./ssl/certbot/logs/`

## Security Best Practices

1. **Use strong SSL configuration**
   - Only TLS 1.2 and 1.3
   - Disable weak ciphers
   - Enable HSTS

2. **Regular updates**
   - Keep nginx updated
   - Renew certificates before expiry

3. **Monitor certificate expiry**
   ```bash
   # Check certificate expiry
   openssl x509 -enddate -noout -in ./ssl/certbot/live/your-domain.com/cert.pem
   ```

4. **Backup certificates**
   ```bash
   # Backup Let's Encrypt certificates
   tar -czf ssl-backup-$(date +%Y%m%d).tar.gz ./ssl/certbot
   ```

## Next Steps

1. Choose certificate type (self-signed for dev, Let's Encrypt for prod)
2. Run appropriate setup script
3. Test SSL configuration
4. Set up automatic renewal (production)
5. Monitor certificate expiry