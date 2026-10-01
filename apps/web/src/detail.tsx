import { useCallback, useEffect, useRef, useState } from "react";
import type { RunClient, RunDto } from "./api";
import { ErrorNotice, Loading } from "./ui";

const isTerminal = (run: RunDto) =>
  ["completed", "rejected", "failed"].includes(run.status);

export function DetailScreen({
  client,
  id,
}: {
  client: RunClient;
  id: string;
}) {
  const [run, setRun] = useState<RunDto | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [revision, setRevision] = useState(0);
  const latest = useRef<RunDto | null>(null);
  const mutation = useRef<AbortController | null>(null);
  const accept = useCallback((candidate: RunDto) => {
    // A delayed queued/running read must not overwrite a newer execution response.
    if (
      !latest.current ||
      candidate.events.length >= latest.current.events.length
    ) {
      latest.current = candidate;
      setRun(candidate);
    }
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    latest.current = null;
    async function read() {
      if (
        controller.signal.aborted ||
        (latest.current && isTerminal(latest.current))
      )
        return;
      try {
        const candidate = await client.getRun(id, controller.signal);
        if (controller.signal.aborted) return;
        accept(candidate);
        if (latest.current && !isTerminal(latest.current))
          timer = setTimeout(() => {
            void read();
          }, 1000);
      } catch (failure) {
        if (!controller.signal.aborted) setError(failure);
      }
    }
    void read();
    return () => {
      controller.abort();
      clearTimeout(timer);
    };
  }, [client, id, revision, accept]);
  useEffect(() => () => mutation.current?.abort(), []);
  async function execute() {
    if (!run || run.status !== "queued" || mutation.current) return;
    const controller = new AbortController();
    mutation.current = controller;
    setBusy(true);
    setError(null);
    try {
      const result = await client.executeRun(id, controller.signal);
      if (!controller.signal.aborted) accept(result);
    } catch (failure) {
      if (!controller.signal.aborted) setError(failure);
    } finally {
      if (!controller.signal.aborted) {
        mutation.current = null;
        setBusy(false);
      }
    }
  }
  return (
    <section className="panel screen">
      <div className="panel-heading">
        <h2>Run detail</h2>
        <button
          className="secondary-button"
          disabled={busy}
          onClick={() => {
            setError(null);
            setRevision((value) => value + 1);
          }}
        >
          Reload detail
        </button>
      </div>
      <code>{id}</code>
      <ErrorNotice error={error} />
      {!run && !error && <Loading />}
      {run && (
        <>
          <div className="detail-status">
            <span className={`status-badge status-${run.status}`} role="status">
              {run.status}
            </span>
          </div>
          <h3>Configuration snapshot</h3>
          <dl className="metadata">
            <dt>Seed</dt>
            <dd>{run.configuration.seed}</dd>
            <dt>Region</dt>
            <dd>{run.configuration.region}</dd>
            <dt>Minimum average score</dt>
            <dd>{run.configuration.threshold}</dd>
            <dt>Forced rejection</dt>
            <dd>{run.configuration.forceValidationFailure ? "yes" : "no"}</dd>
          </dl>
          {run.status === "queued" && (
            <button
              className="primary-button execute-button"
              disabled={busy || !!error}
              onClick={() => {
                void execute();
              }}
            >
              {busy ? "Executing…" : "Execute run"}
            </button>
          )}
          {run.status === "running" && (
            <p>
              Run is running. Checking its saved status automatically.
              Interrupted jobs are not recovered automatically.
            </p>
          )}
          {run.status === "completed" && run.result && (
            <div className="run-result">
              <h3>Execution result</h3>
              <div className="metrics-grid">
                <Metric label="Generated" value={run.result.totalRecords} />
                <Metric label="Matched" value={run.result.matchedRecords} />
                <Metric
                  label="Average score"
                  value={run.result.averageScore.toFixed(2)}
                />
              </div>
              <h3>Category breakdown</h3>
              <div className="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th>Category</th>
                      <th>Records</th>
                      <th>Avg. score</th>
                    </tr>
                  </thead>
                  <tbody>
                    {run.result.groups.map((group) => (
                      <tr key={group.category}>
                        <td>{group.category}</td>
                        <td>{group.matchedRecords}</td>
                        <td>{group.averageScore.toFixed(2)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
          {run.status === "rejected" && (
            <div className="rejection-box">
              <h3>Validation rejected this run</h3>
              {run.validationMessages.map((message, index) => (
                <p key={`${message.code}-${index}`}>
                  {message.field ? `${message.field}: ` : ""}
                  {message.message}
                </p>
              ))}
            </div>
          )}
          {run.status === "failed" && (
            <div className="rejection-box">
              <h3>Run failed</h3>
              <p>
                Inspect the saved lifecycle events. This is not a validation
                rejection or a zero-valued result.
              </p>
            </div>
          )}
          <section className="event-details" aria-label="Lifecycle events">
            <h3>Lifecycle events ({run.events.length})</h3>
            <ol>
              {run.events.map((event, index) => (
                <li key={`${event.kind}-${index}`}>
                  <span>{event.kind}</span>
                  <span>{event.message}</span>
                </li>
              ))}
            </ol>
          </section>
        </>
      )}
      <p className="screen-action">
        <a href="#/runs">Back to Runs</a>
      </p>
    </section>
  );
}
function Metric({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
