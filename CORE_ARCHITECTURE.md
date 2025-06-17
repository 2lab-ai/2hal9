# 🏗️ HAL9 핵심 아키텍처 (실제 구현된 것)

## 📁 프로젝트 구조

### 실제로 코드가 있는 곳
```
2hal9/
├── substrate/tooling/rust/
│   ├── legacy-crates/       # 대부분의 실제 코드
│   │   ├── hal9-core/       # 기본 뉴런 구조 ✅
│   │   ├── hal9-server/     # 서버 구현 (MockClaude 포함) ✅
│   │   └── hal9-client/     # 클라이언트 SDK
│   ├── crates/              # 새로운 구조 (미완성)
│   └── workspace.toml       # Rust 워크스페이스 설정
├── examples/                # 작동하는 데모들 ✅
├── scripts/                 # 유틸리티 스크립트
└── reports/                 # 프로젝트 상태 보고서
```

## 🧠 뉴런 시스템 (실제 구현)

### 기본 뉴런 구조
```rust
// hal9-core/src/neuron.rs
pub struct Neuron {
    pub id: String,
    pub layer: String,
    pub activation: f32,
    pub connections: Vec<String>,
}
```

### 계층 시스템 (단순화된 버전)
- **L1**: 반사 계층 (즉각 응답)
- **L2**: 실행 계층 (작업 수행)
- **L3**: 계획 계층 (전략 수립)

*주의: 9계층은 대부분 미구현 상태*

## 🔧 MockClaude 시스템

### 작동 방식
```rust
// 설정에 따라 자동 전환
match config.claude.mode {
    "api" => ClaudeMode::Api(client),
    "mock" => ClaudeMode::Mock(mock_claude),
}
```

### Mock 응답 설정
```toml
[claude.mock_responses.L1]
[[claude.mock_responses.L1]]
trigger = "default"
response = "L1 로컬 응답: 즉각적인 반응을 처리합니다."
```

## 💾 데이터베이스 (로컬 모드)

### SQLite 사용
```bash
DATABASE_URL=sqlite://hal9_local.db
```

### 주요 테이블
- `neurons`: 뉴런 정보
- `memories`: 처리 히스토리
- `connections`: 뉴런 연결 정보

## 🌐 서버 아키텍처

### 실제 구현된 엔드포인트
```rust
// POST /neurons/{id}/signal
// GET /neurons/{id}
// GET /neurons
```

### WebSocket (부분 구현)
- 기본 연결은 가능
- 실시간 통신은 미완성

## 🧪 테스트 가능한 기능

### 1. 뉴런 생성 및 관리
```rust
let neuron = Neuron::new("test-1", "L1");
assert_eq!(neuron.layer, "L1");
```

### 2. 로컬 신호 처리
```rust
let response = neuron.process("입력");
// MockClaude가 응답 생성
```

### 3. 메모리 저장
```rust
neuron.memory.push(response);
```

## ⚙️ 환경 설정

### 필수 환경 변수
```bash
# 로컬 모드
CLAUDE_MODE=mock
DATABASE_URL=sqlite://hal9_local.db
REDIS_ENABLED=false

# API 모드 (선택)
CLAUDE_API_KEY=sk-ant-...
ANTHROPIC_MODEL=claude-3-sonnet-20240229
```

## 📊 실제 vs 계획

| 기능 | 계획 | 실제 | 상태 |
|------|------|------|------|
| 9계층 시스템 | ✅ | 3계층만 | 30% |
| 자가 조직화 | ✅ | ❌ | 0% |
| 분산 처리 | ✅ | ❌ | 0% |
| MockClaude | ✅ | ✅ | 100% |
| 로컬 실행 | ✅ | ✅ | 100% |
| WebSocket | ✅ | 부분 | 40% |
| 테스트 | ✅ | 최소한 | 0.5% |

## 🚀 실행 가능한 명령어

```bash
# 테스트
cargo test -p hal9-core

# 로컬 데모
cargo run --example simple_local_demo

# 로컬 서버
./scripts/use_local_only.sh && ./run_local.sh

# 빌드
cargo build --workspace
```

## 📝 핵심 파일 위치

- **뉴런 정의**: `substrate/tooling/rust/legacy-crates/hal9-core/src/neuron.rs`
- **MockClaude**: `substrate/tooling/rust/legacy-crates/hal9-server/src/claude.rs`
- **서버 메인**: `substrate/tooling/rust/legacy-crates/hal9-server/src/main.rs`
- **로컬 데모**: `examples/simple_local_demo.rs`

## ⚠️ 주의사항

1. **대부분의 고급 기능은 미구현**
2. **프로덕션 사용 불가**
3. **테스트 커버리지 매우 낮음**
4. **문서와 실제 구현 차이 큼**

이 문서는 실제로 존재하고 작동하는 코드 기준으로 작성되었습니다.