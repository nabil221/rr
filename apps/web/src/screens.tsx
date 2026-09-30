import { useEffect, useRef, useState, type FormEvent } from "react";
import { ApiError, type RunClient, type RunDto } from "./api";
import { detailLink } from "./routes";
import { ErrorNotice, Loading } from "./ui";
export function RunsScreen({ client }: { client: RunClient }) {
  const [runs, setRuns] = useState<RunDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    const controller = new AbortController();
    void client
      .listRuns(controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) setRuns(value);
      })
      .catch((failure: unknown) => {
        if (!controller.signal.aborted) setError(failure);
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [client, revision]);
  return (
    <section className="panel screen">
      <div className="panel-heading">
        <h2>Runs</h2>
        <button
          className="secondary-button"
          onClick={() => {
            setLoading(true);
            setError(null);
            setRevision((value) => value + 1);
          }}
          disabled={loading}
        >
          Refresh runs
        </button>
      </div>
      <ErrorNotice error={error} />
      {loading && <Loading />}
      {!loading && !error && runs.length === 0 && (
        <p>No saved runs. Create one to prove the request path.</p>
      )}
      <ul className="run-list" aria-label="Saved runs" aria-busy={loading}>
        {runs.map((run) => (
          <li key={run.id}>
            <a
              className="run-row"
              href={detailLink(run.id)}
              aria-label={`View ${run.id}`}
            >
              <span className="run-id">{run.id}</span>
              <span className={`status-badge status-${run.status}`}>
                {run.status}
              </span>
              <span className="run-region">{run.configuration.region}</span>
            </a>
          </li>
        ))}
      </ul>
      <p className="screen-action">
        <a href="#/new">Create a new run</a>
      </p>
    </section>
  );
}
export function NewRunScreen({ client }: { client: RunClient }) {
  const [seed, setSeed] = useState("42");
  const [region, setRegion] = useState("north");
  const [threshold, setThreshold] = useState("0");
  const [forced, setForced] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const request = useRef<AbortController | null>(null);
  useEffect(() => () => request.current?.abort(), []);
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (request.current && !request.current.signal.aborted) return;
    const controller = new AbortController();
    request.current = controller;
    setBusy(true);
    setError(null);
    try {
      const run = await client.createRun(
        {
          seed: Number(seed),
          region,
          threshold: Number(threshold),
          forceValidationFailure: forced,
        },
        controller.signal,
      );
      if (!controller.signal.aborted) window.location.hash = detailLink(run.id);
    } catch (failure) {
      if (!controller.signal.aborted) setError(failure);
    } finally {
      if (!controller.signal.aborted) {
        setBusy(false);
        request.current = null;
      }
    }
  }
  const invalid = (field: string) =>
    error instanceof ApiError && error.field === field;
  return (
    <section className="panel screen">
      <h2>New Run</h2>
      <p>
        Create a queued run. Execution is a separate action on its detail page.
      </p>
      <ErrorNotice error={error} />
      <form className="run-form" onSubmit={submit} aria-busy={busy}>
        <label>
          Seed
          <input
            type="number"
            min="0"
            max="9007199254740991"
            step="1"
            required
            value={seed}
            onChange={(event) => setSeed(event.target.value)}
            aria-invalid={invalid("seed")}
          />
        </label>
        <label>
          Region
          <input
            type="text"
            required
            value={region}
            onChange={(event) => setRegion(event.target.value)}
            aria-invalid={invalid("region")}
          />
        </label>
        <label>
          Minimum average score
          <input
            type="number"
            min="0"
            max="100"
            step="any"
            required
            value={threshold}
            onChange={(event) => setThreshold(event.target.value)}
            aria-invalid={invalid("threshold")}
          />
        </label>
        <label className="check-row">
          <input
            type="checkbox"
            checked={forced}
            onChange={(event) => setForced(event.target.checked)}
          />
          <span>Force a controlled validation rejection</span>
        </label>
        <button className="primary-button" type="submit" disabled={busy}>
          {busy ? "Creating…" : "Create queued run"}
        </button>
      </form>
    </section>
  );
}
export { DetailScreen } from "./detail";
export { DiagnosticsScreen } from "./diagnostics";
