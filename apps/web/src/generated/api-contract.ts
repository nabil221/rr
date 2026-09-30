// Generated from Rust HTTP DTOs. Run npm run contract:generate; do not edit.

export type CreateRunRequest = { seed: number, region: string, threshold: number, forceValidationFailure: boolean, };
export type ConfigurationDto = { seed: number, region: string, threshold: number, forceValidationFailure: boolean, };
export type StatusDto = "queued" | "running" | "completed" | "rejected" | "failed";
export type EventKindDto = "created" | "started" | "completed" | "rejected" | "failed";
export type GroupDto = { category: string, matchedRecords: number, totalScore: number, averageScore: number, };
export type ResultDto = { totalRecords: number, matchedRecords: number, totalScore: number, averageScore: number, groups: Array<GroupDto>, };
export type ValidationMessageDto = { code: string, message: string, field: string | null, };
export type EventDto = { kind: EventKindDto, message: string, };
export type RunDto = { id: string, configuration: ConfigurationDto, status: StatusDto, result: ResultDto | null, validationMessages: Array<ValidationMessageDto>, events: Array<EventDto>, };
export type HealthDto = { status: string, storage: string, };
export type ErrorDto = { code: string, message: string, field: string | null, };
