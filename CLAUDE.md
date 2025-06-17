READ and RESEPCT './CLAUDE.LOCAL.md'


# 📁 HAL9 프로젝트 구조

## ✅ 정식 구조 (CANONICAL)

```
2hal9/
├── layers/                        # HA (Hierarchical Architecture) 구현
│   ├── L1_reflexive/             # 긴급/운영 계층
│   ├── L2_implementation/        # 핵심 구현 ⭐
│   │   ├── neurons/              # 모든 뉴런 코드
│   │   │   ├── core/            # 기본 뉴런 프레임워크
│   │   │   ├── game_neurons/    # 게임 특화 뉴런
│   │   │   └── agent_dropout/   # 에이전트 패턴
│   │   └── validation/          # 테스트 및 벤치마크
│   ├── L3_operational/          # 서버 및 운영 ⭐
│   │   └── architecture/
│   │       ├── server/          # 메인 HAL9 서버
│   │       ├── browser/         # 브라우저 자동화
│   │       └── cli/             # CLI 도구
│   └── L4~L9.../               # 상위 추상화 계층
│
├── substrate/                    # 인프라 및 도구
│   └── tooling/
└── demo/                        # HAL9 뉴런 데모만 (성능 테스트)

../gradient-core/           # Gradient Core: Foundation library providing mathematical and algorithmic primitives for distributed AI systems and collective intelligence.
../2hal9-demo/
├── crates/
│   ├── genius-core/      # Game traits & types
│   ├── genius-engine/    # Execution & emergence detection
│   ├── genius-ai/        # AI provider abstractions
│   ├── genius-games/     # All game implementations
│   ├── genius-server/    # HTTP/WebSocket server
│   └── genius-client/    # Client SDK
├── demo/                 # Interactive HTML demos
├── docker/               # Container configuration
└── k8s/                  # Kubernetes manifests
```

## 🎯 올바른 위치

| 구성요소 | 정식 위치 |
|---------|----------|
| **서버** | `layers/L3_operational/architecture/server/` |
| **핵심 뉴런** | `layers/L2_implementation/neurons/core/` |
| **게임 뉴런** | `layers/L2_implementation/neurons/game_neurons/` |
| **CLI** | `layers/L3_operational/architecture/cli/` |
| **데모** | `demo/` (HAL9 뉴런 성능만) |
| **쇼케이스 데모** | `../2hal9-demo/` |


# TESTING
- Always test your code before showing results to the user
- Run make --dry-run for Makefiles, cargo check for Rust, syntax validation for configs

---

# 🧹 ADHD 중복 정리 계획 (2025-06-17)

## 루트에 있으면 안 되는 것들 (ADHD가 만든 중복)

다음 디렉토리들은 layers/L3_operational/에 이미 있거나 있어야 함:

### 즉시 삭제/이동 대상:
- `k8s/` → `layers/L3_operational/architecture/kubernetes/` (이미 있음)
- `monitoring/` → `layers/L3_operational/workflows/monitoring/` (이미 있음) 
- `nginx/` → `layers/L3_operational/configuration/nginx/`
- `ssl/` → `layers/L3_operational/configuration/ssl/`
- `scripts/` → 용도별로 분산:
  - 긴급: `layers/L1_reflexive/*/scripts/`
  - 운영: `layers/L3_operational/scripts/`
- `docker-compose*.yml` → `layers/L3_operational/configuration/docker/`

### 정리 필요:
- `docs/` → 계층별로 분산 또는 `layers/L3_operational/documentation/`
- `examples/` → `demo/`와 통합
- `tests/` → 각 계층별 tests/로 분산

### 루트에 있어야 하는 것 (예외):
- `.github/` - GitHub 요구사항
- `.cargo/`, `Cargo.toml` - Rust 설정
- `README.md`, `CLAUDE.md` - 프로젝트 문서
- 기타 .으로 시작하는 설정 파일들

## 실행 명령어

```bash
# Phase 1: K8s 정리 (중복 제거)
rm -rf k8s/  # layers/L3_operational/architecture/kubernetes/ 사용

# Phase 2: Docker 설정 이동
mv docker-compose*.yml layers/L3_operational/configuration/docker/

# Phase 3: 인프라 이동
mv nginx/ layers/L3_operational/configuration/
mv ssl/ layers/L3_operational/configuration/
# monitoring은 내용 확인 후 병합 또는 삭제

# Phase 4: Scripts 정리
# 각 스크립트를 용도에 맞게 이동

# Phase 5: 문서 정리
# docs/를 적절히 분산
```

상세 계획은 `ADHD_CLEANUP_PLAN.md` 참조