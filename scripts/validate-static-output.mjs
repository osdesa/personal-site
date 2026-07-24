import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";

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
    "cv/Hayden-Farrell-CV.pdf",
    "images/project-default.svg",
  ]) {
    assert(names.includes(expected), `missing required production asset: ${expected}`);
  }
  assert(names.some((name) => name.endsWith(".css")), "missing generated CSS asset");
  assert(names.some((name) => name.endsWith(".js")), "missing generated JavaScript asset");
  assert(names.some((name) => name.endsWith(".wasm")), "missing generated WebAssembly asset");

  for (const expected of [
    `<link rel="canonical" href="${origin}/"`,
    `<meta property="og:url" content="${origin}/"`,
    `<meta property="og:image" content="${origin}/images/project-default.svg"`,
  ]) {
    assert(index.includes(expected), `index.html is missing production metadata: ${expected}`);
  }

  const productionText = Buffer.concat(await Promise.all(files.map((file) => readFile(file)))).toString("utf8");
  for (const prohibited of ["localhost", ".pages.dev", ...tokenNames]) {
    assert(!productionText.includes(prohibited), `production output contains prohibited text: ${prohibited}`);
  }
  assert(productionText.includes(origin), "production output does not contain the canonical origin");

  const sitemap = await readFile(resolve(outputDirectory, "sitemap.xml"), "utf8");
  const locations = [...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => match[1]);
  assert(
    JSON.stringify(locations) === JSON.stringify([`${origin}/`, `${origin}/projects`, `${origin}/cv`, `${origin}/legal-notice`]),
    "sitemap.xml must contain exactly the four public canonical routes",
  );
  const robots = await readFile(resolve(outputDirectory, "robots.txt"), "utf8");
  assert(robots.includes(`Sitemap: ${origin}/sitemap.xml`), "robots.txt has the wrong sitemap URL");

  console.log(`Validated ${files.length} production files in ${outputDirectory}.`);
}

main().catch((error) => {
  console.error(`Static output validation failed: ${error.message}`);
  process.exitCode = 1;
});
