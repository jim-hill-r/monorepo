// Serves the static export in dist/exported the way Cloudflare Pages does,
// so the Playwright tests load exactly the files ../cloudflare deploys.
//
// Like Pages, a path is tried as a file, then with `.html`, then as a folder
// with `index.html`, and anything else is a 404 (the site has no 404.html,
// so Pages would serve its own). Usage: node e2e/serve.mjs [port]
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, normalize, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..', 'dist', 'exported');
const port = Number(process.argv[2] ?? 8090);

const types = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.wasm': 'application/wasm',
  '.css': 'text/css',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.jpg': 'image/jpeg',
  '.webp': 'image/webp',
  '.ico': 'image/x-icon',
  '.ttf': 'font/ttf',
};

function isFile(path) {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

/** The file Pages would serve for a URL path, or undefined for a 404. */
function lookup(urlPath) {
  const path = join(root, normalize(decodeURIComponent(urlPath)));
  if (!path.startsWith(root)) return undefined;
  return [path, `${path}.html`, join(path, 'index.html')].find(isFile);
}

createServer((req, res) => {
  const file = lookup(new URL(req.url ?? '/', 'http://localhost').pathname);
  if (!file) {
    res.writeHead(404, { 'content-type': 'text/plain' }).end('Not found');
    return;
  }
  res.writeHead(200, { 'content-type': types[extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(res);
}).listen(port, () => console.log(`serving ${root} on http://localhost:${port}`));
