use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Clone, Serialize, Deserialize)]
struct DemoState {
    counter: i32,
    messages: Vec<String>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
    counter: i32,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(DemoState {
        counter: 0,
        messages: vec!["HAL9 데모 시작".to_string()],
    }));

    let app = Router::new()
        .route("/", get(index))
        .route("/api/test", get(test_api))
        .route("/api/increment", post(increment))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("🚀 HAL9 작동 데모 서버 시작");
    println!("   http://localhost:3333 에서 접속하세요");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 실제 작동 데모</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            margin-bottom: 30px;
        }
        .button-group {
            display: flex;
            gap: 10px;
            margin: 20px 0;
        }
        button {
            background: #007bff;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 5px;
            font-size: 16px;
            cursor: pointer;
            transition: background 0.3s;
        }
        button:hover {
            background: #0056b3;
        }
        #result {
            margin-top: 20px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 5px;
            border: 1px solid #dee2e6;
        }
        .success {
            color: #28a745;
            font-weight: bold;
        }
        .counter {
            font-size: 48px;
            font-weight: bold;
            color: #007bff;
            margin: 20px 0;
        }
        #log {
            margin-top: 20px;
            padding: 10px;
            background: #f1f3f4;
            border-radius: 5px;
            font-family: monospace;
            font-size: 14px;
            max-height: 200px;
            overflow-y: auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 HAL9 실제 작동 데모</h1>
        
        <p>이것은 실제로 작동하는 데모입니다. 버튼을 클릭해서 테스트하세요.</p>
        
        <div class="button-group">
            <button onclick="testAPI()">API 테스트</button>
            <button onclick="incrementCounter()">카운터 증가</button>
            <button onclick="clearLog()">로그 지우기</button>
        </div>
        
        <div id="result">
            <div class="counter" id="counter">0</div>
            <div id="status">대기 중...</div>
        </div>
        
        <div id="log"></div>
    </div>
    
    <script>
        let logEntries = [];
        
        function addLog(message) {
            const time = new Date().toLocaleTimeString();
            logEntries.push(`[${time}] ${message}`);
            updateLog();
        }
        
        function updateLog() {
            const logDiv = document.getElementById('log');
            logDiv.innerHTML = logEntries.slice(-10).join('<br>');
            logDiv.scrollTop = logDiv.scrollHeight;
        }
        
        function clearLog() {
            logEntries = [];
            updateLog();
            addLog('로그 지움');
        }
        
        async function testAPI() {
            addLog('API 테스트 시작...');
            const statusDiv = document.getElementById('status');
            statusDiv.textContent = '요청 중...';
            
            try {
                const response = await fetch('/api/test');
                const data = await response.json();
                
                statusDiv.innerHTML = `
                    <span class="success">✓ API 작동 확인</span><br>
                    메시지: ${data.message}<br>
                    현재 카운터: ${data.counter}
                `;
                
                document.getElementById('counter').textContent = data.counter;
                addLog('API 테스트 성공');
            } catch (error) {
                statusDiv.textContent = '오류: ' + error.message;
                addLog('API 테스트 실패: ' + error.message);
            }
        }
        
        async function incrementCounter() {
            addLog('카운터 증가 요청...');
            
            try {
                const response = await fetch('/api/increment', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({})
                });
                const data = await response.json();
                
                document.getElementById('counter').textContent = data.counter;
                document.getElementById('status').innerHTML = `
                    <span class="success">✓ 카운터 업데이트</span><br>
                    ${data.message}
                `;
                
                addLog(`카운터 증가: ${data.counter}`);
            } catch (error) {
                addLog('카운터 증가 실패: ' + error.message);
            }
        }
        
        // 페이지 로드 시 초기화
        window.onload = () => {
            addLog('페이지 로드 완료');
            testAPI();
        };
    </script>
</body>
</html>
    "#)
}

async fn test_api(State(state): State<Arc<Mutex<DemoState>>>) -> Json<ApiResponse> {
    let state = state.lock().await;
    
    Json(ApiResponse {
        success: true,
        message: "HAL9 API가 정상 작동 중입니다!".to_string(),
        counter: state.counter,
    })
}

async fn increment(State(state): State<Arc<Mutex<DemoState>>>) -> Json<ApiResponse> {
    let mut state = state.lock().await;
    state.counter += 1;
    let new_counter = state.counter;
    state.messages.push(format!("카운터 증가: {}", new_counter));
    
    Json(ApiResponse {
        success: true,
        message: format!("카운터가 {}로 증가했습니다", new_counter),
        counter: new_counter,
    })
}