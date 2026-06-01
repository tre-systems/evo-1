#!/usr/bin/env python3
"""evo-1 development server.

Serves the WASM application with COOP/COEP headers for SharedArrayBuffer support.
Required for wasm-bindgen-rayon parallel processing in the browser.
"""

import http.server
import errno
import sys

HOST = "127.0.0.1"
DEFAULT_PORT = 8000


class EvoOneServer(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

    def log_message(self, format, *args):
        print(f"{self.address_string()} - {format % args}")


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT

    print("evo-1 development server")
    print("=" * 40)
    print(f"Serving on http://{HOST}:{port}")
    print("COOP Header: Cross-Origin-Opener-Policy=same-origin")
    print("COEP Header: Cross-Origin-Embedder-Policy=require-corp")
    print("SharedArrayBuffer support: ENABLED")
    print("wasm-bindgen-rayon: READY")
    print("=" * 40)
    print("Press Ctrl+C to stop the server")
    print()

    try:
        with http.server.ThreadingHTTPServer((HOST, port), EvoOneServer) as httpd:
            print("Server started successfully!")
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped by user")
    except OSError as e:
        if e.errno in (errno.EADDRINUSE, 48, 98):
            print(f"Port {port} is already in use. Please stop any existing server.")
            sys.exit(1)
        raise
