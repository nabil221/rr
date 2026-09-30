import { ApiError } from "./api";
export function ErrorNotice({ error }: { error: unknown }) {
  if (!error) return null;
  return (
    <div className="error-box" role="alert">
      <strong>Request could not be completed</strong>
      <p>
        {error instanceof ApiError
          ? error.message
          : "The local host could not be reached. Check that it is running and retry."}
      </p>
      {error instanceof ApiError && (
        <small>
          Code: {error.code}
          {error.requestId ? ` · Request: ${error.requestId}` : ""}
        </small>
      )}
    </div>
  );
}
export function Loading() {
  return <p role="status">Loading from the local host…</p>;
}
