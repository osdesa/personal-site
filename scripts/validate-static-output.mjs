import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";

import { bootstrapHash } from "./finalize-static-output.mjs";

const outputDirectory = resolve(process.argv[2] ?? "dist");
const origin = "https://haydenfarrell.dev";
const tokenNames = [
  "CV_SYNC_TOKEN",
  "PORTFOLIO_SYNC_TOKEN",
  "PORTFOLIO_GITHUB_TOKEN",
];

async function filesIn(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const path = resolve(directory, entry.name);
      return entry.isDirectory() ? filesIn(path) : [path];
    }),
  );
  return files.flat();
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const files = await filesIn(outputDirectory);
  const names = files.map((file) => file.slice(outputDirectory.length + 1).replaceAll("\\", "/"));
  const index = await readFile(resolve(outputDirectory, "index.html"), "utf8");

  for (const expected of [
    "index.html",
    "robots.txt",
    "sitemap.xml",
    "favicon.svg",
    "_headers",
    "_redirects",
    "cv/Hayden-Farrell-CV.pdf",
    "images/project-default.svg",
  ]) {
    assert(names.includes(expected), `missing required production asset: ${expected}`);
  }
  assert(names.some((name) => name.endsWith(".css")), "missing generated CSS asset");
  assert(names.some((name) => name.endsWith(".js")), "missing generated JavaScript asset");
  assert(names.some((name) => name.endsWith(".wasm")), "missing generated WebAssembly asset");

  for (const expected of [
    `<link id="site-canonical" rel="canonical" href="${origin}/"`,
    `<meta id="site-og-url" property="og:url" content="${origin}/"`,
    `<meta property="og:image" content="${origin}/images/project-default.svg"`,
  ]) {
    assert(index.includes(expected), `index.html is missing production metadata: ${expected}`);
  }

  const productionText = Buffer.concat(await Promise.all(files.map((file) => readFile(file)))).toString("utf8");
  for (const prohibited of ["localhost", ...tokenNames]) {
    assert(!productionText.includes(prohibited), `production output contains prohibited text: ${prohibited}`);
  }
  const publicContent = Buffer.concat(
    await Promise.all(
      files
        .filter((file) => file !== resolve(outputDirectory, "_headers"))
        .map((file) => readFile(file)),
    ),
  ).toString("utf8");
  assert(
    !publicContent.includes(".pages.dev"),
    "public production content contains a Pages preview hostname",
  );
  assert(productionText.includes(origin), "production output does not contain the canonical origin");

  const sitemap = await readFile(resolve(outputDirectory, "sitemap.xml"), "utf8");
  const locations = [...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => match[1]);
  assert(
    JSON.stringify(locations) === JSON.stringify([`${origin}/`, `${origin}/projects`, `${origin}/cv`, `${origin}/legal`, `${origin}/privacy`]),
    "sitemap.xml must contain exactly the five public canonical routes",
  );
  const robots = await readFile(resolve(outputDirectory, "robots.txt"), "utf8");
  assert(robots.includes(`Sitemap: ${origin}/sitemap.xml`), "robots.txt has the wrong sitemap URL");
  const redirects = await readFile(resolve(outputDirectory, "_redirects"), "utf8");
  assert(
    redirects.split(/\r?\n/).includes("/legal-notice /legal 301"),
    "_redirects must preserve the legacy legal notice URL",
  );

  const headers = await readFile(resolve(outputDirectory, "_headers"), "utf8");
  for (const expected of [
    "Content-Security-Policy:",
    "Permissions-Policy:",
    "Referrer-Policy: strict-origin-when-cross-origin",
    "X-Content-Type-Options: nosniff",
    "X-Frame-Options: DENY",
    "Cache-Control: public, max-age=31536000, immutable",
    "X-Robots-Tag: noindex",
  ]) {
    assert(headers.includes(expected), `_headers is missing: ${expected}`);
  }
  assert(
    headers.includes(bootstrapHash(index)),
    "_headers does not allow the exact generated Trunk bootstrap",
  );
  assert(
    !headers.includes("__TRUNK_BOOTSTRAP_HASH__"),
    "_headers contains an unresolved bootstrap hash placeholder",
  );

  console.log(`Validated ${files.length} production files in ${outputDirectory}.`);
}

main().catch((error) => {
  console.error(`Static output validation failed: ${error.message}`);
  process.exitCode = 1;
});
