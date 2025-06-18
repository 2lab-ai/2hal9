#!/bin/bash
# AI Genius Game - Puppeteer 자동화 테스트

set -e

echo "🎮 AI Genius Game - 상업용 수준 테스트"
echo "========================================"
echo ""

# 1. 프로젝트 루트로 이동
cd "$(dirname "$0")/.."
echo "📁 프로젝트 디렉토리: $(pwd)"
echo ""

# 2. Puppeteer 확인 (이미 설치됨)
if [ ! -d "node_modules/puppeteer" ]; then
    echo "❌ Puppeteer가 설치되어 있지 않습니다."
    exit 1
fi

# 3. AI Genius Game 빌드
echo "🔨 AI Genius Game 빌드 중..."
cd demo/ai-genius-game

if cargo build --release 2>/dev/null; then
    echo "   ✅ 빌드 성공"
else
    echo "   ❌ 빌드 실패"
    exit 1
fi

# 4. 기존 서버 프로세스 종료
echo ""
echo "🔄 기존 서버 프로세스 정리..."
pkill -f "ai-genius-game" 2>/dev/null || true
sleep 1

# 5. 서버 시작
echo ""
echo "🚀 AI Genius Game 서버 시작 중..."
./target/release/ai-genius-game &
SERVER_PID=$!

# 서버 시작 대기
echo "   서버 시작 대기중..."
for i in {1..10}; do
    if curl -s http://localhost:3456 >/dev/null 2>&1; then
        echo "   ✅ 서버 준비 완료"
        break
    fi
    sleep 1
done

# 서버 확인
if ! curl -s http://localhost:3456 >/dev/null 2>&1; then
    echo "   ❌ 서버 시작 실패"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# 6. Puppeteer 테스트 실행
echo ""
echo "🤖 Puppeteer 브라우저 자동화 테스트 시작..."
echo ""

cd ../..
node demo/ai-genius-game-test.js

TEST_RESULT=$?

# 7. 서버 종료
echo ""
echo "🛑 서버 종료 중..."
kill $SERVER_PID 2>/dev/null || true

# 8. 결과 출력
echo ""
echo "========================================"
if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 테스트 성공! AI Genius Game이 상업용 수준으로 작동합니다"
    echo ""
    echo "🎮 게임 기능:"
    echo "   - 실시간 WebSocket 통신"
    echo "   - 의식 출현 게임 메카닉"
    echo "   - HAL9 집단 지능 vs 개별 AI 대결"
    echo "   - 상업용 수준의 UI/UX"
    echo ""
    echo "📸 스크린샷 확인:"
    echo "   - 메뉴 화면: /tmp/ai-genius-game-menu.png"
    echo "   - 최종 상태: /tmp/ai-genius-game-final.png"
    
    # macOS에서 스크린샷 열기
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/ai-genius-game-menu.png /tmp/ai-genius-game-final.png 2>/dev/null || true
    fi
else
    echo "❌ 테스트 실패"
    echo ""
    echo "📸 에러 스크린샷: /tmp/ai-genius-game-error.png"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/ai-genius-game-error.png 2>/dev/null || true
    fi
fi
echo "========================================"

exit $TEST_RESULT