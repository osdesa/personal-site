import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

/**
 * Creates a static server that applies the production SPA fallback only to
 * non-asset requests. Both browser checks and Lighthouse use this boundary so
 * they validate the exact Trunk output that CI publishes as an artifact.
 */
export function createStaticSpaServer(root = resolve("dist")) {
  return createServer(async (request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const candidate = resolve(root, `.${pathname}`);
    const safeCandidate = candidate === root || candidate.startsWith(`${root}${sep}`);

    try {
      const file = safeCandidate ? await stat(candidate).catch(() => null) : null;
      if (!file && pathname.includes(".")) {
        response.writeHead(404, { "content-type": "text/plain" });
        response.end("Static asset not found.");
        return;
      }

      const target = file?.isFile() ? candidate : resolve(root, "index.html");
      response.writeHead(200, { "content-type": contentType(target) });
      createReadStream(target).pipe(response);
    } catch {
      response.writeHead(500, { "content-type": "text/plain" });
      response.end("Unable to serve the production bundle.");
    }
  });
}

function contentType(path) {
  if (path.endsWith(".css")) return "text/css";
  if (path.endsWith(".js")) return "text/javascript";
  if (path.endsWith(".wasm")) return "application/wasm";
  if (path.endsWith(".svg")) return "image/svg+xml";
  if (path.endsWith(".jpg")) return "image/jpeg";
  if (path.endsWith(".pdf")) return "application/pdf";
  return "text/html";
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
