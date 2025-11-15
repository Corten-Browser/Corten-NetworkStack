#!/usr/bin/env python3
"""
Minimal HTTP test server for NetworkStack testing
Provides endpoints similar to httpbin.org for local testing
"""

import json
import time
import gzip
import zlib
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import sys

class TestRequestHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        """Suppress default logging"""
        pass

    def send_json_response(self, status_code, data):
        """Send JSON response"""
        response = json.dumps(data).encode('utf-8')
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def send_text_response(self, status_code, text, content_type='text/plain'):
        """Send text response"""
        response = text.encode('utf-8')
        self.send_response(status_code)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def do_GET(self):
        """Handle GET requests"""
        path = urlparse(self.path).path
        query = parse_qs(urlparse(self.path).query)

        # /get - Basic GET endpoint
        if path == '/get':
            self.send_json_response(200, {
                'method': 'GET',
                'url': f'http://{self.headers.get("Host")}{self.path}',
                'headers': dict(self.headers)
            })

        # /status/<code> - Return specific status code
        elif path.startswith('/status/'):
            try:
                status_code = int(path.split('/')[-1])
                self.send_response(status_code)
                self.end_headers()
            except ValueError:
                self.send_response(400)
                self.end_headers()

        # /headers - Return request headers
        elif path == '/headers':
            self.send_json_response(200, {
                'headers': dict(self.headers)
            })

        # /response-headers - Return with specific response headers
        elif path == '/response-headers':
            content_type = query.get('Content-Type', ['application/json'])[0]
            self.send_response(200)
            self.send_header('Content-Type', content_type)
            self.end_headers()
            self.wfile.write(b'{}')

        # /redirect/<n> - Redirect chain
        elif path.startswith('/redirect/'):
            try:
                remaining = int(path.split('/')[-1])
                if remaining > 1:
                    self.send_response(302)
                    self.send_header('Location', f'/redirect/{remaining - 1}')
                    self.end_headers()
                else:
                    self.send_json_response(200, {'redirected': True})
            except ValueError:
                self.send_response(400)
                self.end_headers()

        # /json - Return JSON
        elif path == '/json':
            self.send_json_response(200, {
                'slideshow': {
                    'title': 'Sample Slide Show',
                    'slides': [
                        {'title': 'Wake up to WonderWidgets!', 'type': 'all'}
                    ]
                }
            })

        # /html - Return HTML
        elif path == '/html':
            self.send_text_response(200,
                '<!DOCTYPE html><html><body><h1>Test HTML</h1></body></html>',
                'text/html')

        # /gzip - Return gzip-compressed response
        elif path == '/gzip':
            data = json.dumps({'gzipped': True, 'method': 'GET'}).encode('utf-8')
            compressed = gzip.compress(data)
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Encoding', 'gzip')
            self.send_header('Content-Length', str(len(compressed)))
            self.end_headers()
            self.wfile.write(compressed)

        # /deflate - Return deflate-compressed response
        elif path == '/deflate':
            data = json.dumps({'deflated': True, 'method': 'GET'}).encode('utf-8')
            compressed = zlib.compress(data)
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Encoding', 'deflate')
            self.send_header('Content-Length', str(len(compressed)))
            self.end_headers()
            self.wfile.write(compressed)

        # /encoding/utf8 - UTF-8 content
        elif path == '/encoding/utf8':
            self.send_text_response(200,
                'Hello World! 你好世界! Привет мир! مرحبا بالعالم!',
                'text/html; charset=utf-8')

        # /cache/<seconds> - Cache control
        elif path.startswith('/cache/'):
            try:
                seconds = int(path.split('/')[-1])
                self.send_response(200)
                self.send_header('Cache-Control', f'public, max-age={seconds}')
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps({'cached': True}).encode('utf-8'))
            except ValueError:
                self.send_response(400)
                self.end_headers()

        # /delay/<seconds> - Delayed response
        elif path.startswith('/delay/'):
            try:
                delay = int(path.split('/')[-1])
                time.sleep(delay)
                self.send_json_response(200, {'delayed': delay})
            except ValueError:
                self.send_response(400)
                self.end_headers()

        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        """Handle POST requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        path = urlparse(self.path).path

        if path == '/post':
            try:
                data = json.loads(body) if body else {}
            except json.JSONDecodeError:
                data = {'raw': body.decode('utf-8', errors='replace')}

            self.send_json_response(200, {
                'method': 'POST',
                'url': f'http://{self.headers.get("Host")}{self.path}',
                'headers': dict(self.headers),
                'json': data
            })
        else:
            self.send_response(404)
            self.end_headers()

    def do_PUT(self):
        """Handle PUT requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        if urlparse(self.path).path == '/put':
            self.send_json_response(200, {
                'method': 'PUT',
                'data': body.decode('utf-8', errors='replace')
            })
        else:
            self.send_response(404)
            self.end_headers()

    def do_DELETE(self):
        """Handle DELETE requests"""
        if urlparse(self.path).path == '/delete':
            self.send_json_response(200, {
                'method': 'DELETE'
            })
        else:
            self.send_response(404)
            self.end_headers()

    def do_PATCH(self):
        """Handle PATCH requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        if urlparse(self.path).path == '/patch':
            self.send_json_response(200, {
                'method': 'PATCH',
                'data': body.decode('utf-8', errors='replace')
            })
        else:
            self.send_response(404)
            self.end_headers()

def run_server(port=8080):
    """Run the test server"""
    server_address = ('127.0.0.1', port)
    httpd = HTTPServer(server_address, TestRequestHandler)
    print(f'Starting test server on http://127.0.0.1:{port}/')
    print('Press Ctrl+C to stop')
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print('\nShutting down server...')
        httpd.shutdown()

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    run_server(port)
