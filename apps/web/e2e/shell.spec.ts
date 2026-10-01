import { test, expect } from "@playwright/test";

test("shell renders routes and remains usable at desktop and narrow widths", async ({
  page,
}) => {
  for (const width of [1120, 360]) {
    await page.setViewportSize({ width, height: 820 });
    for (const [hash, heading] of [
      ["#/runs", "Runs"],
      ["#/new", "New Run"],
      ["#/runs/run-shell-test", "Run detail"],
      ["#/diagnostics", "Diagnostics"],
    ]) {
      await page.goto(`/${hash}`);
      await expect(
        page.getByRole("heading", { name: heading, exact: true }),
      ).toBeVisible();
      await expect(
        page.getByRole("navigation", { name: "Main navigation" }),
      ).toBeVisible();
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
      ).toBe(true);
    }
  }
  await page.getByRole("link", { name: "New Run", exact: true }).click();
  await expect(page).toHaveURL(/#\/new$/);
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "New Run", exact: true }),
  ).toBeVisible();
});
