# zeolite-one

Zeolite single-file edition: a browser-shaped web tunnel that fits in one HTML file.

Open zeolite.html in a modern browser. No service worker, no build step, no server of your own. Pages are fetched through wisp relays over WebSocket (wasm TLS carried by libcurl.js), rendered into sandboxed iframes, and their scripts rewritten so location, fetch, XHR, WebSocket, storage and cookies behave as if the page were on the real origin.

## How it works

- Transport: wisp protocol over WebSocket through a relay pool of five public relays ordered by live evidence with wisp.terbiumon.top first, plus a custom endpoint option in settings, carried by libcurl.js with wasm TLS. At startup every relay is probed in parallel with raw wisp v1 CONNECT handshakes over plain WebSockets (spec framing: little-endian, version CONTINUE first, then a CONNECT stream), with staggered starts and a four second cap; the first winner is then verified end to end through libcurl.
- Engine loading: every engine script is raced in parallel across five CDN mirrors (jsDelivr, its fastly, gcore and testingcf edges, and unpkg), the wasm is tried per mirror with a hard timeout each, and a boot status line shows the current stage. If the engine is still missing after ninety seconds the UI stays usable and a background watcher keeps checking, quietly retrying the wasm chain, and brings the tunnel up the moment the engine arrives.
- Rewriting: acorn parses every script when available (optional; without it scripts pass through unrewritten); references to location are rewritten to a virtualized proxy, ES module specifiers are resolved to absolute URLs, and all modules share one merged import map.
- Rendering: HTML is fetched, parsed, stripped of CSP and integrity constraints, then serialized into a blob loaded in a sandboxed iframe without allow-same-origin. Nested iframes are proxied recursively. CSS url() values, fonts and media are inlined; media above a cap is routed through the tunnel on demand.
- UI: real Material 3 Expressive web components (md3e-web) themed dark with a verdigris palette: tabs, an md-search-bar omnibox with a security glyph and a relay chip, a network log drawer, settings, downloads, and a new tab page with clock, an md-search-bar search field and favorites. Every icon is an inline SVG built from real Material Symbols Rounded weight-400 outline paths; there is no icon font and no icon network fetch anywhere in the file.

## Runtime dependencies (loaded from CDN mirrors, not committed)

- libcurl.js 0.7.4 (LGPL) for transport, plus its wasm module
- acorn 8.14.0 and acorn-walk 8.3.4 (MIT) for script rewriting, optional
- @materialwebunofficial/md3e-web 1.0.14 (Apache-2.0) for the Material 3 Expressive UI

Icon paths sourced from the marella/material-symbols SVG set (Material Symbols Rounded 400 outline).

## Known limits

- No WebRTC media; peer connections do not traverse the tunnel.
- document.write output is ignored.
- File uploads are dropped; file-typed POST bodies are not tunneled.
- dynamic import() of specifiers not seen at rewrite time fails unless covered by the merged import map.
- url() values in style attributes are not rewritten.
- Per-hop cookie capture depends on the manual redirect loop observing each hop.

## Usage

Download zeolite.html and open it. At startup the browser probes the relay pool and shows the winning relay in the chip next to the address bar. Pick or enter a relay in settings if you want a different one. If every CDN mirror is blocked on your network the status line says so and the engine keeps retrying in the background; browsing starts the moment a mirror gets through.
