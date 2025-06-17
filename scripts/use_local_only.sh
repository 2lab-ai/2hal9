#!/bin/bash

# HAL9 로컬 전용 모드 설정 스크립트
# 모든 외부 의존성을 제거하고 로컬에서만 실행

echo "🏠 HAL9 로컬 전용 모드 설정"
echo "=============================="
echo ""

# .env.local 파일 생성
cat > .env.local << 'EOF'
# 로컬 전용 설정
HAL9_ENV=local
RUST_LOG=info,hal9=debug

# Claude Mock 모드
CLAUDE_MODE=mock

# 로컬 SQLite 사용 (PostgreSQL 대신)
DATABASE_URL=sqlite://hal9_local.db
DATABASE_MAX_CONNECTIONS=5

# Redis 비활성화
REDIS_ENABLED=false

# 외부 API 비활성화
OLLAMA_ENABLED=false
OPENAI_ENABLED=false
BEDROCK_ENABLED=false

# 로컬 서버 설정
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# 모니터링 비활성화
PROMETHEUS_ENABLED=false
JAEGER_ENABLED=false

# 네트워크 기능 비활성화
NETWORK_ENABLED=false

# Mock 응답 설정
MOCK_DELAY_MS=100
MOCK_DEFAULT_RESPONSE="로컬 모드에서 실행 중입니다"
EOF

echo "✅ .env.local 파일 생성됨"

# config.local.toml 생성
cat > config.local.toml << 'EOF'
[server]
server_id = "local-hal9"

[[neurons]]
id = "local-neuron-l1"
layer = "L1"
has_api = false

[[neurons]]
id = "local-neuron-l2"
layer = "L2"
has_api = false

[claude]
mode = "mock"
fallback_to_mock = true
temperature = 0.7
max_tokens = 1000

[claude.mock_responses.L1]
[[claude.mock_responses.L1]]
trigger = "default"
response = "L1 로컬 응답: 즉각적인 반응을 처리합니다."

[claude.mock_responses.L2]
[[claude.mock_responses.L2]]
trigger = "default"
response = "L2 로컬 응답: 실행 계층에서 작업을 수행합니다."

[memory]
enabled = false

[network]
enabled = false

[auth]
enabled = false

[monitoring]
enabled = false
EOF

echo "✅ config.local.toml 파일 생성됨"

# SQLite 데이터베이스 초기화
if [ ! -f "hal9_local.db" ]; then
    echo "📦 로컬 데이터베이스 생성 중..."
    sqlite3 hal9_local.db < migrations/sqlite/001_initial.sql 2>/dev/null || echo "⚠️  데이터베이스 마이그레이션 파일 없음 (괜찮음)"
fi

# 실행 스크립트 생성
cat > run_local.sh << 'EOF'
#!/bin/bash

# 환경 변수 로드
set -a
source .env.local
set +a

# 로컬 모드로 실행
echo "🚀 HAL9 로컬 모드 시작..."
echo ""
echo "설정:"
echo "  - Claude: Mock 모드"
echo "  - Database: SQLite (로컬)"
echo "  - Redis: 비활성화"
echo "  - 외부 API: 모두 비활성화"
echo ""

cargo run --bin hal9-server -- --config config.local.toml
EOF

chmod +x run_local.sh

echo ""
echo "✅ 설정 완료!"
echo ""
echo "실행 방법:"
echo "  ./run_local.sh"
echo ""
echo "특징:"
echo "  - 외부 의존성 없음 ✅"
echo "  - 인터넷 연결 불필요 ✅"
echo "  - 로컬에서만 실행 ✅"
echo "  - Mock 응답 사용 ✅"