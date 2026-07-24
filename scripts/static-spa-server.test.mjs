import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, test } from "node:test";

import { createStaticSpaServer } from "./static-spa-server.mjs";

let directory;
let origin;
let server;

before(async () => {
  directory = await mkdtemp(join(tmpdir(), "personal-site-server-"));
  await writeFile(join(directory, "index.html"), "<!doctype html><title>Test SPA</title>");
  await writeFile(join(directory, "thumbnail.png"), Buffer.from("\x89PNG\r\n\x1a\n", "binary"));
  await writeFile(join(directory, "robots.txt"), "User-agent: *\n");
  await writeFile(
    join(directory, "_headers"),
    "/*\n  Content-Security-Policy: default-src 'none'\n  Referrer-Policy: strict-origin-when-cross-origin\n  X-Content-Type-Options: nosniff\n  X-Frame-Options: DENY\n\n/*.js\n  Cache-Control: immutable\n",
  );

  server = createStaticSpaServer(directory);
  await new Promise((resolveServer) => server.listen(0, "127.0.0.1", resolveServer));
  const address = server.address();
  assert(address && typeof address !== "string");
  origin = `http://127.0.0.1:${address.port}`;
});

after(async () => {
  await new Promise((resolveServer, rejectServer) =>
    server.close((error) => (error ? rejectServer(error) : resolveServer())),
  );
  await rm(directory, { recursive: true, force: true });
});

test("serves known assets with accurate MIME and security headers", async () => {
  const response = await fetch(`${origin}/thumbnail.png`);

  assert.equal(response.status, 200);
  assert.equal(response.headers.get("content-type"), "image/png");
  assert.equal(response.headers.get("x-content-type-options"), "nosniff");
  assert.equal(response.headers.get("x-frame-options"), "DENY");
});

test("falls back to the SPA only for extensionless routes", async () => {
  const route = await fetch(`${origin}/projects`);
  assert.equal(route.status, 200);
  assert.match(await route.text(), /Test SPA/);

  const asset = await fetch(`${origin}/missing.js`);
  assert.equal(asset.status, 404);
  assert.equal(await asset.text(), "Static asset not found.");
});

test("supports HEAD and rejects unsupported methods", async () => {
  const head = await fetch(`${origin}/robots.txt`, { method: "HEAD" });
  assert.equal(head.status, 200);
  assert.equal(head.headers.get("content-type"), "text/plain; charset=utf-8");
  assert.equal(await head.text(), "");

  const post = await fetch(`${origin}/`, { method: "POST" });
  assert.equal(post.status, 405);
  assert.equal(post.headers.get("allow"), "GET, HEAD");
});
