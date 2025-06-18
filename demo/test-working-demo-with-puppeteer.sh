#!/bin/bash
# HAL9 Working Demo - Puppeteer 자동화 테스트

set -e

echo "🧪 HAL9 Working Demo - Puppeteer 자동화 테스트"
echo "=============================================="
echo ""

# 1. Node.js 확인
if ! command -v node &> /dev/null; then
    echo "❌ Node.js가 설치되어 있지 않습니다."
    echo "   설치: brew install node (macOS) 또는 apt install nodejs (Linux)"
    exit 1
fi

# 2. 프로젝트 루트로 이동
cd "$(dirname "$0")/.."
echo "📁 프로젝트 디렉토리: $(pwd)"
echo ""

# 3. Puppeteer 설치 확인
if [ ! -d "node_modules/puppeteer" ]; then
    echo "📦 Puppeteer 설치 중..."
    npm init -y >/dev/null 2>&1 || true
    npm install puppeteer
    echo "   ✓ Puppeteer 설치 완료"
else
    echo "✓ Puppeteer 이미 설치됨"
fi

# 4. Working Demo 빌드
echo ""
echo "🔨 Working Demo 빌드 중..."
cd demo/working-demo

if cargo build --release; then
    echo "   ✅ 빌드 성공"
else
    echo "   ❌ 빌드 실패"
    exit 1
fi

# 5. 기존 서버 프로세스 종료
echo ""
echo "🔄 기존 서버 프로세스 정리..."
pkill -f "hal9-working-demo" 2>/dev/null || true
sleep 1

# 6. 서버 시작
echo ""
echo "🚀 HAL9 서버 시작 중..."
./target/release/hal9-working-demo &
SERVER_PID=$!

# 서버 시작 대기
echo "   서버 시작 대기중..."
for i in {1..10}; do
    if curl -s http://localhost:3333 >/dev/null 2>&1; then
        echo "   ✅ 서버 준비 완료"
        break
    fi
    sleep 1
done

# 서버 확인
if ! curl -s http://localhost:3333 >/dev/null 2>&1; then
    echo "   ❌ 서버 시작 실패"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# 7. Puppeteer 테스트 실행
echo ""
echo "🤖 Puppeteer 브라우저 자동화 테스트 시작..."
echo "   (헤드리스 모드로 실행 - 백그라운드에서 동작)"
echo ""

cd ../..
node demo/working-demo-puppeteer-test.js

TEST_RESULT=$?

# 8. 서버 종료
echo ""
echo "🛑 서버 종료 중..."
kill $SERVER_PID 2>/dev/null || true

# 9. 결과 출력
echo ""
echo "=============================================="
if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 테스트 성공! HAL9 데모가 완벽하게 작동합니다"
    echo ""
    echo "📸 스크린샷 확인:"
    echo "   - 초기 상태: /tmp/hal9-demo-initial.png"
    echo "   - 최종 상태: /tmp/hal9-demo-final.png"
    
    # macOS에서 스크린샷 열기
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/hal9-demo-initial.png /tmp/hal9-demo-final.png 2>/dev/null || true
    fi
else
    echo "❌ 테스트 실패"
    echo ""
    echo "📸 에러 스크린샷: /tmp/hal9-demo-error.png"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open /tmp/hal9-demo-error.png 2>/dev/null || true
    fi
fi
echo "=============================================="

exit $TEST_RESULT