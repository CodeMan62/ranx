#!/usr/bin/env python3
from http.server import BaseHTTPRequestHandler, HTTPServer
import sys

class MockWebHandler(BaseHTTPRequestHandler):
    def _set_headers(self, content_type='text/html'):
        self.send_response(200)
        self.send_header('Content-type', content_type)
        self.end_headers()

    def do_GET(self):
        self._set_headers()
        
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Mock Web Server</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }}
                h1 {{ color: #333; }}
                .info {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
            </style>
        </head>
        <body>
            <h1>Mock Web Server</h1>
            <div class="info">
                <p>Path requested: <strong>{self.path}</strong></p>
                <p>Headers received:</p>
                <ul>
                    {"".join(f"<li><strong>{key}:</strong> {value}</li>" for key, value in self.headers.items())}
                </ul>
            </div>
        </body>
        </html>
        """
        
        self.wfile.write(html.encode('utf-8'))

def run(server_class=HTTPServer, handler_class=MockWebHandler, port=8000):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    print(f'Starting mock web server on port {port}...')
    httpd.serve_forever()

if __name__ == '__main__':
    # Get port from command line arguments if provided
    port = 8000
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    run(port=port) 
