# HAL9 SSL/TLS Configuration Guide

## 📋 Overview

HAL9 provides comprehensive SSL/TLS support for secure HTTPS communication. This guide covers setup for development, staging, and production environments.

## 🚀 Quick Start

### Development (Self-Signed Certificate)
```bash
# Generate and enable self-signed certificate
./layers/L3_operational/scripts/deployment/enable_ssl_dev.sh

# Access HAL9 via HTTPS
https://localhost:443
```

### Production (Let's Encrypt)
```bash
# Obtain and configure Let's Encrypt certificate
./layers/L3_operational/scripts/deployment/enable_ssl_prod.sh yourdomain.com admin@yourdomain.com

# Access HAL9 via HTTPS
https://yourdomain.com
```

## 🏗️ Architecture

### Components
1. **Nginx** - Reverse proxy handling SSL termination
2. **Let's Encrypt** - Free SSL certificate provider
3. **Certbot** - Automated certificate management
4. **Docker** - Containerized deployment

### Certificate Storage
```
ssl/
├── certbot/           # Let's Encrypt certificates
│   └── live/
│       └── yourdomain.com/
│           ├── cert.pem
│           ├── chain.pem
│           ├── fullchain.pem
│           └── privkey.pem
└── self-signed/       # Development certificates
    ├── fullchain.pem
    └── privkey.pem
```

## 🔧 Configuration

### Nginx SSL Configuration
Located at: `layers/L3_operational/configuration/nginx/nginx.conf`

Key settings:
- **Protocols**: TLS 1.2 and 1.3 only
- **Cipher Suites**: Modern, secure ciphers
- **HSTS**: Enabled with 1-year max-age
- **Security Headers**: X-Frame-Options, CSP, etc.
- **OCSP Stapling**: Enabled for performance

### Environment Variables
```bash
# SSL/TLS Configuration
SSL_ENABLED=true
SSL_CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
SSL_KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
SSL_RENEWAL_EMAIL=admin@yourdomain.com

# Development mode
DEV_MODE=false
DEV_AUTO_RELOAD=false
```

## 📝 Step-by-Step Setup

### 1. Prerequisites
- Domain pointing to your server
- Ports 80 and 443 open
- Docker and Docker Compose installed

### 2. DNS Configuration
```bash
# Verify DNS is properly configured
nslookup yourdomain.com
dig yourdomain.com

# Should return your server's IP address
```

### 3. Initial Setup
```bash
# Clone the repository
git clone https://github.com/yourusername/2hal9.git
cd 2hal9

# Create necessary directories
mkdir -p ssl/certbot nginx logs

# Copy configuration files
cp layers/L3_operational/configuration/nginx/nginx.conf nginx/
cp layers/L3_operational/configuration/docker/docker-compose.ssl.yml .
```

### 4. Obtain SSL Certificate

#### Option A: Production (Let's Encrypt)
```bash
# Run the automated setup script
./layers/L3_operational/scripts/deployment/enable_ssl_prod.sh yourdomain.com admin@yourdomain.com

# The script will:
# 1. Start temporary nginx for verification
# 2. Request certificate from Let's Encrypt
# 3. Configure nginx with the certificate
# 4. Restart services with HTTPS enabled
```

#### Option B: Development (Self-Signed)
```bash
# Generate self-signed certificate
./layers/L3_operational/scripts/deployment/enable_ssl_dev.sh

# This creates a certificate valid for 365 days
```

### 5. Verify Installation
```bash
# Test HTTPS connection
curl -I https://yourdomain.com

# Check certificate details
openssl s_client -connect yourdomain.com:443 -servername yourdomain.com

# Run comprehensive SSL test
./layers/L3_operational/scripts/test-ssl-setup.sh
```

## 🔄 Certificate Renewal

### Automatic Renewal
Let's Encrypt certificates expire after 90 days. Set up automatic renewal:

```bash
# Add to crontab
crontab -e

# Add this line (runs daily at 2 AM)
0 2 * * * /path/to/2hal9/layers/L3_operational/scripts/deployment/renew_ssl.sh
```

### Manual Renewal
```bash
# Run renewal script manually
./layers/L3_operational/scripts/deployment/renew_ssl.sh

# Check renewal status
docker logs hal9-nginx | grep -i ssl
```

## 🛡️ Security Best Practices

### 1. Strong Configuration
- ✅ Use TLS 1.2 or higher only
- ✅ Disable weak cipher suites
- ✅ Enable Perfect Forward Secrecy (PFS)
- ✅ Use 2048-bit or stronger RSA keys
- ✅ Enable HSTS with long duration

### 2. Headers Configuration
```nginx
# Security headers in nginx.conf
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options "DENY" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "no-referrer-when-downgrade" always;
add_header Content-Security-Policy "default-src 'self' https:; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline';" always;
```

### 3. Additional Security Measures
```bash
# CAA DNS Record (prevents unauthorized certificate issuance)
yourdomain.com. CAA 0 issue "letsencrypt.org"

# Enable OCSP Must-Staple
# Add to certificate request
--must-staple

# Implement certificate pinning for mobile apps
# Pin the Let's Encrypt intermediate certificate
```

## 🐳 Docker Deployment

### Docker Compose with SSL
```yaml
# docker-compose.ssl.yml
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl/certbot:/etc/ssl/certbot:ro
      - ./ssl/self-signed:/etc/ssl/self-signed:ro
      - ./ssl/certbot:/var/www/certbot:ro
    depends_on:
      - hal9-server
```

### Start with SSL
```bash
# Production
docker-compose -f docker-compose.yml -f docker-compose.ssl.yml up -d

# Development
docker-compose -f docker-compose.yml -f docker-compose.ssl.yml -f docker-compose.dev.yml up
```

## 🌐 Kubernetes Deployment

### Using cert-manager
```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hal9-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    kubernetes.io/ingress.class: "nginx"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - hal9.yourdomain.com
    secretName: hal9-tls
  rules:
  - host: hal9.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: hal9-service
            port:
              number: 8080
```

### Install cert-manager
```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@yourdomain.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

## 🔍 Monitoring & Alerts

### Certificate Expiration Monitoring
```bash
# Check certificate expiration
openssl x509 -in /path/to/cert.pem -noout -dates

# Monitor with Prometheus
- job_name: 'ssl_expiry'
  metrics_path: /probe
  params:
    module: [ssl_cert]
  static_configs:
    - targets:
      - 'https://hal9.yourdomain.com'
```

### Log Monitoring
```bash
# Check SSL-related logs
docker logs hal9-nginx | grep -i ssl
docker logs hal9-nginx | grep -i certificate

# Check renewal logs
tail -f ./logs/ssl-renewal.log
```

## 🚨 Troubleshooting

### Common Issues

#### 1. Certificate Request Failed
```bash
# Check DNS
dig +short yourdomain.com

# Test HTTP accessibility
curl http://yourdomain.com/.well-known/acme-challenge/test

# Check firewall
sudo iptables -L -n | grep -E "80|443"
```

#### 2. SSL Handshake Failed
```bash
# Test SSL connection
openssl s_client -connect yourdomain.com:443 -debug

# Check certificate chain
openssl verify -CAfile chain.pem cert.pem

# Check nginx configuration
nginx -t
```

#### 3. Mixed Content Warnings
```bash
# Find HTTP resources
grep -r "http://" ./public/
grep -r "http://" ./src/

# Update to use protocol-relative URLs
// Instead of: http://example.com/resource
// Use: //example.com/resource
```

### Debug Commands
```bash
# View certificate details
openssl x509 -in cert.pem -text -noout

# Test specific TLS version
openssl s_client -connect yourdomain.com:443 -tls1_2
openssl s_client -connect yourdomain.com:443 -tls1_3

# Check cipher suites
nmap --script ssl-enum-ciphers -p 443 yourdomain.com

# SSL Labs test (external)
# Visit: https://www.ssllabs.com/ssltest/analyze.html?d=yourdomain.com
```

## 📚 Additional Resources

- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [SSL Labs Best Practices](https://github.com/ssllabs/research/wiki/SSL-and-TLS-Deployment-Best-Practices)
- [OWASP TLS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html)

## 🎯 Security Checklist

- [ ] TLS 1.2 or higher only
- [ ] Strong cipher suites configured
- [ ] HSTS enabled with includeSubDomains
- [ ] Security headers configured
- [ ] Certificate auto-renewal set up
- [ ] Monitoring for expiration
- [ ] Regular security updates
- [ ] Certificate transparency logs monitored
- [ ] CAA records configured
- [ ] OCSP stapling enabled