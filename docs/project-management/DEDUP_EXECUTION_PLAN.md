# 🗑️ HAL9 중복 제거 실행 계획

## 0. 사전 준비

```bash
# 전체 백업 (혹시 모르니)
git add -A && git commit -m "backup: before massive deduplication"
git push origin main

# 새 브랜치 생성
git checkout -b dedup-cleanup
```

## 1. Phase 1: 즉시 삭제 (30분)

### 1.1 .bak 파일 전체 삭제
```bash
# 먼저 확인
find . -name "*.bak" -type f | wc -l
find . -name "*.bak" -type f

# 삭제
find . -name "*.bak" -type f -delete
```

### 1.2 빈 디렉토리 삭제
```bash
# 확인
find . -type d -empty | wc -l

# 삭제
find . -type d -empty -delete
```

### 1.3 .DS_Store 등 시스템 파일 삭제
```bash
find . -name ".DS_Store" -delete
find . -name "Thumbs.db" -delete
```

## 2. Phase 2: Legacy 제거 (1시간)

### 2.1 legacy-crates 제거
```bash
# 의존성 확인
grep -r "legacy-crates" . --include="*.toml" --include="*.rs"

# workspace.toml 수정 - legacy-crates 멤버 제거
vim substrate/tooling/rust/workspace.toml

# 디렉토리 삭제
rm -rf substrate/tooling/rust/legacy-crates/
```

### 2.2 workspace 통합
```bash
# 2hal9-minimal의 workspace를 메인으로 이동
# 먼저 차이점 분석
diff Cargo.toml 2hal9-minimal/Cargo.toml

# 필요한 멤버만 메인 workspace에 추가
# 2hal9-minimal/Cargo.toml 삭제
```

## 3. Phase 3: 서버 통합 (반나절)

### 3.1 어느 서버를 남길 것인가?

**분석 결과**: 
- `2hal9-minimal/core/hal9-server/` - 가장 최신, 기능 완전
- ~~`layers/L3_operational/architecture/server/`~~ - 삭제
- ~~`substrate/tooling/rust/legacy-crates/hal9-server/`~~ - 이미 삭제됨

### 3.2 서버 통합 작업
```bash
# 1. 유니크한 기능 확인
diff -r 2hal9-minimal/core/hal9-server/ layers/L3_operational/architecture/server/

# 2. 필요한 기능만 병합
# (수동 작업 필요)

# 3. 중복 제거
rm -rf layers/L3_operational/architecture/server/

# 4. import 경로 업데이트
grep -r "L3_operational/architecture/server" . --include="*.rs" --include="*.toml"
# 찾은 것들 모두 수정
```

## 4. Phase 4: 데모 정리 (반나절)

### 4.1 데모 분류 및 정리

**남길 데모** (examples/ 디렉토리로 통합):
- `simple_local_demo.rs` - 가장 간단한 로컬 데모
- `gentle_singularity_demo.rs` - 멀티스레드 데모
- `true_self_organization_demo.rs` - 자가조직화 (최고 버전 1개만)

**삭제할 데모**:
- 모든 중복 버전들
- .bak이 있는 것들
- 작동하지 않는 것들

### 4.2 실행
```bash
# 새 구조 생성
mkdir -p examples/{basic,advanced,experimental}

# 데모 이동 및 정리
mv examples/simple_local_demo.rs examples/basic/
mv examples/gentle_singularity_demo.rs examples/basic/
# ... 등등

# 중복 제거
rm -rf layers/L2_implementation/neurons/examples/
rm -rf 2hal9-minimal/examples/
```

## 5. Phase 5: 테스트 통합 (반나절)

### 5.1 테스트 구조 정리

**목표 구조**:
```
tests/
├── unit/
├── integration/
└── e2e/
```

### 5.2 중복 테스트 제거
```bash
# 중복 찾기
find . -name "*test*.rs" -type f | grep -v target | sort

# 통합
# 각 테스트 파일을 비교하고 최신/완전한 버전만 남기기
```

## 6. Phase 6: 모듈 정리 (1일)

### 6.1 neuron.rs 통합 (4개 → 1개)
```bash
# 비교
diff -u layers/L3_operational/architecture/server/neuron.rs \
        layers/L2_implementation/neurons/core/neuron.rs

# 가장 완전한 버전 선택하고 나머지 삭제
```

### 6.2 enterprise/, plugins/ 등 중복 모듈
- 한 곳으로 통합
- 나머지 삭제

## 7. Phase 7: 설정 파일 정리 (2시간)

### 7.1 Docker 관련
```bash
# docker-compose 파일들 통합
# 기본 + override 패턴 사용
docker-compose.yml          # 기본
docker-compose.override.yml # 개발용
docker-compose.prod.yml     # 프로덕션
```

### 7.2 기타 설정
- 중복 .env 파일 제거
- 중복 config.toml 제거

## 8. 검증 체크리스트

각 Phase 후 반드시 확인:

### □ 빌드 확인
```bash
cargo build --workspace
```

### □ 테스트 확인
```bash
cargo test --workspace
```

### □ 주요 데모 실행
```bash
cargo run --example simple_local_demo
```

### □ 문서 링크 확인
```bash
# 삭제한 파일을 참조하는 문서가 있는지
grep -r "삭제한파일명" . --include="*.md"
```

## 9. 예상 결과

### Before
- 파일 수: ~2,500개
- 코드 줄: 178,644
- 빌드 시간: 10분+

### After  
- 파일 수: ~1,000개 (-60%)
- 코드 줄: ~100,000 (-44%)
- 빌드 시간: ~5분 (-50%)

## 10. 위험 관리

### 실수로 중요한 것 삭제 시
```bash
# 특정 파일 복구
git checkout main -- path/to/file

# 전체 되돌리기
git checkout main
git branch -D dedup-cleanup
```

### 안전 규칙
1. **삭제 전 항상 diff 확인**
2. **기능이 겹치는지 확인**
3. **의존성 확인**
4. **단계별 커밋**

## 11. 실행 일정

- **Day 1 AM**: Phase 1-2 (즉시 삭제, Legacy 제거)
- **Day 1 PM**: Phase 3 (서버 통합)
- **Day 2 AM**: Phase 4 (데모 정리)
- **Day 2 PM**: Phase 5 (테스트 통합)
- **Day 3**: Phase 6-7 (모듈 정리, 설정 정리)
- **Day 4**: 최종 검증 및 문서 업데이트

## 12. 커밋 전략

각 Phase별로 별도 커밋:
```bash
git add -A
git commit -m "cleanup: Phase 1 - remove .bak files and empty dirs"

git add -A  
git commit -m "cleanup: Phase 2 - remove legacy-crates"

# ... 등등
```

## 13. 성공 지표

- [ ] 중복 코드 5% 미만
- [ ] 모든 테스트 통과
- [ ] 주요 데모 작동
- [ ] 빌드 시간 50% 감소
- [ ] 명확한 프로젝트 구조

---

**⚠️ 주의**: 삭제는 신중하게! 의심스러우면 일단 남기고 나중에 판단.

**💪 파이팅**: 깨끗한 코드베이스를 향해!