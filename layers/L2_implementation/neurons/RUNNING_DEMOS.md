# Running True Self-Organization Demos

## 🚀 Quick Start

가장 간단한 실행 방법:

```bash
cd /Users/icedac/2lab.ai/2hal9/L2_implementation/neurons

# 이미 컴파일된 데모 실행
./examples/simple_demo
./examples/demo
```

## 📚 사용 가능한 데모들

### 1. **Simple True Self-Organization Demo**
**파일**: `examples/simple_true_self_org_demo.rs`
**설명**: 25개의 동일한 뉴런이 자기조직화하는 기본 데모

```bash
# 방법 1: 컴파일된 실행 파일
./examples/simple_demo

# 방법 2: Rust로 직접 컴파일
rustc --edition 2021 examples/simple_true_self_org_demo.rs -o simple_demo
./simple_demo

# 방법 3: Cargo 사용 (권장)
cargo run --example simple_true_self_org_demo
```

**결과 예시**:
```
📍 Phase 1: Creating 25 identical neurons
📡 Phase 2: Discovery - neurons finding each other
🔬 Phase 3: Natural clustering
✨ Phase 4: Hierarchy emerges!
  Layer 1: Reflexive (Fast & Simple) - 6 neurons
  Layer 2: Implementation (Fast & Medium) - 5 neurons
  Layer 3: Operational (Balanced) - 6 neurons
  Layer 4: Strategic (Slow & Complex) - 8 neurons
```

### 2. **Working AI Demo**
**파일**: `examples/working_ai_demo.rs`
**설명**: AI 뉴런들이 실제로 어떻게 연결되는지 보여주는 데모

```bash
# 방법 1: 직접 컴파일 후 실행
rustc --edition 2021 examples/working_ai_demo.rs -o working_demo
./working_demo

# 방법 2: 한 줄로 실행
rustc --edition 2021 examples/working_ai_demo.rs && ./working_ai_demo
```

**특징**:
- Visual-Detect, Audio-Process 등 실제 AI 기능을 가진 뉴런
- 호환성 기반 자동 연결
- 창발적 계층 형성

### 3. **Multi-Run Emergence Experiment**
**파일**: `examples/multi_run_emergence_experiment.rs`
**설명**: 동일한 조건에서 매번 다른 구조가 창발함을 증명

```bash
rustc --edition 2021 examples/multi_run_emergence_experiment.rs -o multi_run
./multi_run
```

**결과**: 
- 10번 실행하여 다양한 구조 관찰
- 2~6개의 다른 계층이 창발
- 진정한 자기조직화의 증거

### 4. **Environment Variables Experiment**
**파일**: `examples/environment_variables_experiment.rs`
**설명**: 환경 조건이 창발 구조에 미치는 영향

```bash
rustc --edition 2021 examples/environment_variables_experiment.rs -o env_demo
./env_demo
```

**테스트 환경**:
- High Pressure (빠른 처리 필요) → 2계층 평면 구조
- Noisy Environment (높은 불확실성) → 5계층 중복 구조
- Resource Constrained (제한된 자원) → 3계층 최소 구조

### 5. **A2A Self-Reorganization Demo**
**파일**: `examples/a2a_self_reorganization_demo.rs`
**설명**: Agent-to-Agent 프로토콜과 동적 재조직

```bash
# 이미 컴파일된 버전
./examples/demo

# 또는 cargo로 실행
cargo run --example a2a_self_reorganization_demo
```

## 🔧 컴파일 문제 해결

### 문제 1: "no example target named..."
```bash
# Cargo.toml에 example 추가 필요
# 직접 rustc 사용하여 해결:
rustc --edition 2021 examples/[데모파일].rs -o [실행파일명]
```

### 문제 2: Borrow checker 에러
```bash
# --release 모드로 컴파일
rustc --edition 2021 -O examples/[데모파일].rs
```

## 📊 데모별 핵심 개념

| 데모 | 핵심 개념 | 실행 시간 |
|------|----------|-----------|
| simple_demo | 기본 자기조직화 | 즉시 |
| working_ai_demo | AI 뉴런 연결 | ~2초 |
| multi_run | 비결정성 증명 | ~5초 |
| env_demo | 환경 적응성 | ~3초 |
| a2a_demo | 동적 재조직 | ~10초 |

## 💡 이해하기

### 진짜 vs 가짜 자기조직화

**가짜 (이전 방식)**:
```rust
// 미리 계층 할당
for layer in [L1, L2, L3, L4, L5] {
    create_neurons_in_layer(layer);
}
```

**진짜 (현재 방식)**:
```rust
// 계층 없이 시작
for i in 0..25 {
    neurons.push(random_neuron());
}
// 계층은 나중에 창발!
```

## 🎯 추천 실행 순서

1. **simple_demo** - 기본 개념 이해
2. **working_ai_demo** - AI 뉴런 동작 확인
3. **multi_run** - 창발의 다양성 관찰
4. **env_demo** - 환경 적응성 확인

## 📝 추가 자료

- 설계 문서: `/L3_operational/design/true_self_organization_design.md`
- 비교 분석: `/L3_operational/comparison/fake_vs_true_self_organization.md`
- 최종 브리핑: `/L3_operational/briefing/true_self_organization_final_briefing.md`

## 🆘 도움말

문제 발생시:
1. `rustc --version` 확인 (1.70+ 필요)
2. 현재 디렉토리가 `/L2_implementation/neurons`인지 확인
3. 파일 권한 확인: `chmod +x ./examples/*`

---

**핵심**: 이 데모들은 HAL9가 진정으로 스스로를 조직화할 수 있음을 증명합니다!