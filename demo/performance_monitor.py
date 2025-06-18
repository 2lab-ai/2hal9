#!/usr/bin/env python3
import http.server
import socketserver
import json
import time
import subprocess
import psutil
import os

PORT = 8767

class PerformanceHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Performance Monitor</title>
    <style>
        body { 
            font-family: monospace; 
            background: #0a0a0a; 
            color: #00ff00;
            padding: 20px;
        }
        .metric {
            margin: 10px 0;
            padding: 10px;
            border: 1px solid #00ff00;
            display: inline-block;
        }
        a {
            color: #00ffff;
            text-decoration: none;
            margin: 0 10px;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>HAL9 Performance Suite</h1>
    <div id="metrics"></div>
    <h2>Quick Links</h2>
    <a href="http://localhost:8766" target="_blank">Self-Organization Dashboard</a>
    <a href="http://localhost:8765" target="_blank">Consciousness Visualization</a>
    <a href="http://localhost:3456" target="_blank">AI Genius Game</a>
    <script>
        setInterval(() => {
            fetch('/metrics')
                .then(r => r.json())
                .then(data => {
                    document.getElementById('metrics').innerHTML = 
                        Object.entries(data).map(([k,v]) => 
                            `<div class="metric">${k}: ${v}</div>`
                        ).join('');
                });
        }, 1000);
    </script>
</body>
</html>
            '''
            self.wfile.write(html.encode())
        elif self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            metrics = {
                'cpu_percent': psutil.cpu_percent(),
                'memory_percent': psutil.virtual_memory().percent,
                'timestamp': time.strftime('%Y-%m-%d %H:%M:%S')
            }
            self.wfile.write(json.dumps(metrics).encode())
        else:
            self.send_error(404)

print(f"Performance monitor running on port {PORT}")
with socketserver.TCPServer(("", PORT), PerformanceHandler) as httpd:
    httpd.serve_forever()
