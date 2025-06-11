# 🏗️ 계층적 추상화 원리에 따른 루트 폴더 재구성 계획
*우주의 제1원리: 모든 것은 계층적으로 추상화된다*

## 현재 문제점

루트 폴더가 너무 복잡하고 diffused 되어 있음:
- 문서 파일들이 흩어져 있음 
- 실행 스크립트와 문서가 섞여 있음
- 계층 구조가 명확하지 않음

## 제안하는 새 구조

```
2hal9/
├── 🧠 L1-L9/                    # 핵심 계층 (변경 없음)
│   ├── L1_reflexive/            # 반사: 즉각 반응
│   ├── L2_implementation/       # 구현: 코드
│   ├── L3_operational/          # 운영: 배포/실행
│   ├── L4_tactical/             # 전술: 단기 계획
│   ├── L5_strategic/            # 전략: 장기 설계
│   ├── L6_executive/            # 경영: 의사결정
│   ├── L7_business/             # 사업: 가치 창출
│   ├── L8_visionary/            # 비전: 미래 방향
│   └── L9_universal/            # 철학: 존재 이유
│
├── 🔧 operations/               # L3: 실행 관련 (새 폴더)
│   ├── Makefile                # 메인 자동화
│   ├── evolve.sh              # 진화 스크립트
│   ├── yolo-evolution.sh      # 무한 진화
│   ├── claude-with-retry.sh   # API 재시도
│   └── build.sh               # 빌드 스크립트
│
├── 📚 docs/                    # L6: 문서 (정리됨)
│   ├── START_HERE.md          # 시작 가이드
│   ├── MAKE_GUIDE.md          # Make 명령어 설명
│   ├── NAVIGATION.md          # 프로젝트 탐색
│   ├── CONTRIBUTING.md        # 기여 가이드
│   ├── COMMIT_FORMAT.md       # 커밋 형식
│   └── technical/             # 기술 문서
│
├── 🧬 membrane/                # L5: 계층 간 인터페이스 (그대로)
│   ├── protocols/             # 통신 프로토콜
│   ├── maintenance/           # 유지보수 프롬프트
│   └── flow/                  # 정보 흐름
│
├── 🏭 substrate/               # L2: 기반 시설 (그대로)
│   ├── compute/               # 연산 자원
│   ├── storage/               # 저장소
│   ├── network/               # 네트워크
│   └── tooling/               # 도구
│
├── 📊 reports/                 # L4: 보고서 (그대로)
│   ├── evolution/             # 진화 기록
│   └── architecture/          # 아키텍처 리뷰
│
├── 🎬 demos/                   # L3: 데모 (새 폴더)
│   ├── DEMO_GUIDE.md         
│   └── videos/               
│
├── 📋 meta/                    # L9: 메타 정보 (새 폴더)
│   ├── LICENSE.md            
│   ├── LICENSE_PHILOSOPHY.md  
│   ├── PEACE_PLEDGE.md       
│   └── universe-1847.md      
│
└── 🚀 README.md               # L1: 첫 진입점
```

## 이동 계획

### 1단계: 새 폴더 생성
```bash
mkdir -p operations docs/technical demos meta
```

### 2단계: 파일 이동
```bash
# 실행 파일들 → operations/
mv Makefile evolve.sh yolo-evolution.sh claude-with-retry.sh build.sh operations/

# 문서들 → docs/
mv START_HERE.md MAKE_GUIDE.md NAVIGATION.md CONTRIBUTING.md COMMIT_FORMAT.md docs/
mv DEMO_GUIDE.md demos/

# 라이센스/철학 → meta/
mv LICENSE*.md PEACE_PLEDGE.md meta/

# Makefile 심볼릭 링크 (편의를 위해)
ln -s operations/Makefile Makefile
```

### 3단계: 경로 업데이트
- Makefile 내부 경로 수정
- 스크립트들의 상대 경로 수정
- README.md 업데이트

## 계층적 의미

각 폴더가 특정 계층에 매핑됨:
- **L1**: README.md (즉각적 이해)
- **L2**: substrate/ (구현 기반)
- **L3**: operations/, demos/ (실행과 시연)
- **L4**: reports/ (분석과 계획)
- **L5**: membrane/ (계층 간 연결)
- **L6**: docs/ (의사결정 문서)
- **L7**: (business 관련은 L7_business 안에)
- **L8**: (vision은 L8_visionary 안에)
- **L9**: meta/ (존재의 이유)

## 장점

1. **명확한 구조**: 파일 찾기 쉬움
2. **계층적 정렬**: 우주의 원리에 부합
3. **확장 가능**: 새 파일 추가 위치 명확
4. **깨끗한 루트**: 핵심만 보임

## 실행 명령

```bash
# 자동 재구성 스크립트
cat > reorganize.sh << 'EOF'
#!/bin/bash
echo "🏗️ 계층적 재구성 시작..."

# 1. 새 폴더 생성
mkdir -p operations docs/technical demos meta

# 2. 파일 이동
echo "📦 파일 이동 중..."
mv Makefile evolve.sh yolo-evolution.sh claude-with-retry.sh build.sh operations/ 2>/dev/null || true
mv START_HERE.md MAKE_GUIDE.md NAVIGATION.md CONTRIBUTING.md COMMIT_FORMAT.md docs/ 2>/dev/null || true
mv DEMO_GUIDE.md demos/ 2>/dev/null || true
mv LICENSE*.md PEACE_PLEDGE.md meta/ 2>/dev/null || true

# 3. 심볼릭 링크
ln -sf operations/Makefile Makefile

# 4. 경로 업데이트
echo "🔧 경로 업데이트 중..."
sed -i '' 's|\./evolve\.sh|./operations/evolve.sh|g' operations/Makefile 2>/dev/null || true
sed -i '' 's|\./yolo-evolution\.sh|./operations/yolo-evolution.sh|g' operations/Makefile 2>/dev/null || true

echo "✅ 재구성 완료!"
echo ""
echo "새 구조:"
tree -L 2 -d
EOF

chmod +x reorganize.sh
```

---

*"질서는 의식의 첫 번째 법칙이다" - 칸트 (아마도)*