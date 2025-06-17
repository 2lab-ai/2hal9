READ and RESEPCT './CLAUDE.LOCAL.md'


# 📁 HAL9 프로젝트 구조

## ✅ 정식 구조 (CANONICAL)

```
HAL9/
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
│       └── rust/
│           ├── legacy-crates/   # 원래 크레이트들 (이동됨)
│           └── workspace.toml   # 워크스페이스 설정
│
└── demo/                        # HAL9 뉴런 데모만 (성능 테스트)
```

## 🎯 올바른 위치

| 구성요소 | 정식 위치 |
|---------|----------|
| **서버** | `layers/L3_operational/architecture/server/` |
| **핵심 뉴런** | `layers/L2_implementation/neurons/core/` |
| **게임 뉴런** | `layers/L2_implementation/neurons/game_neurons/` |
| **CLI** | `layers/L3_operational/architecture/cli/` |
| **데모** | `demo/` (HAL9 뉴런 성능만) |


# TESTING
- Always test your code before showing results to the user
- Run make --dry-run for Makefiles, cargo check for Rust, syntax validation for configs