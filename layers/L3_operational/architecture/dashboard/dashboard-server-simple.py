#!/usr/bin/env python3
"""
HAL9 Integrated Dashboard Server (Simple Version)
외부 의존성 없는 간단한 버전
"""

import json
import os
import psutil
import time
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import threading
import random

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
        
    def update_metrics(self):
        """시스템 메트릭 업데이트"""
        # 실제 시스템 정보
        self.metrics["system"]["cpu"] = psutil.cpu_percent(interval=0.1)
        self.metrics["system"]["memory"] = psutil.virtual_memory().percent
        self.metrics["system"]["uptime"] = int(time.time() - self.start_time)
        self.metrics["system"]["latency"] = random.randint(10, 50)
        
        # 시뮬레이션 데이터
        self.metrics["neurons"]["active"] = random.randint(1000, 10000)
        self.metrics["neurons"]["layers"] = random.randint(2, 8)
        self.metrics["neurons"]["speed"] = round(random.uniform(1, 100), 2)
        self.metrics["neurons"]["grid"] = [random.random() > 0.7 for _ in range(100)]
        
        self.metrics["consciousness"]["phi"] = round(random.uniform(0, 2), 3)
        self.metrics["consciousness"]["ratio"] = f"{round(random.uniform(1, 10), 1)}:1"
        
        self.metrics["games"]["active"] = random.randint(0, 5)
        self.metrics["games"]["winRate"] = random.randint(60, 95)
        
        self.metrics["performance"]["ops"] = random.randint(10000000, 100000000)
        self.metrics["performance"]["avgResponse"] = round(random.uniform(10, 100), 1)
        self.metrics["performance"]["fps"] = random.randint(30, 120)
    
    def run_metrics_updater(self):
        """메트릭 업데이트 루프"""
        while True:
            self.update_metrics()
            time.sleep(1)
    
    def start(self):
        """대시보드 서버 시작"""
        print(f"🌌 HAL9 Integrated Dashboard Server (Simple)")
        print(f"=" * 50)
        
        # 메트릭 업데이터 시작
        metrics_thread = threading.Thread(target=self.run_metrics_updater)
        metrics_thread.daemon = True
        metrics_thread.start()
        print(f"✓ Metrics updater started")
        
        # HTTP 서버 핸들러
        dashboard_server = self
        
        class DashboardHandler(SimpleHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/" or self.path == "/dashboard":
                    self.serve_dashboard()
                elif self.path == "/api/dashboard/metrics":
                    self.serve_metrics()
                elif self.path == "/api/metrics":  # 대체 경로
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
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(json.dumps(dashboard_server.metrics).encode())
            
            def log_message(self, format, *args):
                # 기본 로그만 표시
                if "/api/" not in args[0]:
                    print(f"[{time.strftime('%H:%M:%S')}] {args[0]}")
        
        # HTTP 서버 시작
        httpd = HTTPServer(("", self.port), DashboardHandler)
        
        print(f"✓ HTTP server started on port {self.port}")
        print(f"\n🚀 Dashboard available at: http://localhost:{self.port}")
        print(f"📊 Metrics API: http://localhost:{self.port}/api/dashboard/metrics")
        print(f"\nPress Ctrl+C to stop the server")
        print(f"=" * 50)
        
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\nShutting down dashboard server...")
            httpd.shutdown()

def main():
    """메인 함수"""
    port = int(os.environ.get("DASHBOARD_PORT", 8080))
    server = DashboardServer(port)
    server.start()

if __name__ == "__main__":
    main()