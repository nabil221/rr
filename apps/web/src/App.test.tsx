import { afterEach, describe, expect, it, vi } from "vitest";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import {
  ApiError,
  createClient,
  createRun,
  executeRun,
  listRuns,
  localApiRoot,
  type RunClient,
} from "./api";
import { renderToStaticMarkup } from "react-dom/server";
import { App } from "./App";
import { parseRoute, type Route } from "./routes";

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => false),
  convertFileSrc: vi.fn(() => "http://proof-api.localhost/"),
}));

describe("local API client", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
    vi.clearAllMocks();
    vi.mocked(isTauri).mockReturnValue(false);
  });

  it("allows only local credential-free API roots", () => {
    expect(localApiRoot("http://127.0.0.1:3000/")).toBe(
      "http://127.0.0.1:3000",
    );
    for (const value of [
      "https://example.com",
      "http://user:password@127.0.0.1:3000",
      "http://127.0.0.1:3000/api",
      "http://127.0.0.1:3000/?secret=x",
    ])
      expect(() => localApiRoot(value)).toThrow();
  });

  it("fails safely on invalid configured endpoints without exposing their contents", () => {
    vi.stubEnv("VITE_API_BASE_URL", "http://secret:password@example.com");
    const html = renderToStaticMarkup(<App initialRoute={{ kind: "runs" }} />);
    expect(html).toContain("Local configuration needs attention");
    expect(html).not.toContain("password");
    expect(html).not.toContain("example.com");
  });

  it("loads detail and safe diagnostics through the selected client", async () => {
    const fetchMock = vi.fn().mockImplementation(() =>
      Response.json({
        status: "ok",
        storage: "postgres",
        secret: "not-displayed",
      }),
    );
    vi.stubGlobal("fetch", fetchMock);
    const client = createClient("http://127.0.0.1:3000");
    await client.getRun("id with spaces");
    expect(fetchMock.mock.calls[0][0]).toBe(
      "http://127.0.0.1:3000/api/runs/id%20with%20spaces",
    );
    await expect(client.diagnostics()).resolves.toEqual({
      host: "web-local",
      transport: "loopback-http",
      storage: "postgres",
      status: "ok",
    });
    vi.mocked(isTauri).mockReturnValue(true);
    await expect(
      createClient("https://ignored.example.com").diagnostics(),
    ).resolves.toMatchObject({
      host: "desktop-local",
      transport: "embedded-protocol",
    });
  });

  it("creates runs with JSON through the same-origin API path", async () => {
    const response = { id: "run-1", status: "queued" };
    const fetchMock = vi.fn().mockResolvedValue(Response.json(response));
    vi.stubGlobal("fetch", fetchMock);

    await expect(
      createRun({
        seed: 42,
        region: "north",
        threshold: 50,
        forceValidationFailure: false,
      }),
    ).resolves.toEqual(response);
    expect(fetchMock).toHaveBeenCalledWith("/api/runs", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        seed: 42,
        region: "north",
        threshold: 50,
        forceValidationFailure: false,
      }),
    });
  });

  it("lists runs and encodes the ID when executing a run", async () => {
    const fetchMock = vi.fn().mockImplementation(() => Response.json([]));
    vi.stubGlobal("fetch", fetchMock);

    await expect(listRuns()).resolves.toEqual([]);
    await executeRun("id with spaces");

    expect(fetchMock.mock.calls[0]).toEqual(["/api/runs", undefined]);
    expect(fetchMock.mock.calls[1]).toEqual([
      "/api/runs/id%20with%20spaces/execute",
      { method: "POST" },
    ]);
  });

  it("surfaces API error messages", async () => {
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          Response.json(
            { code: "not_found", message: "Run was not found", field: null },
            { status: 404 },
          ),
        ),
    );

    await expect(listRuns()).rejects.toThrow("Run was not found");
  });

  it("uses the custom-protocol root in desktop without encoding route slashes", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    const fetchMock = vi.fn().mockImplementation(() => Response.json([]));
    vi.stubGlobal("fetch", fetchMock);
    await listRuns();
    await executeRun("id with spaces");
    expect(convertFileSrc).toHaveBeenCalledWith("", "proof-api");
    expect(fetchMock.mock.calls[0][0]).toBe(
      "http://proof-api.localhost/api/runs",
    );
    expect(fetchMock.mock.calls[1][0]).toBe(
      "http://proof-api.localhost/api/runs/id%20with%20spaces/execute",
    );
  });

  it("forwards cancellation to fetch and preserves typed API failure details", async () => {
    const fetchMock = vi.fn().mockImplementation(() =>
      Response.json(
        {
          code: "invalid-request",
          message: "Region is required",
          field: "region",
        },
        { status: 400 },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);
    const controller = new AbortController();
    const failure = await listRuns(controller.signal).catch(
      (error: unknown) => error,
    );
    expect(failure).toBeInstanceOf(ApiError);
    expect(failure).toMatchObject({
      status: 400,
      code: "invalid-request",
      field: "region",
      message: "Region is required",
    });
    expect(fetchMock.mock.calls[0][1]).toEqual({ signal: controller.signal });
  });
});

describe("host-neutral shell", () => {
  const client: RunClient = {
    listRuns: vi.fn(),
    getRun: vi.fn(),
    createRun: vi.fn(),
    executeRun: vi.fn(),
    diagnostics: vi.fn(),
  };
  it.each<[Route, string]>([
    [{ kind: "runs" }, "Runs"],
    [{ kind: "new" }, "New Run"],
    [{ kind: "detail", id: "run-1" }, "Run detail"],
    [{ kind: "diagnostics" }, "Diagnostics"],
    [{ kind: "missing" }, "Page not found"],
  ])("renders route %j", (route, title) => {
    const html = renderToStaticMarkup(
      <App client={client} initialRoute={route} />,
    );
    expect(html).toContain(title);
    expect(html).toContain('aria-label="Main navigation"');
  });
  it("parses safe hash deep links and handles invalid encodings", () => {
    expect(parseRoute("#/runs/id%20with%20spaces")).toEqual({
      kind: "detail",
      id: "id with spaces",
    });
    expect(parseRoute("#/runs/%broken")).toEqual({ kind: "missing" });
    expect(parseRoute("#/unknown")).toEqual({ kind: "missing" });
  });
});
