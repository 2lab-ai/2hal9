#!/usr/bin/env python3
"""
Simple HTTP server for self-organization monitoring dashboard
"""

import http.server
import socketserver
import os

PORT = 8766

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
    print(f"🤖 HAL9 Self-Organization Monitoring Dashboard")
    print(f"📍 Server running at http://localhost:{PORT}")
    print(f"📊 Real-time monitoring of neuron self-organization")
    print(f"   Press Ctrl+C to stop")
    httpd.serve_forever()