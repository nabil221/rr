import { useCallback, useEffect, useState, type FormEvent } from "react";
import {
  createRun,
  executeRun,
  listRuns,
  type CreateRunRequest,
  type RunDto,
} from "./api";

export function App() {
  const [seed, setSeed] = useState("42");
  const [region, setRegion] = useState("north");
  const [threshold, setThreshold] = useState("0");
  const [forceValidationFailure, setForceValidationFailure] = useState(false);
  const [runs, setRuns] = useState<RunDto[]>([]);
  const [selectedRun, setSelectedRun] = useState<RunDto | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refreshRuns = useCallback(async () => {
    try {
      const currentRuns = await listRuns();
      setRuns(currentRuns);
      setError(null);
    } catch (requestError) {
      setError(messageFrom(requestError));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    void listRuns()
      .then((currentRuns) => {
        if (active) {
          setRuns(currentRuns);
          setError(null);
        }
      })
      .catch((requestError: unknown) => {
        if (active) setError(messageFrom(requestError));
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });

    return () => {
      active = false;
    };
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setError(null);

    const request: CreateRunRequest = {
      seed: Number(seed),
      region,
      threshold: Number(threshold),
      forceValidationFailure,
    };

    try {
      const createdRun = await createRun(request);
      setSelectedRun(createdRun);
      const finishedRun = await executeRun(createdRun.id);
      setSelectedRun(finishedRun);
      await refreshRuns();
    } catch (requestError) {
      setError(messageFrom(requestError));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="page-shell">
      <header className="page-header">
        <p className="eyebrow">LOCAL STACK PROOF · WEB + DESKTOP</p>
        <h1>Run the in-memory proof</h1>
        <p className="lede">
          A small frontend-to-Rust round trip. No database, account, or external
          service is involved.
        </p>
      </header>

      <div className="proof-note" role="note">
        <strong>Temporary storage.</strong> Runs live in the Rust host's memory.
        Restarting the API or desktop app clears its history; refreshing this
        screen does not.
      </div>

      <section className="workspace" aria-label="Run workspace">
        <form className="panel run-form" onSubmit={handleSubmit}>
          <div className="panel-heading">
            <div>
              <p className="eyebrow">INPUT</p>
              <h2>Demo configuration</h2>
            </div>
            <span className="step-number">01</span>
          </div>

          <label>
            Seed
            <input
              type="number"
              min="0"
              step="1"
              value={seed}
              onChange={(event) => setSeed(event.target.value)}
              required
            />
          </label>

          <label>
            Region
            <select
              value={region}
              onChange={(event) => setRegion(event.target.value)}
            >
              <option value="north">North</option>
              <option value="south">South</option>
              <option value="west">West</option>
              <option value="east">East (no matching records)</option>
            </select>
          </label>

          <label>
            Minimum average score
            <input
              type="number"
              min="0"
              max="100"
              step="0.1"
              value={threshold}
              onChange={(event) => setThreshold(event.target.value)}
              required
            />
          </label>

          <label className="check-row">
            <input
              type="checkbox"
              checked={forceValidationFailure}
              onChange={(event) =>
                setForceValidationFailure(event.target.checked)
              }
            />
            <span>Force a controlled validation rejection</span>
          </label>

          <button
            className="primary-button"
            type="submit"
            disabled={isSubmitting}
          >
            {isSubmitting ? "Running…" : "Create and execute run"}
          </button>
          <p className="form-footnote">
            The UI creates the run, then asks the Rust host to execute it.
          </p>
        </form>

        <section
          className="panel result-panel"
          aria-labelledby="result-heading"
        >
          <div className="panel-heading">
            <div>
              <p className="eyebrow">OUTPUT</p>
              <h2 id="result-heading">Execution result</h2>
            </div>
            <span className="step-number">02</span>
          </div>

          {error && (
            <div className="error-box" role="alert">
              <strong>Could not complete the request</strong>
              <span>{error}</span>
            </div>
          )}

          {selectedRun ? (
            <RunResultView run={selectedRun} />
          ) : (
            <div className="empty-state">
              <span className="empty-icon" aria-hidden="true">
                ↗
              </span>
              <p>
                {isLoading
                  ? "Connecting to the Rust host…"
                  : "Your next run will appear here."}
              </p>
              <span>
                Submit the configuration to exercise the full request path.
              </span>
            </div>
          )}
        </section>
      </section>

      <section
        className="panel history-panel"
        aria-labelledby="history-heading"
      >
        <div className="panel-heading history-heading">
          <div>
            <p className="eyebrow">API READ-BACK</p>
            <h2 id="history-heading">Runs in this host session</h2>
          </div>
          <span className="count-pill">
            {runs.length} {runs.length === 1 ? "run" : "runs"}
          </span>
        </div>

        {runs.length === 0 ? (
          <p className="history-empty">
            {isLoading
              ? "Loading run history…"
              : "No runs in memory. Create one above."}
          </p>
        ) : (
          <ul className="run-list">
            {runs.map((run) => (
              <li key={run.id}>
                <button
                  className="run-row"
                  type="button"
                  onClick={() => setSelectedRun(run)}
                  aria-label={`Show ${run.id}, ${run.status}`}
                >
                  <span className="run-id">{run.id}</span>
                  <span className={`status-badge status-${run.status}`}>
                    {run.status}
                  </span>
                  <span className="run-region">{run.configuration.region}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <footer className="page-footer">
        <span>React UI</span>
        <span className="footer-connector" aria-hidden="true" />
        <span>Shared Axum router</span>
        <span className="footer-connector" aria-hidden="true" />
        <span>Application service</span>
        <span className="footer-connector" aria-hidden="true" />
        <span>In-memory repository</span>
      </footer>
    </main>
  );
}

function RunResultView({ run }: { run: RunDto }) {
  return (
    <div className="run-result" aria-live="polite">
      <div className="result-summary">
        <div>
          <span className="summary-label">Run ID</span>
          <code>{run.id}</code>
        </div>
        <span className={`status-badge status-${run.status}`}>
          {run.status}
        </span>
      </div>

      {run.result ? (
        <>
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
        </>
      ) : (
        <div className="rejection-box">
          <h3>Validation rejected this run</h3>
          {run.validationMessages.map((message) => (
            <p key={message.code}>
              <strong>{message.field ? `${message.field}: ` : ""}</strong>
              {message.message}
            </p>
          ))}
        </div>
      )}

      <details className="event-details">
        <summary>Lifecycle events ({run.events.length})</summary>
        <ol>
          {run.events.map((event, index) => (
            <li key={`${event.kind}-${index}`}>
              <span>{event.kind}</span>
              <span>{event.message}</span>
            </li>
          ))}
        </ol>
      </details>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function messageFrom(error: unknown) {
  return error instanceof Error
    ? error.message
    : "An unexpected request error occurred.";
}
