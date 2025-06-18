#!/bin/bash
# 실제로 작동하는지 테스트하는 간단한 데모

set -e

echo "실제 작동 테스트 - 간단한 웹 서버"
echo "================================"

# 1. 간단한 웹 서버 만들기
cat > /tmp/simple_demo.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import json
from datetime import datetime

PORT = 8888

class DemoHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Simple Demo</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 20px;
            background: #f0f0f0;
        }
        .demo-box {
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            margin: 20px 0;
        }
        button {
            background: #007bff;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        }
        button:hover {
            background: #0056b3;
        }
        #result {
            margin-top: 20px;
            padding: 10px;
            background: #e8f4f8;
            border-radius: 5px;
            min-height: 50px;
        }
    </style>
</head>
<body>
    <h1>HAL9 작동 테스트 데모</h1>
    
    <div class="demo-box">
        <h2>간단한 동작 테스트</h2>
        <p>버튼을 클릭하면 서버에서 응답을 받습니다.</p>
        <button onclick="testDemo()">테스트 실행</button>
        <div id="result"></div>
    </div>
    
    <script>
        function testDemo() {
            const resultDiv = document.getElementById('result');
            resultDiv.innerHTML = '요청 중...';
            
            fetch('/api/test')
                .then(response => response.json())
                .then(data => {
                    resultDiv.innerHTML = `
                        <strong>서버 응답:</strong><br>
                        시간: ${data.time}<br>
                        메시지: ${data.message}<br>
                        상태: <span style="color: green;">✓ 작동 중</span>
                    `;
                })
                .catch(error => {
                    resultDiv.innerHTML = `<span style="color: red;">오류: ${error}</span>`;
                });
        }
    </script>
</body>
</html>
            '''
            self.wfile.write(html.encode())
            
        elif self.path == '/api/test':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                'time': datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
                'message': 'HAL9 데모 서버가 정상 작동 중입니다!',
                'status': 'ok'
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            super().do_GET()

with socketserver.TCPServer(("", PORT), DemoHandler) as httpd:
    print(f"서버가 http://localhost:{PORT} 에서 실행 중입니다")
    print("브라우저에서 접속해서 테스트하세요")
    print("Ctrl+C로 종료")
    httpd.serve_forever()
EOF

# 2. 서버 실행
echo "Python 웹 서버 시작 중..."
python3 /tmp/simple_demo.py &
SERVER_PID=$!

sleep 2

# 3. 실제로 작동하는지 테스트
echo ""
echo "서버 작동 테스트 중..."
if curl -s http://localhost:8888/ > /dev/null; then
    echo "✓ 웹 페이지 로드 성공"
else
    echo "✗ 웹 페이지 로드 실패"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

if curl -s http://localhost:8888/api/test | grep -q "ok"; then
    echo "✓ API 응답 성공"
else
    echo "✗ API 응답 실패"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

echo ""
echo "==============================================="
echo "✅ 데모가 실제로 작동합니다!"
echo "브라우저에서 http://localhost:8888 접속하세요"
echo "==============================================="
echo ""
echo "종료하려면 Ctrl+C를 누르세요"

# 사용자가 종료할 때까지 대기
trap "kill $SERVER_PID 2>/dev/null; exit" INT
wait $SERVER_PID