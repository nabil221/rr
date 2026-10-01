# ADR-001: Desktop Transport

**Status:** accepted after M2-02 development and production spike  
**Owner issues:** M2-01, M2-02; M2-03 only if the preferred path fails

## Context

M1 proves React -> Axum -> shared Rust application services using process-local storage. M2 must prove the same flow in a Windows desktop executable, with no API process, database, or listening server socket. The transport must remain behind the client boundary so React screens and calculation logic are reused.

## Options considered

1. Preferred: register an asynchronous Tauri URI-scheme handler before WebView creation and dispatch its requests directly into the shared Axum router using `tower::ServiceExt::oneshot`. Windows WebView2 represents the custom scheme as `http://proof-api.localhost`; this is intercepted in-process and does not bind a TCP listener.
2. Fallback: narrow Tauri commands forwarding to the same application services, with a client adapter preserving DTO and error semantics.
3. A desktop loopback HTTP server is not selected: it introduces socket ownership and a reachable endpoint that are unnecessary for this proof.

## Decision and spike plan

Evaluate option 1 first. Extract the existing M1 HTTP router into a small `transport-http` crate without changing the route contract. Compose that router and the in-memory service in the Tauri host. First use a standalone health/create/error harness; only after that passes in development and a production executable, connect the existing React screen.

Use current stable Tauri 2 releases, not Tauri 3 prereleases. Build the production Windows executable with embedded assets and `--no-bundle`; installer/signing/distribution checks belong to M5. The spike uses a maximum of two implementation hypotheses for any custom-protocol failure. Environmental failures such as a missing compiler are reported separately and do not justify rejecting the transport.

## Pass/fail criteria

| Criterion                          | Required evidence                                                                                                                                                                    |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Windows development and production | Health and create requests succeed inside WebView2 in both development and the production executable with bundled assets.                                                            |
| Shared core                        | Router handlers call the existing `RunService`; desktop contains no separate calculation or lifecycle implementation.                                                                |
| No server dependency               | Production executable works with API and Vite stopped; no listening sockets belong to the desktop process. Postgres is unused.                                                       |
| Typed errors                       | Invalid configuration and missing IDs retain HTTP status and JSON code/message/field; the client retains these details.                                                              |
| Origin boundary                    | Only the configured desktop development origin and packaged Tauri origin are accepted; preflight succeeds for those origins and others are rejected. No wildcard origin policy.      |
| Cancellation                       | The client can forward an `AbortSignal` to fetch. This cancels waiting, not a completed synchronous domain operation; business cancellation is outside M2.                           |
| Progress                           | Created/started/terminal lifecycle evidence can be read through the same result/status routes. The tiny synchronous computation does not require live streaming or a new job system. |
| Diagnostics                        | Request method/path/status/timing are logged by the host; no input payload, credential, or external log service is introduced.                                                       |
| Testability                        | Router and custom-protocol adapter have process-free Rust tests; frontend transport selection has targeted tests.                                                                    |
| UI reuse                           | Only the client URL resolver knows about Tauri. M2-04 verifies the same React screen in both hosts after the health/create spike passes.                                             |

## Evidence

- M2-01 reviewed the spike criteria before implementation. M1-04 is complete with a Chrome UI walkthrough.
- Official Tauri documentation describes asynchronous custom-protocol registration and the Windows `http://<scheme>.localhost` mapping: [Builder API](https://docs.rs/tauri/2.12.0/tauri/struct.Builder.html#method.register_asynchronous_uri_scheme_protocol).
- Registry checks resolved stable Tauri 2.12.0, tauri-build 2.7.0, and npm CLI/API 2.12.0. Tauri 3.0.0-alpha releases are excluded.
- M2-02 passed in development and the production executable with embedded assets: health, create, invalid-input and missing-ID response bodies were asserted by WebView JavaScript. Both reached the final success checkpoint. Production ran with API/Vite/Postgres stopped and no desktop-owned listening TCP socket. See M2-02 for local log evidence.
- Router and protocol tests preserve lifecycle/status/typed-error semantics and the exact-origin boundary; client tests verify native URL selection and AbortSignal forwarding. Full shared React UI verification remains M2-04.
- M2-04 subsequently passed the production native React success/rejection/history/reload/restart walkthrough with API/Vite/Postgres stopped. The same screen and shared router/service are verified end to end in both hosts.

## Consequences

The web host and desktop host share both application services and HTTP routing/DTO logic. Desktop fetch still follows browser CORS and CSP rules even though requests never leave the WebView process. Development uses its own Vite port; production embeds the frontend assets. Desktop state remains independent of web state and is lost on process restart until SQLite is introduced.

## Fallback

If custom-protocol registration, request bodies, responses, CORS, or production behavior fails the bounded spike, record the specific reproducible failure here and execute M2-03. Keep the domain/application crates and React screen unchanged; replace only desktop transport composition and the client adapter. Do not retain a failed embedded-router path beside the fallback.
