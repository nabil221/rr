import { test, expect } from "@playwright/test";

test("create preserves rejected input and persists a queued run in history", async ({
  page,
}) => {
  await page.goto("/#/new");
  await page.getByLabel("Seed", { exact: true }).fill("43");
  await page.getByLabel("Region", { exact: true }).fill("   ");
  await page.getByRole("button", { name: "Create queued run" }).click();
  await expect(page.getByRole("alert")).toContainText("region cannot be empty");
  await expect(page.getByLabel("Seed", { exact: true })).toHaveValue("43");
  await expect(page.getByLabel("Region", { exact: true })).toHaveValue("   ");
  await expect(page.getByLabel("Region", { exact: true })).toHaveAttribute(
    "aria-invalid",
    "true",
  );
  await page.getByLabel("Region", { exact: true }).fill("north");
  const createdResponse = page.waitForResponse(
    (response) =>
      response.url().endsWith("/api/runs") &&
      response.request().method() === "POST" &&
      response.status() === 201,
  );
  await page.getByRole("button", { name: "Create queued run" }).click();
  const created = await (await createdResponse).json();
  expect(created.status).toBe("queued");
  expect(created.events).toHaveLength(1);
  await expect(page).toHaveURL(new RegExp(`#/runs/${created.id}$`));
  await expect(page.getByRole("heading", { name: "Run detail" })).toBeVisible();
  await page.getByRole("link", { name: "Runs", exact: true }).click();
  await expect(
    page.getByRole("link", { name: `View ${created.id}` }),
  ).toContainText("queued");
  await page.reload();
  await expect(
    page.getByRole("link", { name: `View ${created.id}` }),
  ).toBeVisible();
  await page.getByRole("link", { name: `View ${created.id}` }).click();
  await expect(page).toHaveURL(new RegExp(`#/runs/${created.id}$`));
});
