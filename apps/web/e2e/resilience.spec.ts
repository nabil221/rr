import { test, expect } from "@playwright/test";
test("detail polling observes execution performed by another local client", async ({
  page,
  request,
}) => {
  await page.goto("/#/new");
  await page.getByRole("button", { name: "Create queued run" }).click();
  await expect(page.getByRole("status")).toHaveText("queued");
  const id = decodeURIComponent(
    new URL(page.url()).hash.slice("#/runs/".length),
  );
  const response = await request.post(
    `http://127.0.0.1:3001/api/runs/${encodeURIComponent(id)}/execute`,
  );
  expect(response.status()).toBe(200);
  await expect(page.getByRole("status")).toHaveText("completed");
  await expect(
    page.getByRole("heading", { name: "Lifecycle events (3)" }),
  ).toBeVisible();
});
test("history reports a transport failure and recovers through explicit retry", async ({
  page,
}) => {
  await page.route("**/api/runs", (route) => route.abort("failed"));
  await page.goto("/#/runs");
  await expect(page.getByRole("alert")).toContainText(
    "The local host could not be reached",
  );
  await page.unroute("**/api/runs");
  await page.getByRole("button", { name: "Refresh runs" }).click();
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByRole("list", { name: "Saved runs" })).toHaveAttribute(
    "aria-busy",
    "false",
  );
});
