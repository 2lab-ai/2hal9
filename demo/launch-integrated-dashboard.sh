#!/bin/bash
# HAL9 통합 대시보드 실행 스크립트

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

# ASCII Art
echo -e "${PURPLE}"
cat << "EOF"
 _   _    _    _     ___    ____            _     _                         _ 
| | | |  / \  | |   / _ \  |  _ \  __ _ ___| |__ | |__   ___   __ _ _ __ __| |
| |_| | / _ \ | |  | (_) | | | | |/ _` / __| '_ \| '_ \ / _ \ / _` | '__/ _` |
|  _  |/ ___ \| |___\__, | | |_| | (_| \__ \ | | | |_) | (_) | (_| | | | (_| |
|_| |_/_/   \_\_____|  /_  |____/ \__,_|___/_| |_|_.__/ \___/ \__,_|_|  \__,_|
                                                                               
                 Integrated Dashboard v1.0
EOF
echo -e "${NC}"

echo -e "${BLUE}🌌 HAL9 통합 대시보드를 시작합니다...${NC}\n"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${YELLOW}⚠️  Python 3가 필요합니다. 설치해주세요.${NC}"
    exit 1
fi

# Change to dashboard directory
cd "$(dirname "$0")/../layers/L3_operational/architecture/dashboard"

# Check if dashboard files exist
if [ ! -f "dashboard-server-simple.py" ]; then
    echo -e "${YELLOW}⚠️  대시보드 서버 파일을 찾을 수 없습니다.${NC}"
    exit 1
fi

if [ ! -f "integrated-dashboard.html" ]; then
    echo -e "${YELLOW}⚠️  대시보드 HTML 파일을 찾을 수 없습니다.${NC}"
    exit 1
fi

# Kill existing dashboard process
echo -e "${YELLOW}🧹 기존 대시보드 프로세스 정리 중...${NC}"
pkill -f "dashboard-server-simple.py" 2>/dev/null || true
sleep 1

# Start dashboard server
echo -e "${GREEN}🚀 대시보드 서버를 시작합니다...${NC}"
python3 dashboard-server-simple.py &
DASHBOARD_PID=$!

echo -e "\n${GREEN}✅ 대시보드가 시작되었습니다!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "📊 ${GREEN}대시보드 URL: http://localhost:8080${NC}"
echo -e "📡 ${GREEN}메트릭 API: http://localhost:8080/api/dashboard/metrics${NC}"
echo -e "📁 ${GREEN}프로세스 ID: $DASHBOARD_PID${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Open in browser (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    sleep 2
    echo -e "\n${BLUE}🌐 브라우저에서 대시보드를 엽니다...${NC}"
    open "http://localhost:8080"
fi

echo -e "\n${YELLOW}💡 팁:${NC}"
echo -e "   • 대시보드는 실시간으로 메트릭을 업데이트합니다"
echo -e "   • 다른 HAL9 서비스와 함께 사용하면 더 많은 정보를 볼 수 있습니다"
echo -e "   • 종료하려면 Ctrl+C를 누르세요"

# Trap for cleanup
trap "echo -e '\n${YELLOW}대시보드를 종료합니다...${NC}'; kill $DASHBOARD_PID 2>/dev/null; exit" INT TERM

# Keep running
echo -e "\n${GREEN}대시보드가 실행 중입니다. Ctrl+C로 종료하세요.${NC}"
wait $DASHBOARD_PID