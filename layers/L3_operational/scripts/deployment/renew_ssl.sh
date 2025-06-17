#!/bin/bash

# SSL Certificate renewal script for Let's Encrypt
# Add to crontab: 0 2 * * * /path/to/renew_ssl.sh

LOG_FILE="./logs/ssl-renewal.log"
mkdir -p ./logs

echo "[$(date)] Starting SSL renewal check" >> $LOG_FILE

# Renew certificates
docker run --rm \
  -v $(pwd)/ssl/certbot:/etc/letsencrypt \
  -v $(pwd)/ssl/certbot:/var/www/certbot \
  certbot/certbot renew \
  --quiet \
  --no-self-upgrade \
  >> $LOG_FILE 2>&1

if [ $? -eq 0 ]; then
    echo "[$(date)] Certificate renewal successful" >> $LOG_FILE
    
    # Reload nginx to use new certificate
    docker-compose -f docker-compose.yml -f docker-compose.ssl.yml restart nginx >> $LOG_FILE 2>&1
    
    echo "[$(date)] Nginx reloaded" >> $LOG_FILE
else
    echo "[$(date)] Certificate renewal failed" >> $LOG_FILE
fi

echo "[$(date)] Renewal check complete" >> $LOG_FILE
echo "----------------------------------------" >> $LOG_FILE