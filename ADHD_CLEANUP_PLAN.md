# 🧹 ADHD 중복 정리 계획

## 현재 상황 분석

### CLAUDE.md에 정의된 정식 구조:
```
2hal9/
├── layers/      # HA 구현
├── substrate/   # 인프라 및 도구
└── demo/        # HAL9 뉴런 데모
```

### 실제 루트에 있는 것들 (ADHD가 만든 것):
```
k8s/            ❌ → layers/L3_operational/architecture/kubernetes/
monitoring/     ❌ → layers/L3_operational/workflows/monitoring/
nginx/          ❌ → layers/L3_operational/configuration/
ssl/            ❌ → layers/L3_operational/configuration/
scripts/        ❌ → layers/L3_operational/scripts/
docs/           ❌ → layers/L3_operational/documentation/ (또는 각 계층별)
docker-compose* ❌ → layers/L3_operational/configuration/docker/
.github/        ❓ → 루트에 있어야 함 (GitHub 요구사항)
tests/          ❓ → 각 계층별 tests/
examples/       ✅ → demo/와 통합 필요
```

## 중복 현황

### 1. Kubernetes (k8s)
- **루트**: `/k8s/` (간단한 2개 파일)
- **정식**: `/layers/L3_operational/architecture/kubernetes/` (완전한 구성)
- **결정**: 루트 삭제, 정식 위치 사용

### 2. Docker 관련
- **루트**: `docker-compose*.yml` (6개 파일)
- **정식**: `/layers/L3_operational/configuration/docker/`
- **결정**: 루트 파일들을 정식 위치로 이동

### 3. 모니터링
- **루트**: `/monitoring/`
- **정식**: `/layers/L3_operational/workflows/monitoring/`
- **결정**: 루트 삭제, 정식 위치 사용

### 4. Scripts
- **루트**: `/scripts/`
- **정식**: 
  - `/layers/L1_reflexive/*/scripts/` (긴급 스크립트)
  - `/layers/L3_operational/scripts/` (운영 스크립트)
- **결정**: 용도별로 적절한 계층으로 이동

### 5. SSL/Nginx
- **루트**: `/ssl/`, `/nginx/`
- **정식**: `/layers/L3_operational/configuration/`
- **결정**: configuration 아래로 이동

### 6. Documentation
- **루트**: `/docs/`
- **정식**: 각 계층별 문서화 또는 L3_operational/documentation/
- **결정**: 계층별로 분산 또는 중앙화

## 실행 계획

### Phase 1: 백업 및 분석
```bash
# 1. 현재 상태 백업
git add -A && git commit -m "backup: before ADHD cleanup"
git tag adhd-cleanup-backup

# 2. 중복 내용 상세 비교
diff -r k8s/ layers/L3_operational/architecture/kubernetes/
diff -r monitoring/ layers/L3_operational/workflows/monitoring/
```

### Phase 2: K8s 정리
```bash
# 루트 k8s가 더 간단하므로 삭제
rm -rf k8s/
# 필요한 경우 layers의 kubernetes를 업데이트
```

### Phase 3: Docker 설정 이동
```bash
# docker-compose 파일들을 정식 위치로
mv docker-compose*.yml layers/L3_operational/configuration/docker/
```

### Phase 4: 인프라 디렉토리 정리
```bash
# monitoring 이동 또는 병합
# 내용 확인 후 결정
mv monitoring/* layers/L3_operational/workflows/monitoring/
rm -rf monitoring/

# nginx, ssl 이동
mv nginx/ layers/L3_operational/configuration/
mv ssl/ layers/L3_operational/configuration/
```

### Phase 5: Scripts 정리
```bash
# 운영 스크립트는 L3로
mv scripts/*.sh layers/L3_operational/scripts/
# 긴급 스크립트는 L1로
# 배포 스크립트는 L3/configuration/deployment/로
```

### Phase 6: Docs 정리
```bash
# API 문서는 L3/architecture/server/docs/
# 배포 문서는 L3/configuration/deployment/docs/
# 일반 문서는 각 계층별로
```

### Phase 7: 나머지 정리
```bash
# examples와 demo 통합
# tests는 각 계층별로 분산
```

## 예외 사항

### 루트에 남겨야 하는 것들:
1. `.github/` - GitHub Actions 요구사항
2. `.cargo/` - Rust 설정
3. `Cargo.toml` (심볼릭 링크)
4. `README.md`
5. `CLAUDE.md`, `CLAUDE.LOCAL.md`
6. 기타 설정 파일들 (.gitignore, rust-toolchain.toml 등)

## 정리 후 구조

```
2hal9/
├── layers/
│   ├── L1_reflexive/
│   │   └── */scripts/           # 긴급 스크립트
│   ├── L2_implementation/
│   └── L3_operational/
│       ├── architecture/
│       │   ├── server/
│       │   ├── kubernetes/      # K8s 설정
│       │   └── browser/
│       ├── configuration/
│       │   ├── docker/          # Docker 설정
│       │   ├── nginx/           # Nginx 설정
│       │   ├── ssl/             # SSL 인증서
│       │   └── deployment/      # 배포 설정
│       ├── workflows/
│       │   └── monitoring/      # 모니터링 설정
│       ├── scripts/             # 운영 스크립트
│       └── documentation/       # 중앙 문서 (선택)
├── substrate/
└── demo/                        # 모든 데모/예제 통합
```

## 위험 및 주의사항

1. **GitHub Actions**: .github/workflows/가 다른 파일들을 참조할 수 있음
2. **Docker Compose**: 상대 경로 수정 필요
3. **Scripts**: 다른 스크립트를 참조하는 경우 경로 수정
4. **Documentation**: 링크가 깨질 수 있음

## 예상 결과

- 루트 디렉토리 수: 23개 → 10개 이하
- 중복 제거: 약 30%
- 구조 명확성: 크게 향상
- HA 원칙 준수: 100%