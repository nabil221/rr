// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { act } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { ApiError, type RunClient, type RunDto } from "./api";
import { DetailScreen } from "./detail";
import { DiagnosticsScreen } from "./diagnostics";
import { NewRunScreen, RunsScreen } from "./screens";

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

describe("form and history states", () => {
  it("preserves submitted inputs when the server returns a field error", async () => {
    const api = client();
    vi.mocked(api.createRun).mockRejectedValue(
      new ApiError(400, {
        code: "invalid-request",
        message: "Region is required",
        field: "region",
      }),
    );
    render(<NewRunScreen client={api} />);
    fireEvent.change(screen.getByLabelText("Seed", { exact: true }), {
      target: { value: "43" },
    });
    fireEvent.change(screen.getByLabelText("Region", { exact: true }), {
      target: { value: "   " },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create queued run" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Region is required",
    );
    expect(
      (screen.getByLabelText("Seed", { exact: true }) as HTMLInputElement)
        .value,
    ).toBe("43");
    expect(
      (screen.getByLabelText("Region", { exact: true }) as HTMLInputElement)
        .value,
    ).toBe("   ");
  });
  it("prevents duplicate creation while a request is pending and never auto-executes", async () => {
    const api = client();
    let finish!: (value: RunDto) => void;
    vi.mocked(api.createRun).mockImplementation(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    render(<NewRunScreen client={api} />);
    const form = screen
      .getByRole("button", { name: "Create queued run" })
      .closest("form")!;
    fireEvent.submit(form);
    fireEvent.submit(form);
    expect(api.createRun).toHaveBeenCalledTimes(1);
    await act(async () => {
      finish(snapshot("queued"));
    });
    expect(api.executeRun).not.toHaveBeenCalled();
    expect(window.location.hash).toBe("#/runs/run-test");
  });
  it("renders an empty history explicitly", async () => {
    render(<RunsScreen client={client()} />);
    expect(
      await screen.findByText(
        "No saved runs. Create one to prove the request path.",
      ),
    ).toBeTruthy();
  });
  it("hides unexpected error details and retries a failed history read", async () => {
    const api = client();
    vi.mocked(api.listRuns)
      .mockRejectedValueOnce(new Error("private-path-and-stack"))
      .mockResolvedValueOnce([snapshot("completed")]);
    render(<RunsScreen client={api} />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).not.toContain("private-path-and-stack");
    fireEvent.click(screen.getByRole("button", { name: "Refresh runs" }));
    expect(
      await screen.findByRole("link", { name: "View run-test" }),
    ).toBeTruthy();
  });
  it.each(["rejected", "failed"] as const)(
    "renders %s separately from completed results",
    async (status) => {
      const api = client();
      vi.mocked(api.getRun).mockResolvedValue(snapshot(status));
      render(<DetailScreen client={api} id="run-test" />);
      expect(
        await screen.findByRole("heading", {
          name:
            status === "rejected"
              ? "Validation rejected this run"
              : "Run failed",
        }),
      ).toBeTruthy();
      expect(screen.queryByText("Average score", { exact: true })).toBeNull();
    },
  );
});
