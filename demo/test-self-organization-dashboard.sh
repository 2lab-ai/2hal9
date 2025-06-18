#!/bin/bash
# Self-Organization Monitoring Dashboard - 테스트 스크립트

set -e

echo "🤖 Self-Organization Monitoring Dashboard 테스트"
echo "=============================================="
echo ""

# 1. 프로젝트 루트로 이동
cd "$(dirname "$0")/.."
echo "📁 프로젝트 디렉토리: $(pwd)"
echo ""

# 2. 기존 서버 프로세스 종료
echo "🔄 기존 서버 프로세스 정리..."
pkill -f "self-organization-dashboard/server.py" 2>/dev/null || true
sleep 1

# 3. 서버 시작
echo "🚀 Dashboard 서버 시작 중..."
cd demo/self-organization-dashboard
python3 server.py &
SERVER_PID=$!

# 서버 시작 대기
echo "   서버 시작 대기중..."
for i in {1..10}; do
    if curl -s http://localhost:8766 >/dev/null 2>&1; then
        echo "   ✅ 서버 준비 완료"
        break
    fi
    sleep 1
done

# 서버 확인
if ! curl -s http://localhost:8766 >/dev/null 2>&1; then
    echo "   ❌ 서버 시작 실패"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# 4. Puppeteer 테스트 실행
echo ""
echo "🤖 Puppeteer 브라우저 자동화 테스트 시작..."
echo ""

cd ../..
node demo/self-organization-dashboard-test.js

TEST_RESULT=$?

# 5. 서버 종료
echo ""
echo "🛑 서버 종료 중..."
kill $SERVER_PID 2>/dev/null || true

# 6. 결과 출력
echo ""
echo "=============================================="
if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 테스트 성공! Self-Organization Dashboard가 완벽하게 작동합니다"
    echo ""
    echo "📊 대시보드 특징:"
    echo "   - 실시간 뉴런 자기조직화 모니터링"
    echo "   - 레이어 출현 시각화"
    echo "   - 성능 메트릭 실시간 차트"
    echo "   - 네트워크 연결 매트릭스"
    echo "   - 시스템 상태 인디케이터"
    echo ""
    echo "📸 스크린샷:"
    echo "   - 초기: /tmp/self-org-dashboard-initial.png"
    echo "   - 최종: /tmp/self-org-dashboard-final.png"
    
    # macOS에서 스크린샷 열기
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/self-org-dashboard-initial.png /tmp/self-org-dashboard-final.png 2>/dev/null || true
    fi
else
    echo "❌ 테스트 실패"
    echo ""
    echo "📸 에러 스크린샷: /tmp/self-org-dashboard-error.png"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/self-org-dashboard-error.png 2>/dev/null || true
    fi
fi
echo "=============================================="

exit $TEST_RESULT