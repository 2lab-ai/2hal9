#!/bin/bash

# Quick local SSL test

echo "=== HAL9 Local SSL Test ==="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if openssl is available
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}Error: openssl is required${NC}"
    exit 1
fi

# Generate test certificate
echo -e "${YELLOW}Generating test certificate...${NC}"
mkdir -p /tmp/hal9-ssl-test
openssl req -x509 -nodes -days 1 -newkey rsa:2048 \
    -keyout /tmp/hal9-ssl-test/key.pem \
    -out /tmp/hal9-ssl-test/cert.pem \
    -subj "/C=US/ST=Test/L=Test/O=HAL9/CN=localhost" 2>/dev/null

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test certificate generated${NC}"
    
    # Show certificate info
    echo -e "\n${YELLOW}Certificate Information:${NC}"
    openssl x509 -in /tmp/hal9-ssl-test/cert.pem -noout -subject -dates
    
    # Test HTTPS server
    echo -e "\n${YELLOW}Starting test HTTPS server...${NC}"
    cat > /tmp/hal9-ssl-test/test.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 SSL Test</title>
</head>
<body>
    <h1>✅ SSL/TLS is working!</h1>
    <p>This page is served over HTTPS</p>
    <pre id="info"></pre>
    <script>
        document.getElementById('info').textContent = 
            'Protocol: ' + window.location.protocol + '\n' +
            'Host: ' + window.location.host + '\n' +
            'Secure: ' + window.isSecureContext;
    </script>
</body>
</html>
EOF
    
    # Start simple HTTPS server with Python
    echo -e "${GREEN}Starting HTTPS server on https://localhost:8443${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo
    
    cd /tmp/hal9-ssl-test
    python3 -c "
import ssl
import http.server
import socketserver

Handler = http.server.SimpleHTTPRequestHandler
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain('cert.pem', 'key.pem')

with socketserver.TCPServer(('', 8443), Handler) as httpd:
    httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
    print('🔒 HTTPS Server running at https://localhost:8443')
    print('📄 Serving test page at https://localhost:8443/test.html')
    print('')
    print('Test with: curl -k https://localhost:8443/test.html')
    print('')
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print('\nServer stopped')
"
    
else
    echo -e "${RED}❌ Failed to generate certificate${NC}"
    exit 1
fi

# Cleanup
rm -rf /tmp/hal9-ssl-test