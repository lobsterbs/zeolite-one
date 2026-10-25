# zeolite-one

A browser-shaped web tunnel in a single HTML file. Everything the page needs is packaged inside the file itself. There are zero external requests at load: no CDN scripts, no remote CSS, no web font downloads, no icon font.

## What is embedded

- Engine: libcurl.js 0.7.5 with its wasm binary, gzipped and base64 embedded, decompressed at boot with DecompressionStream
- Parser: acorn 8.14.0 and acorn-walk 8.3.4
- UI: the full md3e-web (Material 3 Expressive style) component bundle, gzipped and embedded
- Design tokens and type scale
- Fonts: Sora 400/500/600, Roboto 400/500, JetBrains Mono 400/500, embedded as data URLs
- Icons: inline SVG, no icon font

## Use

Open zeolite.html from disk or host it anywhere. It works from a file:// URL too. Type a URL or a search term; pages load through a wisp relay.

Requires a browser with DecompressionStream: Chrome or Edge 80+, Firefox 113+, Safari 16.4+. First boot decompresses the embedded engine and UI bundle, so the progress indicator stays visible briefly.

Browsing depends on public wisp relays being reachable. If the built-in relays are down or blocked on your network, open settings and point the app at a custom wss:// wisp endpoint.

## Notes

- The file is about 2.1 MB and must download fully before the UI unlocks. That is the honest cost of zero external requests.
- Embedded libraries keep their upstream licenses: MIT for libcurl.js, acorn, acorn-walk; OFL for the fonts; the UI bundle keeps its upstream Material Web license terms.
