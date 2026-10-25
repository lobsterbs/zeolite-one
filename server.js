const http = require('http');
const crypto = require('crypto');
const FILES = {
  acorn: 'https://cdn.jsdelivr.net/npm/acorn@8.14.0/dist/acorn.min.js',
  walk: 'https://cdn.jsdelivr.net/npm/acorn-walk@8.3.4/dist/walk.min.js',
  full: 'https://cdn.jsdelivr.net/npm/libcurl.js@0.7.5/libcurl_full.js',
  wasm: 'https://cdn.jsdelivr.net/npm/libcurl.js@0.7.5/libcurl.wasm'
};
const CHUNK = 700000;
const cache = {};
async function get(name) {
  if (cache[name]) return cache[name];
  const res = await fetch(FILES[name]);
  if (!res.ok) throw new Error('upstream ' + res.status + ' for ' + name);
  const buf = Buffer.from(await res.arrayBuffer());
  cache[name] = buf;
  return buf;
}
function sha(buf) { return crypto.createHash('sha256').update(buf).digest('hex'); }
const server = http.createServer(async (req, res) => {
  try {
    const p = new URL(req.url, 'http://x').pathname;
    const send = (body, type) => { res.writeHead(200, { 'Content-Type': type, 'Cache-Control': 'no-store', 'Access-Control-Allow-Origin': '*' }); res.end(body); };
    if (p === '/') return send('endpoints: /acorn /walk /full/<n> /wasm/<n> /meta', 'text/plain; charset=utf-8');
    if (p === '/meta') {
      const meta = {};
      for (const k of Object.keys(FILES)) {
        try { const b = await get(k); meta[k] = { bytes: b.length, sha256: sha(b), chunks: Math.ceil(b.length / CHUNK) }; }
        catch (e) { meta[k] = { error: String(e && e.message || e) }; }
      }
      return send(JSON.stringify(meta, null, 2), 'application/json');
    }
    let m;
    if ((m = p.match(/^\/(acorn|walk|full)$/))) {
      const b = await get(m[1]);
      return send(b.toString('utf8'), 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/full\/(\d+)$/))) {
      const b = await get('full');
      const i = Number(m[1]);
      return send(b.toString('utf8').slice(i * CHUNK, (i + 1) * CHUNK), 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/wasm\/(\d+)$/))) {
      const b = await get('wasm');
      const i = Number(m[1]);
      return send(b.toString('base64').slice(i * CHUNK, (i + 1) * CHUNK), 'text/plain; charset=utf-8');
    }
    res.writeHead(404); res.end('nope');
  } catch (e) { res.writeHead(500); res.end(String(e && e.message || e)); }
});
server.listen(process.env.PORT || 3000, function () { console.log('zpkg up'); });
