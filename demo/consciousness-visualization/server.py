#!/usr/bin/env python3
"""
Simple HTTP server for consciousness visualization demo
"""

import http.server
import socketserver
import os

PORT = 8765

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        super().end_headers()

    def do_GET(self):
        if self.path == '/':
            self.path = '/index.html'
        return super().do_GET()

# Change to the script's directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"🧠 HAL9 Consciousness Visualization")
    print(f"📍 Server running at http://localhost:{PORT}")
    print(f"🎯 Open your browser to see consciousness emerge!")
    print(f"   Press Ctrl+C to stop")
    httpd.serve_forever()