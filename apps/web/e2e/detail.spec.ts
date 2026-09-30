import { test, expect } from "@playwright/test";

for (const rejected of [false, true]) {
  test(`execute and reload ${rejected ? "rejected" : "completed"} detail`, async ({
    page,
  }, testInfo) => {
    await page.goto("/#/new");
    if (rejected)
      await page.getByLabel("Force a controlled validation rejection").check();
    await page.getByRole("button", { name: "Create queued run" }).click();
    await expect(
      page.getByRole("button", { name: "Execute run", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Configuration snapshot" }),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "Execute run", exact: true })
      .click();
    await expect(page.getByRole("status")).toHaveText(
      rejected ? "rejected" : "completed",
    );
    await expect(
      page.getByRole("heading", { name: "Lifecycle events (3)", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Execute run", exact: true }),
    ).toHaveCount(0);
    if (rejected)
      await expect(
        page.getByRole("heading", { name: "Validation rejected this run" }),
      ).toBeVisible();
    else await expect(page.getByText("64.14", { exact: true })).toBeVisible();
    await page.reload();
    await expect(page.getByRole("status")).toHaveText(
      rejected ? "rejected" : "completed",
    );
    await expect(
      page.getByRole("heading", { name: "Lifecycle events (3)", exact: true }),
    ).toBeVisible();
    if (!rejected) {
      for (const width of [1120, 360]) {
        await page.setViewportSize({ width, height: 820 });
        expect(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= window.innerWidth,
          ),
        ).toBe(true);
        await page.screenshot({
          path: testInfo.outputPath(`completed-${width}.png`),
          fullPage: true,
        });
      }
    }
  });
}
test("diagnostics identifies the local web host and Postgres safely", async ({
  page,
}) => {
  await page.goto("/#/diagnostics");
  await expect(page.getByText("web-local", { exact: true })).toBeVisible();
  await expect(page.getByText("postgres", { exact: true })).toBeVisible();
  await expect(page.locator("main")).not.toContainText("postgresql://");
  await expect(page.locator("main")).not.toContainText("54329");
});
