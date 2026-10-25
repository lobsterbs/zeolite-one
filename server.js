const http = require('http');
const crypto = require('crypto');
const FILES = {
  acorn: 'https://cdn.jsdelivr.net/npm/acorn@8.14.0/dist/acorn.min.js',
  walk: 'https://cdn.jsdelivr.net/npm/acorn-walk@8.3.4/dist/walk.min.js',
  full: 'https://cdn.jsdelivr.net/npm/libcurl.js@0.7.5/libcurl_full.js',
  tokens: 'https://cdn.jsdelivr.net/npm/@materialwebunofficial/md3e-web@1.0.14/dist/tokens.min.css',
  md3e: 'https://cdn.jsdelivr.net/npm/@materialwebunofficial/md3e-web@1.0.14/dist/md3-expressive.esm.js',
  sora400: 'https://cdn.jsdelivr.net/npm/@fontsource/sora@5/files/sora-latin-400-normal.woff2',
  sora500: 'https://cdn.jsdelivr.net/npm/@fontsource/sora@5/files/sora-latin-500-normal.woff2',
  sora600: 'https://cdn.jsdelivr.net/npm/@fontsource/sora@5/files/sora-latin-600-normal.woff2',
  roboto400: 'https://cdn.jsdelivr.net/npm/@fontsource/roboto@5/files/roboto-latin-400-normal.woff2',
  roboto500: 'https://cdn.jsdelivr.net/npm/@fontsource/roboto@5/files/roboto-latin-500-normal.woff2',
  jbm400: 'https://cdn.jsdelivr.net/npm/@fontsource/jetbrains-mono@5/files/jetbrains-mono-latin-400-normal.woff2',
  jbm500: 'https://cdn.jsdelivr.net/npm/@fontsource/jetbrains-mono@5/files/jetbrains-mono-latin-500-normal.woff2'
};
const CHUNK = 700000;
const cache = {};
async function get(name) {
  if (cache[name]) return cache[name];
  const url = FILES[name];
  if (!url) throw new Error('unknown key ' + name);
  const res = await fetch(url);
  if (!res.ok) throw new Error('upstream ' + res.status + ' for ' + name);
  const buf = Buffer.from(await res.arrayBuffer());
  cache[name] = buf;
  return buf;
}
function hash(buf, alg) { return crypto.createHash(alg).update(buf).digest('hex'); }
const server = http.createServer(async (req, res) => {
  try {
    const p = new URL(req.url, 'http://x').pathname;
    const send = (body, type) => { res.writeHead(200, { 'Content-Type': type, 'Cache-Control': 'no-store', 'Access-Control-Allow-Origin': '*' }); res.end(body); };
    if (p === '/') return send('endpoints: /meta /raw/<key> /t/<key>/<n> /b64/<key>/<n> keys: ' + Object.keys(FILES).join(','), 'text/plain; charset=utf-8');
    if (p === '/meta') {
      const meta = {};
      for (const k of Object.keys(FILES)) {
        try { const b = await get(k); meta[k] = { bytes: b.length, sha256: hash(b, 'sha256'), md5: hash(b, 'md5'), textChunks: Math.ceil(b.length / CHUNK), b64Len: Math.ceil(b.length / 3) * 4 }; }
        catch (e) { meta[k] = { error: String(e && e.message || e) }; }
      }
      return send(JSON.stringify(meta, null, 2), 'application/json');
    }
    let m;
    if ((m = p.match(/^\/raw\/([a-z0-9]+)$/))) {
      const b = await get(m[1]);
      return send(b.toString('utf8'), 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/t\/([a-z0-9]+)\/(\d+)$/))) {
      const b = await get(m[1]);
      const i = Number(m[2]);
      return send(b.toString('utf8').slice(i * CHUNK, (i + 1) * CHUNK), 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/b64\/([a-z0-9]+)\/(\d+)$/))) {
      const b = await get(m[1]);
      const i = Number(m[2]);
      return send(b.toString('base64').slice(i * CHUNK, (i + 1) * CHUNK), 'text/plain; charset=utf-8');
    }
    res.writeHead(404); res.end('nope');
  } catch (e) { res.writeHead(500); res.end(String(e && e.message || e)); }
});
server.listen(process.env.PORT || 3000, function () { console.log('zpkg up'); });
