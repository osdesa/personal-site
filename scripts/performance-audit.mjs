import { mkdir, readFile, rm, stat } from "node:fs/promises";
import { resolve } from "node:path";
import { spawn } from "node:child_process";
import { chromium } from "@playwright/test";

import { createStaticSpaServer } from "./static-spa-server.mjs";

const routes = ["/", "/projects", "/cv"];
const runsPerRoute = 3;
const categories = ["performance", "accessibility", "best-practices", "seo"];
const reportDirectory = resolve(".lighthouseci");
const staticDirectory = resolve("dist");
const lighthouseCli = resolve("node_modules/lighthouse/cli/index.js");
const chromeFlags = ["--headless=new", "--no-sandbox", "--disable-dev-shm-usage", "--disable-gpu"];
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

function runLighthouse(url, reportPath) {
  const args = [
    lighthouseCli,
    url,
    "--preset=desktop",
    `--only-categories=${categories.join(",")}`,
    "--output=json",
    `--output-path=${reportPath}`,
    `--chrome-path=${chromium.executablePath()}`,
    `--chrome-flags=${chromeFlags.join(" ")}`,
    "--quiet",
  ];

  const attemptRun = (attempt) =>
    new Promise((resolveRun, rejectRun) => {
      const child = spawn(process.execPath, args, { stdio: "inherit" });
      child.once("error", rejectRun);
      child.once("exit", async (code) => {
        if (code === 0) resolveRun();
        else if (process.platform === "win32" && (await completedReportExists(reportPath))) {
          console.warn(`Lighthouse wrote ${reportPath} but could not clean up its Windows profile.`);
          resolveRun();
        } else if (attempt === 1) {
          console.warn(`Lighthouse did not complete ${url}; retrying once with a fresh Chromium process.`);
          attemptRun(2).then(resolveRun, rejectRun);
        } else {
          rejectRun(new Error(`Lighthouse exited with status ${code ?? "unknown"} for ${url}.`));
        }
      });
    });

  return attemptRun(1);
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
