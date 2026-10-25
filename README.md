# zeolite-one

Zeolite single-file edition: a browser-shaped web tunnel that fits in one HTML file.

Open zeolite.html in a modern browser. No service worker, no build step, no server of your own. Pages are fetched through wisp relays over WebSocket (wasm TLS carried by libcurl.js), rendered into sandboxed iframes, and their scripts rewritten so location, fetch, XHR, WebSocket, storage and cookies behave as if the page were on the real origin.

## How it works

- Transport: wisp protocol over WebSocket through a relay pool (three defaults plus a custom endpoint option in settings), carried by libcurl.js with wasm TLS.
- Rewriting: acorn parses every script; references to location are rewritten to a virtualized proxy, ES module specifiers are resolved to absolute URLs, and all modules share one merged import map.
- Rendering: HTML is fetched, parsed, stripped of CSP and integrity constraints, then serialized into a blob loaded in a sandboxed iframe without allow-same-origin. Nested iframes are proxied recursively. CSS url() values, fonts and media are inlined; media above a cap is routed through the tunnel on demand.
- UI: Material 3 Expressive dark theme with tabs, an omnibox with security and relay chips, a network log drawer, settings, downloads, and a new tab page with clock and favorites.

## Runtime dependencies (loaded from jsDelivr, not committed)

- libcurl.js 0.7.4 (LGPL) for transport
- acorn 8.14.0 and acorn-walk 8.3.4 (MIT) for script rewriting

## Known limits

- No WebRTC media; peer connections do not traverse the tunnel.
- document.write output is ignored.
- File uploads are dropped; file-typed POST bodies are not tunneled.
- dynamic import() of specifiers not seen at rewrite time fails unless covered by the merged import map.
- url() values in style attributes are not rewritten.
- Per-hop cookie capture depends on the manual redirect loop observing each hop.

## Usage

Download zeolite.html and open it. Pick or enter a relay in settings if the defaults are unreachable.
