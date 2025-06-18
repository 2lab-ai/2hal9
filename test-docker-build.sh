#!/bin/bash
# Docker 빌드 테스트 스크립트

set -e

echo "🐳 HAL9 Docker 빌드 테스트"
echo "=========================="
echo ""

# 1. Docker 설치 확인
if ! command -v docker &> /dev/null; then
    echo "❌ Docker가 설치되어 있지 않습니다."
    echo "   설치: https://docs.docker.com/get-docker/"
    exit 1
fi

echo "✓ Docker 버전: $(docker --version)"

# 2. Docker Compose 설치 확인
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose가 설치되어 있지 않습니다."
    exit 1
fi

echo "✓ Docker Compose 버전: $(docker compose version 2>/dev/null || docker-compose --version)"

# 3. 빌드 컨텍스트 크기 확인
echo ""
echo "📦 빌드 컨텍스트 크기 확인..."
BUILD_SIZE=$(du -sh . 2>/dev/null | cut -f1)
echo "   전체 프로젝트 크기: $BUILD_SIZE"

# 4. Docker 이미지 빌드 (캐시 없이)
echo ""
echo "🔨 Docker 이미지 빌드 시작..."
echo "   (첫 빌드는 시간이 걸릴 수 있습니다)"

if docker build --no-cache -t hal9:test .; then
    echo "   ✅ Docker 이미지 빌드 성공!"
else
    echo "   ❌ Docker 이미지 빌드 실패"
    exit 1
fi

# 5. 이미지 크기 확인
echo ""
echo "📏 이미지 크기:"
docker images hal9:test --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

# 6. 컨테이너 실행 테스트
echo ""
echo "🚀 컨테이너 실행 테스트..."

# 기존 컨테이너 정리
docker stop hal9-test 2>/dev/null || true
docker rm hal9-test 2>/dev/null || true

# 컨테이너 실행
if docker run -d --name hal9-test -p 3456:3456 hal9:test; then
    echo "   ✅ 컨테이너 시작 성공"
    
    # 헬스체크 대기
    echo "   헬스체크 대기 중..."
    sleep 5
    
    # API 테스트
    if curl -s http://localhost:3456/api/games > /dev/null 2>&1; then
        echo "   ✅ API 응답 확인"
    else
        echo "   ❌ API 응답 없음"
        docker logs hal9-test
    fi
    
    # 컨테이너 정리
    docker stop hal9-test
    docker rm hal9-test
else
    echo "   ❌ 컨테이너 시작 실패"
fi

# 7. Docker Compose 테스트
echo ""
echo "🎼 Docker Compose 테스트..."
echo "   (전체 스택 실행 - DB, Redis 포함)"

# 환경 변수 설정
export CLAUDE_MODE=mock

# Compose 실행
if docker compose up -d; then
    echo "   ✅ Docker Compose 시작 성공"
    
    echo "   서비스 상태 확인..."
    sleep 10
    docker compose ps
    
    # 서비스 테스트
    echo ""
    echo "   서비스 테스트:"
    
    # HAL9 서버
    if curl -s http://localhost:3456/api/games > /dev/null 2>&1; then
        echo "   ✅ HAL9 서버: 정상"
    else
        echo "   ❌ HAL9 서버: 응답 없음"
    fi
    
    # Consciousness Viz
    if curl -s http://localhost:8765 > /dev/null 2>&1; then
        echo "   ✅ Consciousness Viz: 정상"
    else
        echo "   ❌ Consciousness Viz: 응답 없음"
    fi
    
    # Self-Org Dashboard
    if curl -s http://localhost:8766 > /dev/null 2>&1; then
        echo "   ✅ Self-Org Dashboard: 정상"
    else
        echo "   ❌ Self-Org Dashboard: 응답 없음"
    fi
    
    # Postgres
    if docker compose exec -T postgres pg_isready -U hal9 > /dev/null 2>&1; then
        echo "   ✅ PostgreSQL: 정상"
    else
        echo "   ❌ PostgreSQL: 응답 없음"
    fi
    
    # Redis
    if docker compose exec -T redis redis-cli ping > /dev/null 2>&1; then
        echo "   ✅ Redis: 정상"
    else
        echo "   ❌ Redis: 응답 없음"
    fi
    
    # 로그 샘플
    echo ""
    echo "📋 HAL9 서버 로그 (최근 10줄):"
    docker compose logs hal9-server --tail=10
    
    # 정리
    echo ""
    echo "🧹 서비스 종료 중..."
    docker compose down
else
    echo "   ❌ Docker Compose 시작 실패"
fi

echo ""
echo "=========================="
echo "✅ Docker 테스트 완료!"
echo ""
echo "프로덕션 배포 준비:"
echo "1. .env 파일에 실제 API 키 설정"
echo "2. docker compose up -d 로 실행"
echo "3. docker compose logs -f 로 로그 확인"
echo "=========================="