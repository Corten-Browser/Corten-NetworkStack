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

    def add_cors_headers(self, origin='*', methods='GET, POST, PUT, DELETE, OPTIONS', headers='*', credentials=False):
        """Add CORS headers to response"""
        self.send_header('Access-Control-Allow-Origin', origin if not credentials else self.headers.get('Origin', origin))
        self.send_header('Access-Control-Allow-Methods', methods)
        self.send_header('Access-Control-Allow-Headers', headers)
        if credentials:
            self.send_header('Access-Control-Allow-Credentials', 'true')
        self.send_header('Access-Control-Max-Age', '86400')

    def add_csp_header(self, policy):
        """Add Content-Security-Policy header"""
        self.send_header('Content-Security-Policy', policy)

    def send_json_response(self, status_code, data, cors=False, csp=None):
        """Send JSON response"""
        response = json.dumps(data).encode('utf-8')
        self.send_response(status_code)
        if cors:
            self.add_cors_headers()
        if csp:
            self.add_csp_header(csp)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def send_text_response(self, status_code, text, content_type='text/plain', cors=False, csp=None):
        """Send text response"""
        response = text.encode('utf-8')
        self.send_response(status_code)
        if cors:
            self.add_cors_headers()
        if csp:
            self.add_csp_header(csp)
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

        # === CORS Endpoints ===
        # /cors/simple - Simple CORS request
        elif path == '/cors/simple':
            self.send_json_response(200, {
                'cors': 'simple',
                'origin': self.headers.get('Origin', 'none')
            }, cors=True)

        # /cors/credentials - CORS with credentials
        elif path == '/cors/credentials':
            origin = self.headers.get('Origin', '*')
            self.send_response(200)
            self.add_cors_headers(origin=origin, credentials=True)
            self.send_header('Content-Type', 'application/json')
            response = json.dumps({'cors': 'credentials', 'origin': origin}).encode('utf-8')
            self.send_header('Content-Length', str(len(response)))
            self.end_headers()
            self.wfile.write(response)

        # /cors/no-headers - No CORS headers (should fail)
        elif path == '/cors/no-headers':
            self.send_json_response(200, {'cors': 'blocked'})

        # /cors/custom-method - Custom method test
        elif path == '/cors/custom-method':
            self.send_json_response(200, {'cors': 'custom-method'}, cors=True)

        # === CSP Endpoints ===
        # /csp/default-src - Test default-src directive
        elif path == '/csp/default-src':
            self.send_json_response(200, {'csp': 'default-src'},
                                    csp="default-src 'self'")

        # /csp/script-src - Test script-src directive
        elif path == '/csp/script-src':
            self.send_json_response(200, {'csp': 'script-src'},
                                    csp="script-src 'self'")

        # /csp/nonce - Test nonce-based CSP
        elif path.startswith('/csp/nonce/'):
            nonce = path.split('/')[-1]
            self.send_json_response(200, {'csp': 'nonce', 'nonce': nonce},
                                    csp=f"script-src 'nonce-{nonce}'")

        # /csp/hash - Test hash-based CSP
        elif path == '/csp/hash':
            # SHA-256 hash of "alert('test')"
            hash_value = "sha256-qznLcsROx4GACP2dm0UCKCzCG+HiZ1guq6ZZDob/Tng="
            self.send_json_response(200, {'csp': 'hash'},
                                    csp=f"script-src '{hash_value}'")

        # /csp/multiple - Multiple CSP directives
        elif path == '/csp/multiple':
            self.send_json_response(200, {'csp': 'multiple'},
                                    csp="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'")

        # /csp/report-uri - CSP with report-uri
        elif path == '/csp/report-uri':
            self.send_json_response(200, {'csp': 'report-uri'},
                                    csp="default-src 'self'; report-uri /csp/report")

        # /csp/report - CSP violation report endpoint
        elif path == '/csp/report':
            self.send_json_response(200, {'received': 'report'})

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

    def do_OPTIONS(self):
        """Handle OPTIONS requests (CORS preflight)"""
        path = urlparse(self.path).path

        # CORS preflight for all /cors/* endpoints
        if path.startswith('/cors/'):
            origin = self.headers.get('Origin', '*')
            requested_method = self.headers.get('Access-Control-Request-Method', 'GET')
            requested_headers = self.headers.get('Access-Control-Request-Headers', '*')

            self.send_response(204)

            # Handle credentials mode
            if 'credentials' in path:
                self.add_cors_headers(
                    origin=origin,
                    methods=requested_method,
                    headers=requested_headers,
                    credentials=True
                )
            else:
                self.add_cors_headers(
                    origin='*',
                    methods='GET, POST, PUT, DELETE, OPTIONS',
                    headers=requested_headers
                )

            self.end_headers()
        else:
            # Standard OPTIONS response
            self.send_response(200)
            self.send_header('Allow', 'GET, POST, PUT, DELETE, PATCH, OPTIONS')
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
