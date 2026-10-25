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
    const u = new URL(req.url, 'http://x');
    const p = u.pathname;
    const send = (body, type) => { res.writeHead(200, { 'Content-Type': type, 'Cache-Control': 'no-store', 'Access-Control-Allow-Origin': '*' }); res.end(body); };
    if (p === '/') return send('endpoints: /meta /md5/<key>/<start>/<len> /b64w/<key>/<start>/<len> /ghmd5/<owner>/<repo>/<ref>/<path>', 'text/plain; charset=utf-8');
    if (p === '/meta') {
      const meta = {};
      for (const k of Object.keys(FILES)) {
        try { const b = await get(k); meta[k] = { bytes: b.length, sha256: hash(b, 'sha256'), md5: hash(b, 'md5') }; }
        catch (e) { meta[k] = { error: String(e && e.message || e) }; }
      }
      return send(JSON.stringify(meta, null, 2), 'application/json');
    }
    let m;
    if ((m = p.match(/^\/md5\/([a-z0-9]+)\/(\d+)\/(\d+)$/))) {
      const b = await get(m[1]);
      const s = Number(m[2]);
      const len = Math.min(Number(m[3]), b.length - s);
      if (s < 0 || s >= b.length || len < 0) throw new Error('bad range');
      return send(hash(b.subarray(s, s + len), 'md5'), 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/b64w\/([a-z0-9]+)\/(\d+)\/(\d+)$/))) {
      const b = await get(m[1]);
      const s = Number(m[2]);
      let len = Math.min(Number(m[3]), 700002, b.length - s);
      if (s < 0 || s >= b.length || len < 0) throw new Error('bad range');
      const b64 = b.subarray(s, s + len).toString('base64').replace(/(.{76})/g, '$1\n') + '\n';
      return send(b64, 'text/plain; charset=utf-8');
    }
    if ((m = p.match(/^\/ghmd5\/([^/]+)\/([^/]+)\/([^/]+)(?:\/(.*))?$/))) {
      const owner = decodeURIComponent(m[1]);
      const repo = decodeURIComponent(m[2]);
      const ref = decodeURIComponent(m[3]);
      const path = decodeURIComponent(m[4] || '');
      const ru = 'https://raw.githubusercontent.com/' + owner + '/' + repo + '/' + ref + '/' + path;
      const rr = await fetch(ru);
      if (!rr.ok) return send(JSON.stringify({ error: 'github ' + rr.status, url: ru }), 'application/json');
      const bb = Buffer.from(await rr.arrayBuffer());
      return send(hash(bb, 'md5') + ' ' + bb.length + ' ' + hash(bb, 'sha256'), 'text/plain; charset=utf-8');
    }
    res.writeHead(404); res.end('nope');
  } catch (e) { res.writeHead(500); res.end(String(e && e.message || e)); }
});
server.listen(process.env.PORT || 3000, function () { console.log('zpkg up'); });
