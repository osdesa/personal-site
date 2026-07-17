import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const routes = [
  { name: "home", path: "/" },
  { name: "projects", path: "/projects" },
  { name: "CV", path: "/cv" },
  { name: "not found", path: "/a-route-that-does-not-exist" },
] as const;

for (const route of routes) {
  test(`${route.name} has no automated accessibility violations`, async ({ page }) => {
    await page.goto(route.path);
    await expect(page.locator("main")).toBeVisible();

    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });
}

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
    await expect(page.locator(":focus")).toHaveText("01Home");
    await page.keyboard.press("Tab");
    await expect(page.locator(":focus")).toHaveText("02Projects");
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
