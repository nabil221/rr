import { useEffect, useState } from "react";
import { createClient, type RunClient } from "./api";
import { parseRoute, type Route } from "./routes";
import {
  RunsScreen,
  NewRunScreen,
  DetailScreen,
  DiagnosticsScreen,
} from "./screens";

export function App({
  client: suppliedClient,
  initialRoute,
}: {
  client?: RunClient;
  initialRoute?: Route;
}) {
  const [client] = useState(() => {
    try {
      return suppliedClient ?? createClient();
    } catch {
      return null;
    }
  });
  const [route, setRoute] = useState<Route>(
    () => initialRoute ?? parseRoute(window.location.hash),
  );
  useEffect(() => {
    const navigate = () => setRoute(parseRoute(window.location.hash));
    window.addEventListener("hashchange", navigate);
    return () => window.removeEventListener("hashchange", navigate);
  }, []);
  if (!client)
    return (
      <main className="page-shell">
        <h1>Local configuration needs attention</h1>
        <p role="alert">
          Check the API endpoint setting. Only credential-free loopback HTTP
          roots are accepted.
        </p>
      </main>
    );
  return (
    <main className="page-shell">
      <header className="page-header">
        <p className="eyebrow">LOCAL STACK PROOF · WEB + DESKTOP</p>
        <h1>One UI. Two local hosts.</h1>
        <p className="lede">
          Create, execute, and inspect a deterministic Rust run. No remote
          service or account.
        </p>
      </header>
      <nav className="main-nav" aria-label="Main navigation">
        <a
          href="#/runs"
          aria-current={route.kind === "runs" ? "page" : undefined}
        >
          Runs
        </a>
        <a
          href="#/new"
          aria-current={route.kind === "new" ? "page" : undefined}
        >
          New Run
        </a>
        <a
          href="#/diagnostics"
          aria-current={route.kind === "diagnostics" ? "page" : undefined}
        >
          Diagnostics
        </a>
      </nav>
      {route.kind === "runs" && <RunsScreen client={client} />}
      {route.kind === "new" && <NewRunScreen client={client} />}
      {route.kind === "detail" && (
        <DetailScreen key={route.id} client={client} id={route.id} />
      )}
      {route.kind === "diagnostics" && <DiagnosticsScreen client={client} />}
      {route.kind === "missing" && (
        <section className="panel screen">
          <h2>Page not found</h2>
          <a href="#/runs">Return to Runs</a>
        </section>
      )}
      <footer className="page-footer">
        Shared React client → local Rust host → independent local storage
      </footer>
    </main>
  );
}
