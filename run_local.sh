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

HAL9_CONFIG=config.local.toml cargo run --bin hal9-server
