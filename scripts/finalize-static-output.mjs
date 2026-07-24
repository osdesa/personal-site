import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const HASH_PLACEHOLDER = "__TRUNK_BOOTSTRAP_HASH__";
const INLINE_MODULE = /<script\s+type="module">([\s\S]*?)<\/script>/g;

export function bootstrapHash(html) {
  const modules = [...html.matchAll(INLINE_MODULE)];
  if (modules.length !== 1) {
    throw new Error(`expected one inline Trunk module, found ${modules.length}`);
  }

  const digest = createHash("sha256").update(modules[0][1]).digest("base64");
  return `'sha256-${digest}'`;
}

export async function finalizeStaticOutput(directory) {
  const outputDirectory = resolve(directory);
  const indexPath = resolve(outputDirectory, "index.html");
  const headersPath = resolve(outputDirectory, "_headers");
  const [html, headers] = await Promise.all([
    readFile(indexPath, "utf8"),
    readFile(headersPath, "utf8"),
  ]);

  const placeholders = headers.split(HASH_PLACEHOLDER).length - 1;
  if (placeholders !== 1) {
    throw new Error(`expected one ${HASH_PLACEHOLDER} placeholder, found ${placeholders}`);
  }

  await writeFile(headersPath, headers.replace(HASH_PLACEHOLDER, bootstrapHash(html)), "utf8");
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const outputDirectory = process.env.TRUNK_STAGING_DIR ?? process.argv[2] ?? "dist";
  await finalizeStaticOutput(outputDirectory);
}
