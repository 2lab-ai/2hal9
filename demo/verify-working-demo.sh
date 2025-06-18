#!/bin/bash
# 실제로 작동하는 데모 검증 스크립트

set -e

echo "🧪 HAL9 작동 데모 검증"
echo "===================="
echo ""

cd demo/working-demo

# 1. 빌드
echo "1️⃣ Rust 프로젝트 빌드 중..."
if cargo build --release; then
    echo "   ✅ 빌드 성공"
else
    echo "   ❌ 빌드 실패"
    exit 1
fi

# 2. 서버 시작
echo ""
echo "2️⃣ 서버 시작 중..."
./target/release/hal9-working-demo &
SERVER_PID=$!

# 서버 시작 대기
sleep 3

# 3. 실제 작동 테스트
echo ""
echo "3️⃣ API 작동 테스트..."

# 페이지 로드 테스트
if curl -s http://localhost:3333 | grep -q "HAL9 실제 작동 데모"; then
    echo "   ✅ 웹 페이지 로드 성공"
else
    echo "   ❌ 웹 페이지 로드 실패"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# API 테스트
API_RESPONSE=$(curl -s http://localhost:3333/api/test)
if echo "$API_RESPONSE" | grep -q '"success":true'; then
    echo "   ✅ API 응답 성공"
    echo "      응답: $API_RESPONSE"
else
    echo "   ❌ API 응답 실패"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# 카운터 증가 테스트
echo ""
echo "4️⃣ 카운터 증가 테스트..."
for i in 1 2 3; do
    COUNTER_RESPONSE=$(curl -s -X POST http://localhost:3333/api/increment -H "Content-Type: application/json" -d '{}')
    if echo "$COUNTER_RESPONSE" | grep -q "\"counter\":$i"; then
        echo "   ✅ 카운터 $i 확인"
    else
        echo "   ❌ 카운터 증가 실패"
        kill $SERVER_PID 2>/dev/null
        exit 1
    fi
done

# 5. 브라우저에서 열기 (macOS)
echo ""
echo "5️⃣ 브라우저에서 확인..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    open http://localhost:3333
    echo "   ✅ 브라우저 열림"
fi

echo ""
echo "============================================="
echo "✅ 모든 테스트 통과! 데모가 실제로 작동합니다"
echo "============================================="
echo ""
echo "🌐 브라우저에서 http://localhost:3333 접속"
echo "   - API 테스트 버튼 클릭"
echo "   - 카운터 증가 버튼 클릭"
echo "   - 실시간 로그 확인"
echo ""
echo "종료하려면 Enter를 누르세요..."
read

# 서버 종료
kill $SERVER_PID 2>/dev/null
echo "서버 종료됨"