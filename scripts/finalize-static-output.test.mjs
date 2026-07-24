import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { bootstrapHash, finalizeStaticOutput } from "./finalize-static-output.mjs";

const bootstrap = "\nimport init from '/app.js';\nawait init('/app.wasm');\n";
const html = `<!doctype html><script type="module">${bootstrap}</script>`;

test("computes the CSP hash from the exact inline Trunk bootstrap", () => {
  const digest = createHash("sha256").update(bootstrap).digest("base64");
  assert.equal(bootstrapHash(html), `'sha256-${digest}'`);
});

test("replaces the production header placeholder exactly once", async () => {
  const directory = await mkdtemp(join(tmpdir(), "personal-site-finalize-"));
  try {
    await Promise.all([
      writeFile(join(directory, "index.html"), html),
      writeFile(
        join(directory, "_headers"),
        "/*\n  Content-Security-Policy: script-src 'self' __TRUNK_BOOTSTRAP_HASH__\n",
      ),
    ]);

    await finalizeStaticOutput(directory);

    const headers = await readFile(join(directory, "_headers"), "utf8");
    assert.match(headers, /script-src 'self' 'sha256-[A-Za-z0-9+/=]+'/);
    assert.doesNotMatch(headers, /__TRUNK_BOOTSTRAP_HASH__/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects ambiguous inline modules or header placeholders", async () => {
  assert.throws(
    () => bootstrapHash(`${html}<script type="module">duplicate()</script>`),
    /expected one inline Trunk module, found 2/,
  );

  const directory = await mkdtemp(join(tmpdir(), "personal-site-finalize-"));
  try {
    await Promise.all([
      writeFile(join(directory, "index.html"), html),
      writeFile(join(directory, "_headers"), "/*\n  Content-Security-Policy: script-src 'self'\n"),
    ]);
    await assert.rejects(() => finalizeStaticOutput(directory), /expected one .* placeholder, found 0/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
