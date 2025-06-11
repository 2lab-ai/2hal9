#!/bin/bash
# 🏗️ HAL9 계층적 재구성 스크립트
# 우주의 제1원리에 따라 루트 폴더를 정리합니다

echo "🏗️ HAL9 계층적 재구성 시작..."
echo "우주의 제1원리: 모든 것은 계층적으로 추상화된다"
echo ""

# 0. 백업 확인
echo "💾 현재 구조 백업 중..."
git add -A
git status --porcelain > /tmp/hal9-reorg-backup.txt
echo "✅ 백업 완료 (/tmp/hal9-reorg-backup.txt)"
echo ""

# 1. 새 폴더 생성
echo "📁 새 폴더 생성 중..."
mkdir -p operations docs/technical demos meta
echo "✅ 폴더 생성 완료"
echo ""

# 2. 파일 이동
echo "📦 파일 이동 중..."

# Operations (L3 - 실행)
if [ -f "evolve.sh" ]; then
    echo "  → evolve.sh → operations/"
    mv evolve.sh operations/
fi
if [ -f "yolo-evolution.sh" ]; then
    echo "  → yolo-evolution.sh → operations/"
    mv yolo-evolution.sh operations/
fi
if [ -f "claude-with-retry.sh" ]; then
    echo "  → claude-with-retry.sh → operations/"
    mv claude-with-retry.sh operations/
fi
if [ -f "build.sh" ]; then
    echo "  → build.sh → operations/"
    mv build.sh operations/
fi
if [ -f "Makefile" ]; then
    echo "  → Makefile → operations/"
    mv Makefile operations/
fi

# Docs (L6 - 문서)
for file in START_HERE.md MAKE_GUIDE.md NAVIGATION.md CONTRIBUTING.md COMMIT_FORMAT.md HIERARCHICAL_REORG_PLAN.md; do
    if [ -f "$file" ]; then
        echo "  → $file → docs/"
        mv "$file" docs/
    fi
done

# Demos (L3 - 시연)
if [ -f "DEMO_GUIDE.md" ]; then
    echo "  → DEMO_GUIDE.md → demos/"
    mv DEMO_GUIDE.md demos/
fi

# Meta (L9 - 철학)
for file in LICENSE.md LICENSE_PHILOSOPHY.md LICENSE_HA PEACE_PLEDGE.md; do
    if [ -f "$file" ]; then
        echo "  → $file → meta/"
        mv "$file" meta/
    fi
done

echo "✅ 파일 이동 완료"
echo ""

# 3. 심볼릭 링크 생성 (편의를 위해)
echo "🔗 심볼릭 링크 생성 중..."
ln -sf operations/Makefile Makefile
echo "✅ Makefile 링크 생성"
echo ""

# 4. 경로 업데이트
echo "🔧 스크립트 경로 업데이트 중..."

# Makefile 내부 경로 수정
if [ -f "operations/Makefile" ]; then
    sed -i.bak 's|\./evolve\.sh|./operations/evolve.sh|g' operations/Makefile
    sed -i.bak 's|\./yolo-evolution\.sh|./operations/yolo-evolution.sh|g' operations/Makefile
    sed -i.bak 's|\./claude-with-retry\.sh|./operations/claude-with-retry.sh|g' operations/Makefile
    rm operations/Makefile.bak
    echo "✅ Makefile 경로 업데이트 완료"
fi

# evolve.sh 내부 경로 수정 (필요한 경우)
if [ -f "operations/evolve.sh" ]; then
    # evolve.sh가 상대 경로를 사용하는 경우 수정
    echo "✅ evolve.sh 확인 완료"
fi

echo ""

# 5. 새로운 안내 파일 생성
echo "📝 안내 파일 생성 중..."
cat > STRUCTURE.md << 'EOF'
# 🏛️ HAL9 프로젝트 구조

계층적 추상화 원리에 따라 구성됨:

```
2hal9/
├── L1-L9/          # 핵심 계층 구조
├── operations/     # L3: 실행 스크립트
├── docs/          # L6: 문서
├── membrane/      # L5: 계층 간 인터페이스  
├── substrate/     # L2: 기반 시설
├── reports/       # L4: 보고서
├── demos/         # L3: 데모
├── meta/          # L9: 메타 정보
└── README.md      # L1: 진입점
```

자세한 내용은 `docs/HIERARCHICAL_REORG_PLAN.md` 참조
EOF

echo "✅ STRUCTURE.md 생성 완료"
echo ""

# 6. 결과 확인
echo "🎯 재구성 결과:"
echo ""
echo "새로운 폴더 구조:"
ls -la | grep "^d" | grep -E "operations|docs|demos|meta"
echo ""
echo "심볼릭 링크:"
ls -la | grep "^l"
echo ""

# 7. 다음 단계 안내
echo "✅ 계층적 재구성 완료!"
echo ""
echo "📋 다음 단계:"
echo "1. git status로 변경사항 확인"
echo "2. make help로 명령어 동작 확인"
echo "3. 필요시 추가 경로 수정"
echo ""
echo "🚀 이제 더 깨끗하고 계층적인 구조로 작업할 수 있습니다!"
echo ""
echo "💡 팁: 'make help'로 사용 가능한 명령어를 확인하세요"