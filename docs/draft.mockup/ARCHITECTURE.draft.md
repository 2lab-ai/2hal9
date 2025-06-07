# 2HAL9 분산 뉴런 네트워크 아키텍처

## 시스템 다이어그램

```
┌─────────────────────────────────────────────────────────────────────┐
│                          2HAL9 Server 0                             │
│                     (192.168.1.100:8080)                           │
│                                                                     │
│  ┌─────────────┐                                                  │
│  │  Neuron 1   │  Strategic Layer (L4)                            │
│  │             │  STDIN/STDOUT                                    │
│  └──────┬──────┘                                                  │
│         │                                                          │
│    ┌────┴────┐                                                    │
│    ▼         ▼                                                    │
│  ┌─────────────┐                           TCP                   │
│  │  Neuron 2   │ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐   │
│  │             │  Design Layer (L3)                              │
│  └──────┬──────┘  STDIN/STDOUT                               │   │
│         │                                                         │
│    ┌────┴────┬────────┬────────┐                             │   │
│    ▼         ▼        ▼        ▼                                 │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                         │   │
│  │  4  │  │  5  │  │  6  │  │  7  │  Implementation (L2)    │   │
│  └─────┘  └─────┘  └─────┘  └─────┘  STDIN/STDOUT          │   │
│                                                               │   │
└───────────────────────────────────────────────────────────────┘   │
                                                                     │
                             TCP Socket                              │
                             :8081 ←→ :9080                          │
                                                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          2HAL9 Server 1                             │
│                     (192.168.1.101:9080)                           │
│                                                                     │
│  ┌─────────────┐                                                  │
│  │  Neuron 3   │  Design Layer (L3)                               │
│  │             │  STDIN/STDOUT                                    │
│  └──────┬──────┘                                                  │
│         │                                                          │
│    ┌────┴────┬────────┬────────┐                                 │
│    ▼         ▼        ▼        ▼                                 │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                            │
│  │  4  │  │  5  │  │  6  │  │  7  │  Implementation (L2)        │
│  └─────┘  └─────┘  └─────┘  └─────┘  STDIN/STDOUT              │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘

주의: 뉴런 4,5,6,7이 중복 표시되었지만, 실제로는 사용자가 설정에 따라 
      각 서버에 할당됨 (위 예시는 하나의 가능한 구성)
```

## 유연한 토폴로지 설정 예시

### 설정 1: 균등 분배
```yaml
Server 0: [1, 2, 4, 5]
Server 1: [3, 6, 7]
```

### 설정 2: 레이어별 분리  
```yaml
Server 0: [1, 2, 3]     # L4, L3 레이어
Server 1: [4, 5, 6, 7]  # L2 레이어
```

### 설정 3: 부하 분산
```yaml
Server 0: [1, 2]        # 상위 레이어 (적은 연산)
Server 1: [3, 4, 5, 6, 7] # 하위 레이어 (많은 연산)
```

## 통신 프로토콜

### 1. 뉴런 간 메시지 포맷 (JSON)
```json
{
    "signal_id": "uuid-v4",
    "from_neuron": "neuron-1",
    "to_neuron": "neuron-3",
    "layer_from": "L4",
    "layer_to": "L3",
    "propagation_type": "forward|backward",
    "batch_id": "batch-uuid",
    "timestamp": "2025-06-06T14:30:00Z",
    "payload": {
        "activation": {
            "content": "Process this strategic directive",
            "strength": 0.85,
            "features": {
                "urgency": 0.9,
                "complexity": 0.6
            }
        },
        "gradient": null  // or gradient object for backprop
    }
}
```

### 2. 2HAL9 서버 간 통신 프로토콜
```
┌────────────────┐         ┌────────────────┐
│  2HAL9-0       │         │  2HAL9-1       │
│                │         │                │
│  [Neuron 1]    │         │  [Neuron 3]    │
│      ↓         │   TCP   │      ↓         │
│  [Neuron 2] ───┼────────►│  [Neuron 5]    │
│  [Neuron 4]    │         │  [Neuron 6]    │
└────────────────┘         └────────────────┘

Forward: 1 → 2,4 → 3 → 5,6
Backward: 5,6 → 3 → 2,4 → 1
```

## 구현 세부사항

### 뉴런 타입별 프로세스 관리

1. **Strategic Neuron (L4)**
   - 입력: 사용자 요청 또는 상위 피드백
   - 처리: 전략적 분해
   - 출력: 2개의 Design 뉴런으로 분배

2. **Design Neuron (L3)**
   - 입력: Strategic 지시사항
   - 처리: 시스템 설계
   - 출력: Implementation 뉴런으로 전달

3. **Implementation Neuron (L2)**
   - 입력: Design 명세
   - 처리: 실제 코드 생성
   - 출력: 최종 결과 또는 에러 피드백

### TCP 연결 상세

```
2HAL9-0 (Master)              2HAL9-1 (Slave)
├─ Listen: 0.0.0.0:8081      ├─ Connect to: 192.168.1.100:8081
├─ Neuron Registry:          ├─ Neuron Registry:
│  - neuron-1: local         │  - neuron-3: local
│  - neuron-2: local         │  - neuron-5: local
│  - neuron-4: local         │  - neuron-6: local
│  - neuron-3: remote→TCP    │  - neuron-1,2,4: remote←TCP
```

### 메시지 라우팅 로직

```rust
// 2HAL9 서버의 라우팅 결정
match (from_neuron, to_neuron) {
    (local, local) => send_via_pipe(msg),
    (local, remote) => send_via_tcp(msg),
    (remote, local) => receive_via_tcp(msg),
    _ => error!("Invalid routing")
}
```

## 확장성 고려사항

1. **수평 확장**: 더 많은 2HAL9 서버 추가 가능
2. **동적 토폴로지**: 런타임에 뉴런 추가/제거
3. **장애 복구**: 뉴런 실패 시 자동 재시작
4. **로드 밸런싱**: 같은 레이어 내 작업 분배
