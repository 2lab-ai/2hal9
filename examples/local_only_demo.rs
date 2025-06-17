//! HAL9 로컬 전용 데모 - 외부 의존성 없음

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 간단한 로컬 뉴런 시스템
#[derive(Clone)]
struct LocalNeuron {
    id: String,
    layer: String,
    memory: Arc<RwLock<Vec<String>>>,
}

impl LocalNeuron {
    fn new(id: &str, layer: &str) -> Self {
        Self {
            id: id.to_string(),
            layer: layer.to_string(),
            memory: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn process(&self, input: &str) -> String {
        // 로컬 처리 (외부 API 호출 없음)
        let response = match self.layer.as_str() {
            "L1" => format!("[반사] {}", input),
            "L2" => format!("[실행] {} 작업 수행", input),
            "L3" => format!("[계획] {} 전략 수립", input),
            _ => format!("[처리] {}", input),
        };

        // 메모리에 저장
        self.memory.write().await.push(format!("{}: {}", self.id, response.clone()));

        response
    }

    async fn get_history(&self) -> Vec<String> {
        self.memory.read().await.clone()
    }
}

/// 로컬 신경망
struct LocalNetwork {
    neurons: HashMap<String, LocalNeuron>,
}

impl LocalNetwork {
    fn new() -> Self {
        let mut neurons = HashMap::new();
        
        // 기본 뉴런 생성
        neurons.insert("n1".to_string(), LocalNeuron::new("n1", "L1"));
        neurons.insert("n2".to_string(), LocalNeuron::new("n2", "L2"));
        neurons.insert("n3".to_string(), LocalNeuron::new("n3", "L3"));

        Self { neurons }
    }

    async fn send_signal(&self, neuron_id: &str, signal: &str) -> Result<String, String> {
        match self.neurons.get(neuron_id) {
            Some(neuron) => Ok(neuron.process(signal).await),
            None => Err(format!("뉴런 {} 없음", neuron_id)),
        }
    }

    async fn broadcast(&self, signal: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        for (id, neuron) in &self.neurons {
            let response = neuron.process(signal).await;
            results.push((id.clone(), response));
        }

        results
    }
}

#[tokio::main]
async fn main() {
    println!("=== HAL9 로컬 전용 데모 ===");
    println!("외부 의존성: 없음 ✅");
    println!("인터넷 연결: 불필요 ✅");
    println!();

    // 로컬 네트워크 생성
    let network = LocalNetwork::new();

    // 개별 뉴런 테스트
    println!("1. 개별 뉴런 테스트:");
    let response = network.send_signal("n1", "안녕하세요").await.unwrap();
    println!("   L1 뉴런: {}", response);

    let response = network.send_signal("n2", "데이터 처리").await.unwrap();
    println!("   L2 뉴런: {}", response);

    let response = network.send_signal("n3", "프로젝트").await.unwrap();
    println!("   L3 뉴런: {}", response);

    println!();

    // 브로드캐스트 테스트
    println!("2. 브로드캐스트 테스트:");
    let results = network.broadcast("시스템 점검").await;
    for (id, response) in results {
        println!("   {}: {}", id, response);
    }

    println!();

    // 히스토리 확인
    println!("3. 처리 히스토리:");
    if let Some(neuron) = network.neurons.get("n1") {
        let history = neuron.get_history().await;
        for entry in history {
            println!("   {}", entry);
        }
    }

    println!();
    println!("✅ 모든 기능이 로컬에서만 실행되었습니다!");
}