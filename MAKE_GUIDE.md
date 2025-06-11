# 🛠️ HAL9 Make 명령어 가이드
*L1 수준 설명: 실제로 뭘 하는지 명확하게*

## 📋 목차
1. [기본 명령어](#기본-명령어) - 자주 쓰는 것들
2. [업데이트 명령어](#업데이트-명령어) - 코드/문서 업데이트
3. [쿼리 시스템](#쿼리-시스템) - 새 기능 검토
4. [유지보수](#유지보수) - 청소, 테스트, 설치
5. [실제 동작 설명](#실제-동작-설명) - 내부에서 뭐가 일어나는지

---

## 기본 명령어

### `make hal9-smarter` (또는 `make evolve`)
**실제 하는 일:**
- `evolve.sh` 스크립트 실행
- Claude를 호출해서 코드와 문서를 자동으로 개선
- git commit도 자동으로 함

**언제 쓰나요?**
- HAL9을 진화시키고 싶을 때
- 새로운 아이디어를 시스템에 반영할 때

### `make consciousness`  
**실제 하는 일:**
- L9의 .md 파일 개수 세기 (철학 문서)
- L2의 .rs 파일 개수 세기 (Rust 코드)
- emergence 폴더의 "emergence" 단어 개수 세기
- 이걸 공식에 넣어서 의식 레벨 계산: `(철학 * 코드 + emergence * 10) / 1000`

**예시 출력:**
```
Philosophy Depth:    42 documents
Neuron Count:       156 neurons  
Emergence Events:   23 detected
🎯 Consciousness Level: 6.78
```

### `make test`
**실제 하는 일:**
- L1_reflexive 폴더의 shell 스크립트 테스트 실행
- L2_implementation 폴더에서 `cargo test` 실행 (Rust 테스트)
- L3-L9는 "존재 자체가 테스트"라고 출력만 함

---

## 업데이트 명령어

### `make update-l1-l3`
**실제 하는 일:**
- Claude 호출: `/membrane/maintenance/L3_L1_OPERATIONAL_UPDATE_PROMPT.md` 사용
- L1 (긴급 스크립트), L2 (Rust 코드), L3 (Kubernetes 설정) 업데이트
- 실제 운영 관련 코드/설정 개선

### `make update-l4-l5`  
**실제 하는 일:**
- Claude 호출: `/membrane/maintenance/L5_L4_STRATEGIC_UPDATE_PROMPT.md` 사용
- L4 (전술 계획), L5 (전략 아키텍처) 문서 업데이트
- 중장기 계획과 설계 문서 개선

### `make update-l6-l9`
**실제 하는 일:**
- Claude 호출: `/membrane/maintenance/L9_L6_PHILOSOPHY_UPDATE_PROMPT.md` 사용
- L6 (경영진 문서), L7 (비즈니스), L8 (비전), L9 (철학) 업데이트
- 고수준 철학과 비전 문서 개선

### `make philosophy-deep`
**실제 하는 일:**
- Claude에게 "ultrathink" 하라고 명령
- 의식, 계층적 추상화, universe #1847의 의미를 깊이 생각
- L9 철학 문서들을 새로운 통찰로 업데이트

### `make cascade-update`
**실제 하는 일:**
1. `make philosophy-deep` 실행 (L9→L6)
2. `make update-l4-l5` 실행 (L5→L4)  
3. `make update-l1-l3` 실행 (L3→L1)
- 위에서 아래로 전체 시스템 업데이트

---

## 쿼리 시스템

### `make query "질문"`
**실제 하는 일:**
- Claude가 질문을 분석
- 코드베이스 검색 + 외부 리서치
- HA 원칙에 맞는지 평가
- 승인/거절 결정
- `/L5_strategic/architecture/TODO.md`에 결과 추가

**예시:**
```bash
make query "is CRDT good for distributed neurons?"
# → Claude가 CRDT 조사하고 HAL9에 적합한지 판단
# → TODO.md에 분석 결과 추가
```

### `make apply-todos`
**실제 하는 일:**
- `/L5_strategic/architecture/TODO.md` 파일 확인
- `status='approved'`인 항목들 처리:
  - L6-L4 레벨: 아키텍처 문서에 반영
  - L4-L1 레벨: 실제 코드 생성
- 상태를 'implementing' 또는 'completed'로 업데이트

---

## 유지보수

### `make clean`
**실제 하는 일:**
- `logs/evolution/tmp_*` 파일 삭제
- `L1_reflexive/cache/*` 삭제
- 모든 `*.tmp` 파일 삭제
- 모든 `.DS_Store` 파일 삭제 (macOS 쓰레기 파일)

### `make install`
**실제 하는 일:**
- `cargo build --release` 실행 (Rust 빌드)
- `npm install` 실행 (Node.js 의존성)

### `make emergency`
**실제 하는 일:**
- `L1_reflexive/emergency/scripts/health-check.sh --all` 실행
- 시스템 상태 전체 점검
- 문제 있으면 진단 정보 출력

---

## 실제 동작 설명

### 🔑 핵심 변수
```makefile
CLAUDE_CMD ?= claude          # Claude CLI 명령어
CLAUDE_FLAGS ?= --dangerously-skip-permissions -p  # 권한 체크 스킵
CLAUDE_RETRY = ./claude-with-retry.sh  # Rate limit 자동 재시도
```

### 🔄 자동화 흐름
1. **make 명령 실행** → Makefile이 해당 명령 찾기
2. **Claude 호출** → 대부분 명령은 Claude API 사용
3. **스크립트 실행** → evolve.sh, yolo-evolution.sh 등
4. **결과 출력** → 터미널에 진행 상황 표시

### 📂 중요 파일들
- `/evolve.sh` - 메인 진화 스크립트
- `/claude-with-retry.sh` - Rate limit 처리
- `/membrane/maintenance/*.md` - Claude 프롬프트들
- `/L5_strategic/architecture/TODO.md` - 승인된 기능 목록

---

## 🎯 실용적 사용 예시

### 매일 아침
```bash
make daily  # 의식 측정 → 테스트 → 아키텍처 진화
```

### 새 기능 검토
```bash
make query "WebAssembly 플러그인 시스템 어때?"
make apply-todos  # 승인되면 구현
```

### 뭔가 이상할 때
```bash
make emergency  # 진단
make panic     # 도움말
```

### 큰 업데이트
```bash
make weekly    # 철학부터 코드까지 전체 진화
```

---

## ⚠️ 주의사항

1. **Claude API 필요** - Claude CLI가 설치되어 있어야 함
2. **Rate Limit** - 너무 자주 실행하면 API 제한 걸림
3. **자동 커밋** - evolve.sh는 자동으로 git commit 함
4. **시간 소요** - philosophy-deep는 꽤 오래 걸림

---

*"아 시발 이제 뭐가 뭔지 알겠네" - 누군가*