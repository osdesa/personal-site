import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const routes = [
  {
    name: "home",
    path: "/",
    indexable: true,
  },
  {
    name: "projects",
    path: "/projects",
    indexable: true,
  },
  {
    name: "CV",
    path: "/cv",
    indexable: true,
  },
  {
    name: "legal notice",
    path: "/legal",
    indexable: true,
  },
  {
    name: "privacy notice",
    path: "/privacy",
    indexable: true,
  },
  {
    name: "not found",
    path: "/a-route-that-does-not-exist",
    indexable: false,
  },
] as const;

for (const route of routes) {
  test(`${route.name} has no automated accessibility violations`, async ({ page }) => {
    await page.goto(route.path);
    await expect(page.locator("main")).toBeVisible();

    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });

  test(`${route.name} publishes complete client-side metadata`, async ({ page }) => {
    await page.goto(route.path);

    expect((await page.title()).trim()).not.toBe("");
    const description = page.locator('meta[name="description"]');
    await expect(description).toHaveCount(1);
    expect((await description.getAttribute("content"))?.trim()).not.toBe("");

    const canonical = page.locator('link[rel="canonical"]');
    await expect(canonical).toHaveCount(1);
    const canonicalUrl = new URL((await canonical.getAttribute("href"))!, page.url());
    expect(canonicalUrl.protocol).toBe("https:");
    if (route.indexable) {
      expect(canonicalUrl.pathname).toBe(route.path);
    }

    await expect(page.locator('meta[property="og:url"]')).toHaveCount(1);
    if (route.indexable) {
      await expect(page.locator('meta[name="robots"]')).toHaveCount(0);
    } else {
      await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
        "content",
        "noindex, nofollow",
      );
    }
  });
}

test("legacy legal notice links redirect to the canonical route", async ({ page }) => {
  await page.goto("/legal-notice");

  await expect(page).toHaveURL("/legal");
  await expect(page.getByRole("heading", { name: "Legal notice" })).toBeVisible();
});

test("home heading remains separated from its supporting content", async ({ page }) => {
  for (const viewport of [
    { width: 320, height: 568 },
    { width: 768, height: 1024 },
    { width: 1440, height: 900 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto("/");

    const heading = await page.getByRole("heading", { level: 1 }).boundingBox();
    const role = await page.locator(".home-page__role").boundingBox();

    expect(heading).not.toBeNull();
    expect(role).not.toBeNull();
    expect(role!.y - (heading!.y + heading!.height)).toBeGreaterThanOrEqual(16);
  }
});

test("404 decoration stays above and clear of supporting content", async ({ page }) => {
  for (const viewport of [
    { width: 320, height: 568 },
    { width: 1440, height: 900 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto("/a-route-that-does-not-exist");

    const code = await page.locator(".not-found__code").boundingBox();
    const eyebrow = await page.getByText("Route not found", { exact: true }).boundingBox();

    expect(code).not.toBeNull();
    expect(eyebrow).not.toBeNull();
    expect(code!.y + code!.height).toBeLessThanOrEqual(eyebrow!.y);
    expect(code!.y).toBeLessThan(viewport.height * 0.4);
  }
});

test("written CV remains available while its PDF download is slow or fails", async ({ page }) => {
  await page.goto("/cv");

  const download = page.getByRole("link", { name: "Download CV as a PDF" });
  const href = await download.getAttribute("href");
  expect(href).not.toBeNull();
  const pdfPath = new URL(href!, page.url()).pathname;
  expect(pdfPath.endsWith(".pdf")).toBeTruthy();
  expect((await download.getAttribute("download"))?.trim()).not.toBe("");

  await page.route(`**${pdfPath}`, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 750));
    await route.abort("failed");
  });

  const failedDownload = page.evaluate(async () => {
    try {
      const pdfLink = document.querySelector<HTMLAnchorElement>("a[download]");
      if (!pdfLink) return "missing";
      await fetch(pdfLink.href);
      return "loaded";
    } catch {
      return "failed";
    }
  });

  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.locator("main address")).toBeVisible();
  await expect(failedDownload).resolves.toBe("failed");
});

test("mounted structured data is valid and excludes private contact data", async ({ page }) => {
  await page.goto("/");

  const structuredData = await page.locator('script[type="application/ld+json"]').textContent();
  expect(structuredData).not.toBeNull();

  const graph = JSON.parse(structuredData!)["@graph"];
  const person = graph.find((entry: { "@type": string }) => entry["@type"] === "Person");
  const website = graph.find((entry: { "@type": string }) => entry["@type"] === "WebSite");
  expect(person?.name?.trim()).not.toBe("");
  expect(person).not.toHaveProperty("email");
  expect(Array.isArray(person?.sameAs)).toBeTruthy();
  expect(person.sameAs.every((url: string) => url.startsWith("https://"))).toBeTruthy();
  expect(website?.name?.trim()).not.toBe("");
  expect(new URL(website.url).protocol).toBe("https:");
});

test("skip link moves keyboard focus into main content", async ({ page }) => {
  await page.goto("/");

  const skipLink = page.getByRole("link", { name: "Skip to main content" });
  await skipLink.focus();
  await expect(skipLink).toBeFocused();
  await page.keyboard.press("Enter");

  await expect(page.locator("main")).toBeFocused();
});

test("project images fit within narrow mobile cards", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 568 });
  await page.goto("/projects");

  const imageBounds = await page.locator(".project-card__image").evaluateAll((images) =>
    images.map((image) => {
      const { left, right, width, height } = image.getBoundingClientRect();
      return { left, right, width, height };
    }),
  );

  for (const image of imageBounds) {
    expect(image.left).toBeGreaterThanOrEqual(0);
    expect(image.right).toBeLessThanOrEqual(320);
    expect(image.width).toBeGreaterThan(0);
    expect(image.height).toBeGreaterThan(0);
  }
});

test("footer notices are grouped under Navigate", async ({ page }) => {
  await page.goto("/");

  const footerNavigation = page.getByRole("navigation", { name: "Footer navigation" });
  await expect(footerNavigation.getByRole("link")).toHaveText([
    "Projects",
    "CV",
    "Legal notice",
    "Privacy notice",
  ]);
  await expect(page.getByRole("navigation", { name: "Legal information" })).toHaveCount(0);
});

test("notice headings have clear separation from their supporting text", async ({ page }) => {
  for (const path of ["/legal", "/privacy"]) {
    for (const viewport of [
      { width: 320, height: 568 },
      { width: 1440, height: 900 },
    ]) {
      await page.setViewportSize(viewport);
      await page.goto(path);

      const heading = await page.locator(".page-hero--compact h1").boundingBox();
      const lead = await page.locator(".page-hero--compact .page-hero__lead").boundingBox();

      expect(heading).not.toBeNull();
      expect(lead).not.toBeNull();
      expect(lead!.y - (heading!.y + heading!.height)).toBeGreaterThanOrEqual(20);
    }
  }
});

test.describe("mobile navigation", () => {
  test.use({ viewport: { width: 320, height: 568 } });

  test("keeps closed links inert, opens predictably, and returns focus to main", async ({ page }) => {
    await page.goto("/");

    const openMenuButton = page.getByRole("button", { name: "Open navigation menu" });
    const mobileNavigation = page.locator("#mobile-navigation");

    await expect(openMenuButton).toHaveAttribute("aria-expanded", "false");
    await expect(mobileNavigation).toHaveAttribute("inert", "");
    expect(
      await page.locator("html").evaluate((html) => html.scrollWidth <= window.innerWidth),
    ).toBeTruthy();

    await openMenuButton.focus();
    await page.keyboard.press("Tab");
    await expect(page.locator(":focus")).toHaveText("View projects↗");

    await openMenuButton.click();
    const closeMenuButton = page.getByRole("button", { name: "Close navigation menu" });
    await expect(closeMenuButton).toHaveAttribute("aria-expanded", "true");
    await expect(mobileNavigation).not.toHaveAttribute("inert", "");
    const mobileLinkPadding = await mobileNavigation
      .getByRole("link", { name: "Home" })
      .evaluate((link) => {
        const style = getComputedStyle(link);
        return {
          left: Number.parseFloat(style.paddingLeft),
          right: Number.parseFloat(style.paddingRight),
        };
      });
    expect(mobileLinkPadding.left).toBeGreaterThanOrEqual(12);
    expect(mobileLinkPadding.right).toBeGreaterThanOrEqual(12);

    await closeMenuButton.focus();
    await page.keyboard.press("Tab");
    await expect(page.locator(":focus")).toHaveText("Home");
    await page.keyboard.press("Tab");
    await expect(page.locator(":focus")).toHaveText("Projects");
    await page.keyboard.press("Enter");

    await expect(page).toHaveURL("/projects");
    await expect(page.locator("main")).toBeFocused();
  });
});

test("reduced motion removes animated transitions", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");

  const transitionDuration = await page
    .getByRole("link", { name: "View projects" })
    .evaluate((button) => getComputedStyle(button).transitionDuration);

  expect(Number.parseFloat(transitionDuration)).toBeLessThanOrEqual(0.00001);
});
