import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const routes = [
  {
    name: "home",
    path: "/",
    title: "Hayden Farrell | Software Engineer",
    description: "Hayden Farrell - software engineer and computer science student.",
  },
  {
    name: "projects",
    path: "/projects",
    title: "Projects | Hayden Farrell",
    description: "Selected software engineering projects and technical case studies.",
  },
  {
    name: "CV",
    path: "/cv",
    title: "CV | Hayden Farrell",
    description:
      "Hayden Farrell's generated curriculum vitae: experience, education, projects and technical skills.",
  },
  {
    name: "not found",
    path: "/a-route-that-does-not-exist",
    title: "Page not found | Hayden Farrell",
    description:
      "The requested page could not be found. Return to Hayden Farrell's software engineering portfolio.",
  },
] as const;

for (const route of routes) {
  test(`${route.name} has no automated accessibility violations`, async ({ page }) => {
    await page.goto(route.path);
    await expect(page.locator("main")).toBeVisible();

    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });

  test(`${route.name} updates client-side title and description`, async ({ page }) => {
    await page.goto(route.path);

    await expect(page).toHaveTitle(route.title);
    await expect(page.locator('meta[name="description"]')).toHaveAttribute(
      "content",
      route.description,
    );
  });
}

test("written project content remains available when a project image fails", async ({ page }) => {
  await page.route("**/images/**", (route) => route.abort("failed"));
  await page.goto("/projects");

  await expect(page.getByRole("heading", { name: "Projects" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Personal Website" })).toBeVisible();
});

test("written CV remains available while its PDF download is slow or fails", async ({ page }) => {
  await page.route("**/cv/Hayden-Farrell-CV.pdf", async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 750));
    await route.abort("failed");
  });
  await page.goto("/cv");

  const download = page.getByRole("link", { name: "Download CV as a PDF" });
  await expect(download).toHaveAttribute("download", "Hayden-Farrell-CV.pdf");

  const failedDownload = page.evaluate(async () => {
    try {
      await fetch("/cv/Hayden-Farrell-CV.pdf");
      return "loaded";
    } catch {
      return "failed";
    }
  });

  await expect(page.getByRole("heading", { name: "Hayden Farrell" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Professional experience" })).toBeVisible();
  await expect(failedDownload).resolves.toBe("failed");
});

test("mounted structured data is valid and excludes private contact data", async ({ page }) => {
  await page.goto("/");

  const structuredData = await page.locator('script[type="application/ld+json"]').textContent();
  expect(structuredData).not.toBeNull();

  const graph = JSON.parse(structuredData!)["@graph"];
  expect(graph).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ "@type": "Person", name: "Hayden Farrell" }),
      expect.objectContaining({ "@type": "WebSite", name: "Hayden Farrell" }),
    ]),
  );
  expect(structuredData).not.toContain("haydenfarrell@outlook.com");
  expect(structuredData).toContain('"url":"https://haydenfarrell.dev"');
});

test("skip link moves keyboard focus into main content", async ({ page }) => {
  await page.goto("/");

  const skipLink = page.getByRole("link", { name: "Skip to main content" });
  await skipLink.focus();
  await expect(skipLink).toBeFocused();
  await page.keyboard.press("Enter");

  await expect(page.locator("main")).toBeFocused();
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
