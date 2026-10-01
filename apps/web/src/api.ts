import { convertFileSrc, isTauri } from "@tauri-apps/api/core";

export interface CreateRunRequest {
  seed: number;
  region: string;
  threshold: number;
  forceValidationFailure: boolean;
}

export interface RunDto {
  id: string;
  configuration: CreateRunRequest;
  status: "queued" | "running" | "completed" | "rejected" | "failed";
  result: RunResultDto | null;
  validationMessages: ValidationMessageDto[];
  events: RunEventDto[];
}

export interface RunResultDto {
  totalRecords: number;
  matchedRecords: number;
  totalScore: number;
  averageScore: number;
  groups: GroupDto[];
}

export interface GroupDto {
  category: string;
  matchedRecords: number;
  totalScore: number;
  averageScore: number;
}

export interface ValidationMessageDto {
  code: string;
  message: string;
  field: string | null;
}

export interface RunEventDto {
  kind: string;
  message: string;
}

interface ApiErrorDto {
  code: string;
  message: string;
  field: string | null;
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly field: string | null;

  constructor(status: number, error: ApiErrorDto | null) {
    super(error?.message ?? `Request failed (${status})`);
    this.name = "ApiError";
    this.status = status;
    this.code = error?.code ?? "request-failed";
    this.field = error?.field ?? null;
  }
}

export function listRuns(signal?: AbortSignal) {
  return request<RunDto[]>("/api/runs", undefined, signal);
}

export function createRun(
  configuration: CreateRunRequest,
  signal?: AbortSignal,
) {
  return request<RunDto>(
    "/api/runs",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(configuration),
    },
    signal,
  );
}

export function executeRun(id: string, signal?: AbortSignal) {
  return request<RunDto>(
    `/api/runs/${encodeURIComponent(id)}/execute`,
    {
      method: "POST",
    },
    signal,
  );
}

async function request<T>(
  path: string,
  init?: RequestInit,
  signal?: AbortSignal,
): Promise<T> {
  // Convert only the protocol root: converting an API path would encode its slashes.
  const url = isTauri()
    ? `${convertFileSrc("", "proof-api").replace(/\/$/, "")}${path}`
    : path;
  const response = await fetch(url, signal ? { ...init, signal } : init);
  if (!response.ok) {
    const error = (await response
      .json()
      .catch(() => null)) as ApiErrorDto | null;
    throw new ApiError(response.status, error);
  }
  return (await response.json()) as T;
}
