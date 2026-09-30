import type { RunClient } from "./api";
export function RunsScreen({ client: _client }: { client: RunClient }) {
  return (
    <section className="panel screen">
      <h2>Runs</h2>
      <p role="status">Run history will load here.</p>
      <a href="#/new">Create a new run</a>
    </section>
  );
}
export function NewRunScreen({ client: _client }: { client: RunClient }) {
  return (
    <section className="panel screen">
      <h2>New Run</h2>
      <p>Configure a queued run before executing it.</p>
    </section>
  );
}
export function DetailScreen({
  client: _client,
  id,
}: {
  client: RunClient;
  id: string;
}) {
  return (
    <section className="panel screen">
      <h2>Run detail</h2>
      <code>{id}</code>
      <p>Configuration, execution, and lifecycle history appear here.</p>
    </section>
  );
}
export function DiagnosticsScreen({ client: _client }: { client: RunClient }) {
  return (
    <section className="panel screen">
      <h2>Diagnostics</h2>
      <p>Safe local host and storage information appears here.</p>
    </section>
  );
}
