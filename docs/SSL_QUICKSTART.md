# 🔒 SSL/TLS Quick Start

## Development (Self-Signed Certificate)

```bash
# Enable SSL with one command
./scripts/enable_ssl_dev.sh
```

✅ That's it! Access HAL9 at **https://localhost**

## Production (Let's Encrypt)

```bash
# Enable SSL with your domain
./scripts/enable_ssl_prod.sh your-domain.com admin@your-domain.com
```

✅ Access HAL9 at **https://your-domain.com**

## What You Get

- 🔒 **HTTPS encryption** with TLS 1.2/1.3
- 🔄 **Auto HTTP to HTTPS redirect**
- 🌐 **WebSocket over SSL** support
- 🛡️ **Security headers** (HSTS, CSP, etc.)
- ⚡ **Nginx reverse proxy** with caching
- 🔍 **Health checks** built-in

## Test Your SSL

```bash
# Quick test
curl -k https://localhost/health

# Detailed SSL info
openssl s_client -connect localhost:443 -servername localhost
```

## Certificate Locations

- **Development**: `./ssl/self-signed/`
- **Production**: `./ssl/certbot/live/your-domain.com/`

## Stop SSL Services

```bash
# Stop all services including nginx
docker-compose -f docker-compose.yml -f docker-compose.ssl.yml down
```

## Troubleshooting

### Browser shows security warning?
- Normal for self-signed certificates
- Click "Advanced" → "Proceed to localhost"

### Port 443 already in use?
```bash
sudo lsof -i :443
# Kill the process or change nginx port in docker-compose.ssl.yml
```

### Certificate expired?
```bash
# For Let's Encrypt (auto-renews)
./scripts/renew_ssl.sh

# For self-signed (regenerate)
./scripts/enable_ssl_dev.sh
```

## Next Steps

1. 📖 Read full guide: [SSL_TLS_SETUP.md](./SSL_TLS_SETUP.md)
2. 🌐 Configure your domain
3. 🔐 Set up auto-renewal for production
4. 📊 Monitor certificate expiry