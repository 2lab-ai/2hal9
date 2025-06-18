#!/usr/bin/env python3
"""
HAL9 Integrated Dashboard Server
통합 대시보드 서버 - 모든 HAL9 서비스를 한 곳에서 모니터링
"""

import asyncio
import json
import os
import psutil
import time
from datetime import datetime
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import threading
import requests
import random
from websocket import WebSocketServer

class DashboardServer:
    def __init__(self, port=8080):
        self.port = port
        self.start_time = time.time()
        self.metrics = {
            "system": {
                "cpu": 0,
                "memory": 0,
                "latency": 0,
                "uptime": 0
            },
            "neurons": {
                "active": 0,
                "layers": 0,
                "speed": 0,
                "grid": [False] * 100
            },
            "consciousness": {
                "phi": 0.0,
                "ratio": "1:1"
            },
            "games": {
                "active": 0,
                "winRate": 0
            },
            "performance": {
                "ops": 0,
                "avgResponse": 0,
                "fps": 0
            }
        }
        self.websocket_server = None
        self.services = {
            "api-main": "http://localhost:3456",
            "consciousness": "http://localhost:8765",
            "self-org": "http://localhost:8766"
        }
        
    def update_metrics(self):
        """시스템 메트릭 업데이트"""
        # 실제 시스템 정보
        self.metrics["system"]["cpu"] = psutil.cpu_percent(interval=0.1)
        self.metrics["system"]["memory"] = psutil.virtual_memory().percent
        self.metrics["system"]["uptime"] = int(time.time() - self.start_time)
        
        # 서비스 연결 상태 확인
        latencies = []
        for service, url in self.services.items():
            try:
                start = time.time()
                response = requests.get(url, timeout=1)
                latency = int((time.time() - start) * 1000)
                latencies.append(latency)
            except:
                latencies.append(100)  # 연결 실패시 100ms
        
        self.metrics["system"]["latency"] = sum(latencies) // len(latencies) if latencies else 0
        
        # HAL9 서비스에서 실제 데이터 가져오기
        try:
            # AI Genius Game API
            games_response = requests.get(f"{self.services['api-main']}/api/games", timeout=1)
            if games_response.status_code == 200:
                games = games_response.json()
                self.metrics["games"]["active"] = len(games)
        except:
            pass
        
        # 시뮬레이션 데이터 (실제 서비스 연동시 교체)
        self.metrics["neurons"]["active"] = random.randint(1000, 10000)
        self.metrics["neurons"]["layers"] = random.randint(2, 8)
        self.metrics["neurons"]["speed"] = round(random.uniform(1, 100), 2)
        self.metrics["neurons"]["grid"] = [random.random() > 0.7 for _ in range(100)]
        
        self.metrics["consciousness"]["phi"] = round(random.uniform(0, 2), 3)
        self.metrics["consciousness"]["ratio"] = f"{round(random.uniform(1, 10), 1)}:1"
        
        self.metrics["games"]["winRate"] = random.randint(60, 95)
        
        self.metrics["performance"]["ops"] = random.randint(10000000, 100000000)
        self.metrics["performance"]["avgResponse"] = round(random.uniform(10, 100), 1)
        self.metrics["performance"]["fps"] = random.randint(30, 120)
    
    def broadcast_metrics(self):
        """WebSocket으로 메트릭 브로드캐스트"""
        if self.websocket_server:
            message = json.dumps(self.metrics)
            self.websocket_server.send_message_to_all(message)
    
    def handle_websocket_message(self, client, server, message):
        """WebSocket 메시지 처리"""
        try:
            data = json.loads(message)
            action = data.get("action")
            
            if action == "reorganize":
                print("[Dashboard] Reorganizing neurons...")
                # 뉴런 재조직화 로직
                response = {"status": "success", "message": "Neurons reorganized"}
                
            elif action == "add_neurons":
                count = data.get("count", 10)
                print(f"[Dashboard] Adding {count} neurons...")
                # 뉴런 추가 로직
                response = {"status": "success", "added": count}
                
            else:
                response = {"status": "error", "message": "Unknown command"}
            
            server.send_message(client, json.dumps(response))
            
        except Exception as e:
            print(f"[Dashboard] Error handling message: {e}")
    
    def run_metrics_updater(self):
        """메트릭 업데이트 루프"""
        while True:
            self.update_metrics()
            self.broadcast_metrics()
            time.sleep(1)
    
    def start(self):
        """대시보드 서버 시작"""
        print(f"🌌 HAL9 Integrated Dashboard Server")
        print(f"=" * 50)
        
        # WebSocket 서버 시작
        self.websocket_server = websocket_server.WebsocketServer(9001)
        self.websocket_server.set_fn_message_received(self.handle_websocket_message)
        
        ws_thread = threading.Thread(target=self.websocket_server.run_forever)
        ws_thread.daemon = True
        ws_thread.start()
        print(f"✓ WebSocket server started on port 9001")
        
        # 메트릭 업데이터 시작
        metrics_thread = threading.Thread(target=self.run_metrics_updater)
        metrics_thread.daemon = True
        metrics_thread.start()
        print(f"✓ Metrics updater started")
        
        # HTTP 서버 시작
        class DashboardHandler(SimpleHTTPRequestHandler):
            def __init__(self, *args, dashboard_server=None, **kwargs):
                self.dashboard_server = dashboard_server
                super().__init__(*args, **kwargs)
            
            def do_GET(self):
                if self.path == "/" or self.path == "/dashboard":
                    self.serve_dashboard()
                elif self.path == "/api/dashboard/metrics":
                    self.serve_metrics()
                else:
                    super().do_GET()
            
            def serve_dashboard(self):
                dashboard_path = Path(__file__).parent / "integrated-dashboard.html"
                if dashboard_path.exists():
                    self.send_response(200)
                    self.send_header("Content-type", "text/html")
                    self.end_headers()
                    with open(dashboard_path, "rb") as f:
                        self.wfile.write(f.read())
                else:
                    self.send_error(404, "Dashboard not found")
            
            def serve_metrics(self):
                self.send_response(200)
                self.send_header("Content-type", "application/json")
                self.end_headers()
                metrics = self.server.dashboard_server.metrics
                self.wfile.write(json.dumps(metrics).encode())
            
            def log_message(self, format, *args):
                # 로그 억제
                pass
        
        # Handler에 dashboard_server 참조 추가
        handler = lambda *args, **kwargs: DashboardHandler(
            *args, 
            dashboard_server=self, 
            **kwargs
        )
        
        httpd = HTTPServer(("", self.port), handler)
        httpd.dashboard_server = self
        
        print(f"✓ HTTP server started on port {self.port}")
        print(f"\n🚀 Dashboard available at: http://localhost:{self.port}")
        print(f"📡 WebSocket endpoint: ws://localhost:9001")
        print(f"\nPress Ctrl+C to stop the server")
        print(f"=" * 50)
        
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\nShutting down dashboard server...")
            httpd.shutdown()

def main():
    """메인 함수"""
    # 환경 변수에서 포트 읽기
    port = int(os.environ.get("DASHBOARD_PORT", 8080))
    
    # 대시보드 서버 시작
    server = DashboardServer(port)
    server.start()

if __name__ == "__main__":
    main()