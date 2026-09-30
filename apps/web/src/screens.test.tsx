// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { act } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { ApiError, type RunClient, type RunDto } from "./api";
import { DetailScreen } from "./detail";
import { DiagnosticsScreen } from "./diagnostics";

function snapshot(status: RunDto["status"]): RunDto {
  return {
    id: "run-test",
    configuration: {
      seed: 42,
      region: "north",
      threshold: 0,
      forceValidationFailure: false,
    },
    status,
    result:
      status === "completed"
        ? {
            totalRecords: 24,
            matchedRecords: 0,
            totalScore: 0,
            averageScore: 0,
            groups: [],
          }
        : null,
    validationMessages:
      status === "rejected"
        ? [{ code: "rejected", message: "Controlled rejection", field: null }]
        : [],
    events: [
      { kind: "created", message: "Run created" },
      ...(status !== "queued"
        ? [{ kind: "started" as const, message: "Run started" }]
        : []),
      ...(["completed", "rejected", "failed"].includes(status)
        ? [
            {
              kind: status as "completed" | "rejected" | "failed",
              message: "Terminal event",
            },
          ]
        : []),
    ],
  };
}
function client(): RunClient {
  return {
    listRuns: vi.fn().mockResolvedValue([]),
    getRun: vi.fn().mockResolvedValue(snapshot("queued")),
    createRun: vi.fn(),
    executeRun: vi.fn().mockResolvedValue(snapshot("completed")),
    diagnostics: vi.fn().mockResolvedValue({
      host: "desktop-local",
      transport: "embedded-protocol",
      storage: "in-memory",
      status: "ok",
    }),
  };
}
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("detail observation", () => {
  it("polls nonterminal states and stops at terminal without overlapping reads", async () => {
    vi.useFakeTimers();
    const api = client();
    vi.mocked(api.getRun)
      .mockResolvedValueOnce(snapshot("queued"))
      .mockResolvedValueOnce(snapshot("running"))
      .mockResolvedValue(snapshot("completed"));
    render(<DetailScreen client={api} id="run-test" />);
    await act(async () => {});
    expect(screen.getByRole("status").textContent).toBe("queued");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(screen.getByRole("status").textContent).toBe("running");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(screen.getByRole("status").textContent).toBe("completed");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(api.getRun).toHaveBeenCalledTimes(3);
    expect(screen.getByText("0.00", { exact: true })).toBeTruthy();
    expect(screen.queryByText("Validation rejected this run")).toBeNull();
  });
  it("cancels polling and waiting when leaving the detail screen", async () => {
    vi.useFakeTimers();
    const api = client();
    const view = render(<DetailScreen client={api} id="run-test" />);
    await act(async () => {});
    const signal = vi.mocked(api.getRun).mock.calls[0][1];
    view.unmount();
    expect(signal?.aborted).toBe(true);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(api.getRun).toHaveBeenCalledTimes(1);
    expect(vi.getTimerCount()).toBe(0);
  });
  it("does not regress a completed execution when an older poll resolves late", async () => {
    vi.useFakeTimers();
    const api = client();
    let finishRead!: (value: RunDto) => void;
    vi.mocked(api.getRun)
      .mockResolvedValueOnce(snapshot("queued"))
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            finishRead = resolve;
          }),
      );
    render(<DetailScreen client={api} id="run-test" />);
    await act(async () => {});
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Execute run" }));
    });
    await act(async () => {
      finishRead(snapshot("queued"));
    });
    expect(screen.getByRole("status").textContent).toBe("completed");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(api.getRun).toHaveBeenCalledTimes(2);
    expect(api.executeRun).toHaveBeenCalledTimes(1);
  });
  it("stops on a read failure and renders a safe retryable error", async () => {
    vi.useFakeTimers();
    const api = client();
    vi.mocked(api.getRun).mockRejectedValue(
      new ApiError(
        404,
        { code: "not-found", message: "Run not found", field: null },
        "request-test",
      ),
    );
    render(<DetailScreen client={api} id="missing" />);
    await act(async () => {});
    expect(screen.getByRole("alert").textContent).toContain("Run not found");
    expect(screen.getByRole("alert").textContent).toContain("request-test");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(api.getRun).toHaveBeenCalledTimes(1);
  });
});
it("shows native diagnostics through the same injected screen", async () => {
  render(<DiagnosticsScreen client={client()} />);
  expect(
    await screen.findByText("desktop-local", { exact: true }),
  ).toBeTruthy();
  expect(screen.getByText("in-memory", { exact: true })).toBeTruthy();
});
