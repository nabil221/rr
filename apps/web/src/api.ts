import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import type {
  CreateRunRequest,
  ErrorDto,
  HealthDto,
  RunDto,
} from "./generated/api-contract";
export type { CreateRunRequest, RunDto } from "./generated/api-contract";

export interface Diagnostics {
  host: "web-local" | "desktop-local";
  transport: "loopback-http" | "embedded-protocol";
  storage: string;
  status: string;
}
export interface RunClient {
  listRuns(signal?: AbortSignal): Promise<RunDto[]>;
  getRun(id: string, signal?: AbortSignal): Promise<RunDto>;
  createRun(
    configuration: CreateRunRequest,
    signal?: AbortSignal,
  ): Promise<RunDto>;
  executeRun(id: string, signal?: AbortSignal): Promise<RunDto>;
  diagnostics(signal?: AbortSignal): Promise<Diagnostics>;
}
export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly field: string | null;
  readonly requestId: string | null;
  constructor(
    status: number,
    error: ErrorDto | null,
    requestId: string | null = null,
  ) {
    super(error?.message ?? `Request failed (${status})`);
    this.name = "ApiError";
    this.status = status;
    this.code = error?.code ?? "request-failed";
    this.field = error?.field ?? null;
    this.requestId = requestId;
  }
}
export function localApiRoot(value = ""): string {
  if (!value) return "";
  const url = new URL(value);
  if (
    url.protocol !== "http:" ||
    !["127.0.0.1", "localhost", "[::1]"].includes(url.hostname) ||
    url.username ||
    url.password ||
    url.pathname !== "/" ||
    url.search ||
    url.hash
  )
    throw new Error(
      "API endpoint must be a credential-free loopback HTTP root",
    );
  return url.origin;
}
export function createClient(
  webEndpoint = import.meta.env.VITE_API_BASE_URL ?? "",
): RunClient {
  const native = isTauri();
  const root = native
    ? convertFileSrc("", "proof-api").replace(/\/$/, "")
    : localApiRoot(webEndpoint);
  async function request<T>(
    path: string,
    init?: RequestInit,
    signal?: AbortSignal,
  ): Promise<T> {
    const response = await fetch(
      `${root}${path}`,
      signal ? { ...init, signal } : init,
    );
    if (!response.ok) {
      const error = (await response
        .json()
        .catch(() => null)) as ErrorDto | null;
      throw new ApiError(
        response.status,
        error,
        response.headers.get("x-request-id"),
      );
    }
    return (await response.json()) as T;
  }
  return {
    listRuns: (signal) => request<RunDto[]>("/api/runs", undefined, signal),
    getRun: (id, signal) =>
      request<RunDto>(`/api/runs/${encodeURIComponent(id)}`, undefined, signal),
    createRun: (configuration, signal) =>
      request<RunDto>(
        "/api/runs",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(configuration),
        },
        signal,
      ),
    executeRun: (id, signal) =>
      request<RunDto>(
        `/api/runs/${encodeURIComponent(id)}/execute`,
        { method: "POST" },
        signal,
      ),
    diagnostics: async (signal) => {
      const health = await request<HealthDto>("/api/health", undefined, signal);
      const storage = ["postgres", "in-memory", "sqlite"].includes(
        health.storage,
      )
        ? health.storage
        : "unknown";
      return {
        host: native ? "desktop-local" : "web-local",
        transport: native ? "embedded-protocol" : "loopback-http",
        storage,
        status: health.status === "ok" ? "ok" : "unknown",
      };
    },
  };
}
export const listRuns = (signal?: AbortSignal) =>
  createClient().listRuns(signal);
export const createRun = (config: CreateRunRequest, signal?: AbortSignal) =>
  createClient().createRun(config, signal);
export const executeRun = (id: string, signal?: AbortSignal) =>
  createClient().executeRun(id, signal);
