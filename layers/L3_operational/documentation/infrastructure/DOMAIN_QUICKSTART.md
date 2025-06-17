# 🌐 Domain Setup Quick Start

## Prerequisites

- 📦 Domain name registered
- 🌍 Server with public IP
- 🔓 Ports 80 & 443 open

## Quick Setup (5 minutes)

```bash
# Run the domain setup wizard
./scripts/setup_domain.sh
```

The script will:
1. 📑 Guide you through DNS setup
2. 🔍 Wait for DNS propagation
3. ⚙️ Update configurations
4. 🔒 Install SSL certificate

## Manual DNS Setup

### At Your Domain Registrar

Add these records:

| Type | Name | Value | TTL |
|------|------|-------|-----|
| A | @ | YOUR-SERVER-IP | 300 |
| A | www | YOUR-SERVER-IP | 300 |

### For Subdomains

| Type | Name | Value | TTL |
|------|------|-------|-----|
| A | api | YOUR-SERVER-IP | 300 |
| A | app | YOUR-SERVER-IP | 300 |

## Test Your Domain

```bash
# Check DNS
nslookup your-domain.com

# Test HTTPS
curl https://your-domain.com/health
```

## Common DNS Providers

### Cloudflare
1. Add site to Cloudflare
2. Change nameservers at registrar
3. Add A records in Cloudflare DNS
4. Set SSL/TLS to "Full (strict)"

### Route 53 (AWS)
```bash
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123456789 \
  --change-batch file://dns-records.json
```

### Google Domains
1. Go to DNS settings
2. Add custom records
3. Type: A, Data: YOUR-IP

## Troubleshooting

### DNS not resolving?
- Wait 5-30 minutes for propagation
- Check typos in DNS records
- Verify at: https://dnschecker.org

### SSL certificate failed?
- Ensure DNS is fully propagated
- Check firewall allows port 80
- Try: `telnet your-domain.com 80`

### Still having issues?
```bash
# Debug DNS
dig your-domain.com +trace

# Check server accessibility
curl -I http://YOUR-SERVER-IP
```

## Next Steps

1. 🛡️ Enable DNSSEC
2. 📧 Set up email records (SPF/DKIM)
3. 🌐 Configure CDN
4. 📊 Set up monitoring