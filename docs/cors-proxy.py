#!/usr/bin/env python3

"""A simple HTTP server that forwards all requests to another server, adding CORS headers.

Don't use this in production as it is not optimized for performance!
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
from socketserver import ThreadingMixIn
from urllib import request
from urllib.error import HTTPError


class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    """Handle requests in a separate thread."""
    daemon_threads = True  # Optional: Set True to make the threads exit when the main thread does.


class CORSProxyHTTPRequestHandler(BaseHTTPRequestHandler):
    def do_REQUEST(self, method):
        target_url = self.path[1:]  # Remove the leading '/'

        if not is_url_allowed(target_url):
            self.send_response(403)
            self.end_headers()
            self.wfile.write(b"403 Forbidden")
            return

        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length) if content_length else None
        headers = {key: val for (key, val) in self.headers.items() if key.lower() not in ('host', 'connection', 'content-length', 'content-type')}

        try:
            req = request.Request(target_url, data=body, headers=headers, method=method)
            with request.urlopen(req) as response:
                self.send_response(response.status)

                # Set up response headers, excluding the Access-Control-Allow-Origin header
                for header, value in response.headers.items():
                    if header.lower() != 'access-control-allow-origin':
                        self.send_header(header, value)

                # Replace the Access-Control-Allow-Origin header with our own
                self.send_header('Access-Control-Allow-Origin', '*')

                self.end_headers()
                self.wfile.write(response.read())
        except Exception as e:
            if isinstance(e, HTTPError):
                # Make sure to send the original status code from the upstream server
                self.send_response(e.code)
                # Copy headers from the HTTPError object
                for header, value in e.headers.items():
                    if header.lower() != 'access-control-allow-origin':
                        self.send_header(header, value)
            else:
                self.send_response(500)

            # Include Access-Control-Allow-Origin in case of an exception too
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()

            if isinstance(e, HTTPError):
                # Copy the response body from the HTTPError object
                self.wfile.write(e.read())
            else:
                # Sending a text response containing the error message
                response_text = str(e) if isinstance(e, HTTPError) else 'Internal Server Error'
                self.wfile.write(response_text.encode())


    def do_HEAD(self):
        self.do_REQUEST('HEAD')

    def do_GET(self):
        self.do_REQUEST('GET')

    def do_POST(self):
        self.do_REQUEST('POST')

    def do_PUT(self):
        self.do_REQUEST('PUT')

    def do_DELETE(self):
        self.do_REQUEST('DELETE')

    def do_OPTIONS(self):
        self.do_REQUEST('OPTIONS')


def is_url_allowed(url):
    return True


def run(server_class=ThreadedHTTPServer, handler_class=CORSProxyHTTPRequestHandler, port=8080):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    print(f"CORS proxy server listening on port {port}")
    httpd.serve_forever()


if __name__ == '__main__':
    run(port=3000)