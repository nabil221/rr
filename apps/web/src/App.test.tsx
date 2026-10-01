import { afterEach, describe, expect, it, vi } from "vitest";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { ApiError, createRun, executeRun, listRuns } from "./api";

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => false),
  convertFileSrc: vi.fn(() => "http://proof-api.localhost/"),
}));

describe("local API client", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.clearAllMocks();
    vi.mocked(isTauri).mockReturnValue(false);
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
