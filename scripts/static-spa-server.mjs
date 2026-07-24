import { createReadStream, readFileSync } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

const CONTENT_TYPES = new Map([
  [".avif", "image/avif"],
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".jpeg", "image/jpeg"],
  [".jpg", "image/jpeg"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".mjs", "text/javascript; charset=utf-8"],
  [".pdf", "application/pdf"],
  [".png", "image/png"],
  [".svg", "image/svg+xml"],
  [".txt", "text/plain; charset=utf-8"],
  [".wasm", "application/wasm"],
  [".webp", "image/webp"],
  [".xml", "application/xml; charset=utf-8"],
]);

/**
 * Creates a static server that applies the production SPA fallback only to
 * non-asset requests. Both browser checks and Lighthouse use this boundary so
 * they validate the exact Trunk output that CI publishes as an artifact.
 */
export function createStaticSpaServer(root = resolve("dist")) {
  const absoluteRoot = resolve(root);
  const securityHeaders = universalHeaders(absoluteRoot);

  return createServer(async (request, response) => {
    try {
      if (!["GET", "HEAD"].includes(request.method ?? "")) {
        sendText(response, securityHeaders, 405, "Method not allowed.", {
          allow: "GET, HEAD",
        });
        return;
      }

      let pathname;
      try {
        pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
      } catch {
        sendText(response, securityHeaders, 400, "Invalid request path.");
        return;
      }

      const candidate = resolve(absoluteRoot, `.${pathname}`);
      const safeCandidate =
        candidate === absoluteRoot || candidate.startsWith(`${absoluteRoot}${sep}`);
      const file = safeCandidate ? await stat(candidate).catch(() => null) : null;
      if (!file && pathname.includes(".")) {
        sendText(response, securityHeaders, 404, "Static asset not found.");
        return;
      }

      const target = file?.isFile() ? candidate : resolve(absoluteRoot, "index.html");
      response.writeHead(200, {
        ...securityHeaders,
        "content-type": contentType(target),
      });
      if (request.method === "HEAD") {
        response.end();
        return;
      }
      createReadStream(target)
        .on("error", () => response.destroy())
        .pipe(response);
    } catch {
      sendText(response, securityHeaders, 500, "Unable to serve the production bundle.");
    }
  });
}

function contentType(path) {
  return CONTENT_TYPES.get(extname(path).toLowerCase()) ?? "application/octet-stream";
}

function sendText(response, securityHeaders, status, body, headers = {}) {
  response.writeHead(status, {
    ...securityHeaders,
    "content-type": "text/plain; charset=utf-8",
    ...headers,
  });
  response.end(body);
}

function universalHeaders(root) {
  const source = readFileSync(resolve(root, "_headers"), "utf8");
  const block = source.match(/^\/\*\r?\n((?: {2}.+(?:\r?\n|$))*)/m)?.[1];
  if (!block) {
    throw new Error("The static bundle is missing its universal response headers.");
  }

  return Object.fromEntries(
    block
      .trim()
      .split(/\r?\n/)
      .map((line) => {
        const separator = line.indexOf(":");
        if (separator < 1) {
          throw new Error(`Invalid universal response header: ${line.trim()}`);
        }
        return [
          line.slice(0, separator).trim().toLowerCase(),
          line.slice(separator + 1).trim(),
        ];
      }),
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const port = portArgument(process.argv.slice(2));
  const server = createStaticSpaServer();

  await new Promise((resolveServer) => server.listen(port, "127.0.0.1", resolveServer));
  console.log(`Serving dist/ at http://127.0.0.1:${port}`);

  const close = () => server.close(() => process.exit(0));
  process.once("SIGINT", close);
  process.once("SIGTERM", close);
}

function portArgument(arguments_) {
  const index = arguments_.indexOf("--port");
  const value = index === -1 ? undefined : Number(arguments_[index + 1]);
  if (!Number.isInteger(value) || value < 1 || value > 65_535) {
    throw new Error("A valid --port value is required.");
  }
  return value;
}
