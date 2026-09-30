# Local Stack Proof

A local-only proof that one React/TypeScript UI and shared Rust application layer can run as a browser application and a Windows desktop application.

Start with the [execution roadmap](docs/ROADMAP.md).

## Local prerequisites

- Node.js 24.21+ and npm 11+
- Rust stable (the current workspace was validated with Rust 1.98.1)

The project uses standard npm workspaces. A version manager such as Volta, nvm, or the official Node installer is optional.

## Try the current web proof (M4)

The browser-to-Rust path was first proven in memory in M1. The current web host
stores runs in the project-owned local Postgres container. Start Docker and run:

```text
docker compose -f infra/compose.yaml up -d --wait
```

No `.env` is required: defaults target this fixture only. Optional local overrides
are documented in `.env.example`; non-project database URLs are rejected before listening.

Open two terminals at the repository root:

```powershell
npm run dev:api
```

```powershell
npm run dev:web
```

Then open <http://127.0.0.1:5173>. Use **New Run** to create a queued run, then
**Execute run** on its detail page. **Runs** reads persisted history; **Diagnostics**
identifies the active host/storage. Detail shows configuration, results or controlled
rejection, and lifecycle events; non-terminal runs refresh automatically.
Web history survives browser refresh and API restart. Stop the API with Ctrl+C for graceful shutdown.

Container lifecycle and isolated adapter tests are documented in [infra/README.md](infra/README.md).
The generated Rust/TypeScript contract and HTTP examples are in [docs/api-contract.md](docs/api-contract.md).
The repeatable component/database/browser checks are in [docs/local-testing.md](docs/local-testing.md).
Health reports the selected storage; it is a liveness response, not a database readiness probe.
A process interruption after STARTED can leave a run running: durable jobs/recovery are outside this proof.

## Try the desktop proof (M5)

Windows desktop builds also require the Microsoft C++ build tools, Windows SDK, and WebView2 runtime. No API server, Docker, or separate database service is required; SQLite is bundled.

From the repository root:

```powershell
npm run dev:desktop
```

This starts the desktop-only Vite server on port 5174 and opens Tauri. The shared
routed React UI uses an in-process custom protocol to reach the same Axum router
and Rust application service, without opening a backend server socket. Desktop and
web runs are separate; desktop history is saved to its per-user SQLite database
and survives process restart. See [desktop testing](docs/desktop-testing.md) for
the local data location and an isolated test-directory override.

For a production executable with embedded frontend assets:

```powershell
npm run build:desktop
```

Then launch `target/release/local-stack-proof-desktop.exe`. Rebuild after frontend
changes to embed the current UI. This executable does not require Vite or the web API.
The M2 transport/native baseline is verified in
[M2-04](docs/issues/M2-04-in-memory-desktop-vertical-slice.md). M5 verifies the current
shared UI with SQLite and an actual Windows package; browser/component tests alone
do not substitute for that native evidence. Public distribution/signing is excluded.
