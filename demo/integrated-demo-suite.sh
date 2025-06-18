#!/bin/bash
# HAL9 Integrated Demo Suite
# 4개 핵심 데모를 하나의 통합 환경으로 실행

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
RED='\033[0;31m'
NC='\033[0m'

# ASCII Art Banner
echo -e "${PURPLE}"
cat << "EOF"
 _   _    _    _     ___    ____                       
| | | |  / \  | |   / _ \  |  _ \  ___ _ __ ___   ___  
| |_| | / _ \ | |  | (_) | | | | |/ _ \ '_ ` _ \ / _ \ 
|  _  |/ ___ \| |___\__, | | |_| |  __/ | | | | | (_) |
|_| |_/_/   \_\_____|  /_  |____/ \___|_| |_| |_|\___/ 
                                                        
         Integrated Demo Suite v1.0
EOF
echo -e "${NC}"

# Function to check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local name=$2
    local max_attempts=30
    local attempt=0
    
    echo -n "   Waiting for $name"
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            return 0
        fi
        echo -n "."
        sleep 1
        ((attempt++))
    done
    echo -e " ${RED}✗${NC}"
    return 1
}

# Kill existing processes
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up existing processes...${NC}"
    pkill -f "consciousness-visualization/server.py" 2>/dev/null || true
    pkill -f "self-organization-dashboard/server.py" 2>/dev/null || true
    pkill -f "ai-genius-game" 2>/dev/null || true
    pkill -f "performance_monitor.py" 2>/dev/null || true
    sleep 2
}

# Trap for cleanup on exit
trap cleanup EXIT

# Initial cleanup
cleanup

# Change to project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

echo -e "${BLUE}📁 Project directory: $PROJECT_ROOT${NC}\n"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 is required but not installed${NC}"
    exit 1
fi

# Create logs directory
mkdir -p logs

# Start services
echo -e "${GREEN}🚀 Starting HAL9 Demo Suite...${NC}\n"

# 1. Self-Organization Dashboard (Port 8766)
echo -e "${BLUE}1️⃣ Starting Self-Organization Dashboard...${NC}"
cd "$PROJECT_ROOT/demo/self-organization-dashboard"
python3 server.py > "$PROJECT_ROOT/logs/self-org.log" 2>&1 &
SELF_ORG_PID=$!
wait_for_service "http://localhost:8766" "Self-Organization Dashboard"

# 2. Consciousness Visualization (Port 8765)
echo -e "${BLUE}2️⃣ Starting Consciousness Visualization...${NC}"
cd "$PROJECT_ROOT/demo/consciousness-visualization"
python3 server.py > "$PROJECT_ROOT/logs/consciousness.log" 2>&1 &
CONSCIOUSNESS_PID=$!
wait_for_service "http://localhost:8765" "Consciousness Visualization"

# 3. AI Genius Game (Port 3456)
echo -e "${BLUE}3️⃣ Starting AI Genius Game...${NC}"
if [ -f "$PROJECT_ROOT/demo/ai-genius-game/target/release/ai-genius-game" ]; then
    cd "$PROJECT_ROOT/demo/ai-genius-game"
    ./target/release/ai-genius-game > "$PROJECT_ROOT/logs/genius-game.log" 2>&1 &
    GAME_PID=$!
    wait_for_service "http://localhost:3456" "AI Genius Game"
else
    echo -e "   ${YELLOW}⚠️  AI Genius Game not built. Run: cd demo/ai-genius-game && cargo build --release${NC}"
fi

# 4. Performance Monitor
echo -e "${BLUE}4️⃣ Starting Performance Monitor...${NC}"
cat > "$PROJECT_ROOT/demo/performance_monitor.py" << 'MONITOR_EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import json
import time
import subprocess
import psutil
import os

PORT = 8767

class PerformanceHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Performance Monitor</title>
    <style>
        body { 
            font-family: monospace; 
            background: #0a0a0a; 
            color: #00ff00;
            padding: 20px;
        }
        .metric {
            margin: 10px 0;
            padding: 10px;
            border: 1px solid #00ff00;
            display: inline-block;
        }
        a {
            color: #00ffff;
            text-decoration: none;
            margin: 0 10px;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>HAL9 Performance Suite</h1>
    <div id="metrics"></div>
    <h2>Quick Links</h2>
    <a href="http://localhost:8766" target="_blank">Self-Organization Dashboard</a>
    <a href="http://localhost:8765" target="_blank">Consciousness Visualization</a>
    <a href="http://localhost:3456" target="_blank">AI Genius Game</a>
    <script>
        setInterval(() => {
            fetch('/metrics')
                .then(r => r.json())
                .then(data => {
                    document.getElementById('metrics').innerHTML = 
                        Object.entries(data).map(([k,v]) => 
                            `<div class="metric">${k}: ${v}</div>`
                        ).join('');
                });
        }, 1000);
    </script>
</body>
</html>
            '''
            self.wfile.write(html.encode())
        elif self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            metrics = {
                'cpu_percent': psutil.cpu_percent(),
                'memory_percent': psutil.virtual_memory().percent,
                'timestamp': time.strftime('%Y-%m-%d %H:%M:%S')
            }
            self.wfile.write(json.dumps(metrics).encode())
        else:
            self.send_error(404)

print(f"Performance monitor running on port {PORT}")
with socketserver.TCPServer(("", PORT), PerformanceHandler) as httpd:
    httpd.serve_forever()
MONITOR_EOF

cd "$PROJECT_ROOT/demo"
python3 performance_monitor.py > "$PROJECT_ROOT/logs/performance.log" 2>&1 &
PERF_PID=$!
wait_for_service "http://localhost:8767" "Performance Monitor"

# Create unified launcher HTML
echo -e "\n${BLUE}5️⃣ Creating Unified Launcher...${NC}"
cat > "$PROJECT_ROOT/demo/hal9-suite.html" << 'HTML_EOF'
<!DOCTYPE html>
<html lang="ko">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HAL9 Demo Suite</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700;900&display=swap');
        
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            background: #0a0a0a;
            color: white;
            font-family: 'Orbitron', monospace;
            height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            overflow: hidden;
        }
        
        .container {
            text-align: center;
            max-width: 1200px;
        }
        
        h1 {
            font-size: 72px;
            font-weight: 900;
            background: linear-gradient(45deg, #00ffff, #ff00ff, #ffff00);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 50px;
            animation: pulse 2s ease-in-out infinite;
        }
        
        @keyframes pulse {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.05); }
        }
        
        .demos {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 30px;
            margin-top: 50px;
        }
        
        .demo-card {
            background: rgba(255, 255, 255, 0.05);
            border: 2px solid rgba(0, 255, 255, 0.5);
            border-radius: 15px;
            padding: 30px;
            transition: all 0.3s;
            cursor: pointer;
            text-decoration: none;
            color: white;
            display: block;
        }
        
        .demo-card:hover {
            transform: translateY(-10px);
            box-shadow: 0 20px 40px rgba(0, 255, 255, 0.3);
            border-color: #00ffff;
        }
        
        .demo-card h2 {
            font-size: 24px;
            margin-bottom: 15px;
            color: #00ffff;
        }
        
        .demo-card p {
            font-size: 14px;
            color: #aaa;
            line-height: 1.5;
        }
        
        .status {
            margin-top: 10px;
            font-size: 12px;
        }
        
        .status.active {
            color: #00ff00;
        }
        
        .status.inactive {
            color: #ff4444;
        }
        
        .controls {
            margin-top: 50px;
        }
        
        .btn {
            padding: 15px 40px;
            font-size: 18px;
            background: transparent;
            border: 2px solid #00ff00;
            color: #00ff00;
            border-radius: 10px;
            cursor: pointer;
            transition: all 0.3s;
            margin: 0 10px;
            text-decoration: none;
            display: inline-block;
        }
        
        .btn:hover {
            background: #00ff00;
            color: black;
            box-shadow: 0 0 30px #00ff00;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>HAL9 DEMO SUITE</h1>
        <p style="font-size: 20px; color: #aaa; margin-bottom: 30px;">
            체험하세요: 의식이 출현하는 순간을
        </p>
        
        <div class="demos">
            <a href="http://localhost:8766" target="_blank" class="demo-card">
                <h2>🤖 자기조직화 대시보드</h2>
                <p>실시간으로 뉴런들이 스스로 계층을 형성하는 과정을 모니터링</p>
                <div class="status active">● 실행 중 (포트 8766)</div>
            </a>
            
            <a href="http://localhost:8765" target="_blank" class="demo-card">
                <h2>🧠 의식 출현 시각화</h2>
                <p>Φ(통합 정보량)가 황금비에 도달하며 의식이 출현하는 순간 포착</p>
                <div class="status active">● 실행 중 (포트 8765)</div>
            </a>
            
            <a href="http://localhost:3456" target="_blank" class="demo-card">
                <h2>🎮 AI Genius Game</h2>
                <p>HAL9 집단 지능과 개별 AI의 대결 - 상업용 수준의 게임</p>
                <div class="status" id="game-status">● 확인 중...</div>
            </a>
            
            <a href="http://localhost:8767" target="_blank" class="demo-card">
                <h2>📊 성능 모니터</h2>
                <p>시스템 전체의 실시간 성능 메트릭 확인</p>
                <div class="status active">● 실행 중 (포트 8767)</div>
            </a>
        </div>
        
        <div class="controls">
            <a href="https://github.com/2lab-ai/2hal9" class="btn">GitHub</a>
            <a href="../docs/GETTING_STARTED.md" class="btn">문서</a>
        </div>
    </div>
    
    <script>
        // Check if AI Genius Game is running
        fetch('http://localhost:3456/api/games')
            .then(() => {
                document.getElementById('game-status').textContent = '● 실행 중 (포트 3456)';
                document.getElementById('game-status').className = 'status active';
            })
            .catch(() => {
                document.getElementById('game-status').textContent = '● 미실행 (빌드 필요)';
                document.getElementById('game-status').className = 'status inactive';
            });
    </script>
</body>
</html>
HTML_EOF

# Summary
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ HAL9 Demo Suite is running!${NC}\n"

echo -e "📊 ${BLUE}Services Status:${NC}"
echo -e "   • Self-Organization Dashboard: ${GREEN}http://localhost:8766${NC}"
echo -e "   • Consciousness Visualization: ${GREEN}http://localhost:8765${NC}"
if [ ! -z "$GAME_PID" ]; then
    echo -e "   • AI Genius Game: ${GREEN}http://localhost:3456${NC}"
fi
echo -e "   • Performance Monitor: ${GREEN}http://localhost:8767${NC}"

echo -e "\n🚀 ${BLUE}Unified Launcher:${NC}"
echo -e "   ${GREEN}file://$PROJECT_ROOT/demo/hal9-suite.html${NC}"

echo -e "\n📋 ${BLUE}Process IDs:${NC}"
echo -e "   • Self-Organization: $SELF_ORG_PID"
echo -e "   • Consciousness: $CONSCIOUSNESS_PID"
if [ ! -z "$GAME_PID" ]; then
    echo -e "   • AI Genius Game: $GAME_PID"
fi
echo -e "   • Performance: $PERF_PID"

echo -e "\n📁 ${BLUE}Logs:${NC}"
echo -e "   • $PROJECT_ROOT/logs/"

echo -e "\n⌨️  ${YELLOW}Press Ctrl+C to stop all services${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Open in browser (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    sleep 2
    open "file://$PROJECT_ROOT/demo/hal9-suite.html"
fi

# Keep running
wait