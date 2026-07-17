import { createReadStream } from "node:fs";
import { mkdir, readFile, rm, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve, sep } from "node:path";
import { spawn } from "node:child_process";

const routes = ["/", "/projects", "/cv"];
const runsPerRoute = 3;
const categories = ["performance", "accessibility", "best-practices", "seo"];
const reportDirectory = resolve(".lighthouseci");
const staticDirectory = resolve("dist");
const lighthouseCli = resolve("node_modules/lighthouse/cli/index.js");
const budgets = {
  performance: 0.9,
  accessibility: 1,
  "best-practices": 1,
  seo: 1,
  "total-byte-weight": 550_000,
  "cumulative-layout-shift": 0.05,
};

await stat(staticDirectory);
await rm(reportDirectory, { recursive: true, force: true });
await mkdir(reportDirectory, { recursive: true });

const server = createStaticSpaServer(staticDirectory);
await new Promise((resolveServer) => server.listen(0, "127.0.0.1", resolveServer));
const address = server.address();
if (!address || typeof address === "string") {
  throw new Error("Performance server did not expose a TCP port.");
}

try {
  const reports = [];
  for (const route of routes) {
    for (let run = 1; run <= runsPerRoute; run += 1) {
      const reportPath = resolve(reportDirectory, `${routeName(route)}-${run}.report.json`);
      await runLighthouse(`http://127.0.0.1:${address.port}${route}`, reportPath);
      reports.push(JSON.parse(await readFile(reportPath, "utf8")));
    }
  }
  assertBudgets(reports);
} finally {
  await new Promise((resolveServer, rejectServer) =>
    server.close((error) => (error ? rejectServer(error) : resolveServer())),
  );
}

function createStaticSpaServer(root) {
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

function runLighthouse(url, reportPath) {
  const args = [
    lighthouseCli,
    url,
    "--preset=desktop",
    `--only-categories=${categories.join(",")}`,
    "--output=json",
    `--output-path=${reportPath}`,
    "--quiet",
  ];

  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(process.execPath, args, { stdio: "inherit" });
    child.once("error", rejectRun);
    child.once("exit", async (code) => {
      if (code === 0) resolveRun();
      else if (process.platform === "win32" && (await completedReportExists(reportPath))) {
        console.warn(`Lighthouse wrote ${reportPath} but could not clean up its Windows profile.`);
        resolveRun();
      } else {
        rejectRun(new Error(`Lighthouse exited with status ${code ?? "unknown"} for ${url}.`));
      }
    });
  });
}

async function completedReportExists(reportPath) {
  try {
    return (await stat(reportPath)).size > 0;
  } catch {
    return false;
  }
}

function assertBudgets(reports) {
  const failures = [];
  for (const category of categories) {
    const score = median(reports.map((report) => report.categories[category].score));
    assertAtLeast(failures, `categories:${category}`, score, budgets[category]);
  }

  const transferredBytes = median(reports.map((report) => report.audits["total-byte-weight"].numericValue));
  assertAtMost(failures, "total-byte-weight", transferredBytes, budgets["total-byte-weight"]);

  const layoutShift = median(
    reports.map((report) => report.audits["cumulative-layout-shift"].numericValue),
  );
  assertAtMost(failures, "cumulative-layout-shift", layoutShift, budgets["cumulative-layout-shift"]);

  if (failures.length > 0) {
    throw new Error(`Performance budgets failed:\n${failures.join("\n")}`);
  }
}

function assertAtLeast(failures, name, actual, minimum) {
  console.log(`${name}: ${actual.toFixed(3)} (minimum ${minimum.toFixed(3)})`);
  if (actual < minimum) failures.push(`${name} was ${actual}, below ${minimum}`);
}

function assertAtMost(failures, name, actual, maximum) {
  console.log(`${name}: ${actual.toFixed(3)} (maximum ${maximum.toFixed(3)})`);
  if (actual > maximum) failures.push(`${name} was ${actual}, above ${maximum}`);
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

function routeName(route) {
  return route === "/" ? "home" : route.slice(1);
}
