#!/usr/bin/env python3
"""
BattleO Development Server
==========================
Serves the WASM application with proper CORS headers for SharedArrayBuffer support.
Required for wasm-bindgen-rayon parallel processing in the browser.
"""

import http.server
import socketserver
import os
import sys


class BattleOServer(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Required headers for SharedArrayBuffer support
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

    def log_message(self, format, *args):
        # Custom logging to show the headers are being sent
        try:
            if isinstance(args[0], str) and args[0].startswith("GET"):
                print(f"🌐 {args[0]} - Headers: COOP=same-origin, COEP=require-corp")
            else:
                print(f"📡 {args[0]}")
        except (AttributeError, IndexError):
            # Handle cases where args[0] is not a string or doesn't exist
            print(f"📡 {format % args if args else format}")


if __name__ == "__main__":
    PORT = 8000

    print("🚀 BattleO Development Server")
    print("=" * 40)
    print(f"📍 Serving on http://localhost:{PORT}")
    print("🔒 CORS Headers: Cross-Origin-Opener-Policy=same-origin")
    print("🔒 CORS Headers: Cross-Origin-Embedder-Policy=require-corp")
    print("⚡ SharedArrayBuffer support: ENABLED")
    print("🔄 wasm-bindgen-rayon: READY")
    print("=" * 40)
    print("💡 Press Ctrl+C to stop the server")
    print()

    try:
        with socketserver.TCPServer(("", PORT), BattleOServer) as httpd:
            print(f"✅ Server started successfully!")
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
    except OSError as e:
        if e.errno == 48:  # Address already in use
            print(f"❌ Port {PORT} is already in use. Please stop any existing server.")
            sys.exit(1)
        else:
            raise
