#!/usr/bin/env python3
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import sys

class MockAPIHandler(BaseHTTPRequestHandler):
    def _set_headers(self):
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

    def do_GET(self):
        self._set_headers()
        
        if self.path == '/health':
            response = {"status": "ok"}
        elif self.path == '/users':
            response = {"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}
        else:
            response = {"message": f"API endpoint hit: {self.path}"}
        
        self.wfile.write(json.dumps(response).encode('utf-8'))

    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length)
        
        self._set_headers()
        response = {"message": f"Received POST data on {self.path}", "data": post_data.decode('utf-8')}
        self.wfile.write(json.dumps(response).encode('utf-8'))

def run(server_class=HTTPServer, handler_class=MockAPIHandler, port=3000):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    print(f'Starting mock API server on port {port}...')
    httpd.serve_forever()

if __name__ == '__main__':
    # Get port from command line arguments if provided
    port = 3000
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    run(port=port) 
