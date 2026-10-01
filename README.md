# Local Stack Proof

A local-only proof that one React/TypeScript UI and shared Rust application layer can run as a browser application and a Windows desktop application.

Start with the [execution roadmap](docs/ROADMAP.md).

## Local prerequisites

- Node.js 24.21+ and npm 11+
- Rust stable (the current workspace was validated with Rust 1.98.1)

The project uses standard npm workspaces. A version manager such as Volta, nvm, or the official Node installer is optional.

## Try the current web proof (M1)

This milestone deliberately uses in-memory storage so the browser-to-Rust-API path is proven before database setup. No Docker, Postgres, `.env`, or network service is needed.

Open two terminals at the repository root:

```powershell
npm run dev:api
```

```powershell
npm run dev:web
```

Then open <http://127.0.0.1:5173>. Create a demo run to see the UI send a request through Vite's same-origin `/api` proxy to the local Axum API, execute deterministic Rust domain logic, and render the result and lifecycle events. The app also has a deliberate validation-rejection case. The run list survives a browser refresh, but is cleared when the API process restarts because persistence is a later milestone.

The project-scoped Postgres container is reserved for the later persistence milestone; its setup remains documented in [infra/README.md](infra/README.md).

## Try the desktop proof (M2)

Windows desktop builds also require the Microsoft C++ build tools, Windows SDK, and WebView2 runtime. No API server, Docker, or database is required.

From the repository root:

```powershell
npm run dev:desktop
```

This starts the desktop-only Vite server on port 5174 and opens Tauri. The same React form uses an in-process custom protocol to reach the same Axum router and Rust application service, without opening a backend server socket. Desktop and web runs are separate; desktop history disappears when the desktop process exits.

For a production executable with embedded frontend assets:

```powershell
npm run build:desktop
```

Then launch `target/release/local-stack-proof-desktop.exe`. This executable does not require Vite or the web API. Installer/signing work is deferred to M5. The transport spike and native React success/rejection/history/restart walkthrough are verified; evidence is recorded in [M2-04](docs/issues/M2-04-in-memory-desktop-vertical-slice.md).
