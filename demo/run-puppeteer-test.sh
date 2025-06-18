#!/bin/bash
# Puppeteer로 실제 브라우저 테스트 실행

set -e

echo "🧪 Puppeteer 브라우저 자동화 테스트"
echo "==================================="
echo ""

# 1. Node.js 확인
if ! command -v node &> /dev/null; then
    echo "❌ Node.js가 설치되어 있지 않습니다."
    echo "   설치: brew install node (macOS) 또는 apt install nodejs (Linux)"
    exit 1
fi

# 2. Puppeteer 설치 확인
if [ ! -d "node_modules/puppeteer" ]; then
    echo "📦 Puppeteer 설치 중..."
    npm install puppeteer
fi

# 3. 간단한 데모 서버 시작
echo "🌐 테스트용 서버 시작..."
python3 -c "
import http.server
import socketserver
import json
from datetime import datetime

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            self.wfile.write(b'''
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Simple Demo</title>
    <style>
        body { font-family: Arial; padding: 50px; }
        button { font-size: 20px; padding: 10px 20px; }
        #result { margin-top: 20px; padding: 20px; background: #f0f0f0; }
    </style>
</head>
<body>
    <h1>HAL9 테스트 데모</h1>
    <button onclick=\"test()\">테스트 실행</button>
    <div id=\"result\"></div>
    <script>
        function test() {
            document.getElementById('result').textContent = '요청 중...';
            fetch('/api/test')
                .then(r => r.json())
                .then(data => {
                    document.getElementById('result').innerHTML = 
                        '<strong>서버 응답:</strong><br>' +
                        '시간: ' + data.time + '<br>' +
                        '상태: <span style=\"color:green\">✓ 작동 중</span>';
                });
        }
    </script>
</body>
</html>
            ''')
        elif self.path == '/api/test':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({
                'time': datetime.now().strftime('%H:%M:%S'),
                'status': 'ok'
            }).encode())

PORT = 8888
with socketserver.TCPServer(('', PORT), Handler) as httpd:
    print(f'서버 실행 중: http://localhost:{PORT}')
    httpd.serve_forever()
" &

SERVER_PID=$!
sleep 2

# 4. Puppeteer 테스트 실행
echo ""
echo "🤖 브라우저 자동화 테스트 실행 중..."
echo "   (브라우저 창이 열립니다)"
echo ""

node demo/puppeteer-test.js

TEST_RESULT=$?

# 5. 서버 종료
kill $SERVER_PID 2>/dev/null || true

# 6. 결과 출력
echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 테스트 성공! 데모가 실제로 작동합니다."
    echo ""
    echo "📸 스크린샷 확인:"
    echo "   - 클릭 전: /tmp/before-click.png"
    echo "   - 클릭 후: /tmp/after-click.png"
    
    # macOS에서 스크린샷 열기
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/before-click.png /tmp/after-click.png 2>/dev/null || true
    fi
else
    echo "❌ 테스트 실패"
fi

exit $TEST_RESULT