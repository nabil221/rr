import { useEffect, useState } from "react";
import type { Diagnostics, RunClient } from "./api";
import { ErrorNotice, Loading } from "./ui";

export function DiagnosticsScreen({ client }: { client: RunClient }) {
  const [info, setInfo] = useState<Diagnostics | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    const controller = new AbortController();
    void client
      .diagnostics(controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) setInfo(value);
      })
      .catch((failure: unknown) => {
        if (!controller.signal.aborted) setError(failure);
      });
    return () => controller.abort();
  }, [client, revision]);
  return (
    <section className="panel screen">
      <div className="panel-heading">
        <h2>Diagnostics</h2>
        <button
          className="secondary-button"
          onClick={() => {
            setInfo(null);
            setError(null);
            setRevision((value) => value + 1);
          }}
        >
          Refresh diagnostics
        </button>
      </div>
      <ErrorNotice error={error} />
      {!info && !error && <Loading />}
      {info && (
        <dl className="metadata">
          <dt>Host</dt>
          <dd>{info.host}</dd>
          <dt>Transport</dt>
          <dd>{info.transport}</dd>
          <dt>Storage</dt>
          <dd>{info.storage}</dd>
          <dt>Host health</dt>
          <dd>{info.status}</dd>
        </dl>
      )}
      <p>
        Health confirms host liveness, not database readiness. Stores are
        independent; web and desktop do not synchronize.
      </p>
      <p>
        Desktop persists locally in SQLite; web persists in its local Postgres
        store. No connection strings, credentials, filesystem paths, or stack
        traces are displayed.
      </p>
    </section>
  );
}
