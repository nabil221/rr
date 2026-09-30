export type Route =
  | { kind: "runs" | "new" | "diagnostics" | "missing" }
  | { kind: "detail"; id: string };
export function parseRoute(hash: string): Route {
  if (!hash || hash === "#/runs" || hash === "#/") return { kind: "runs" };
  if (hash === "#/new") return { kind: "new" };
  if (hash === "#/diagnostics") return { kind: "diagnostics" };
  const match = /^#\/runs\/([^/]+)$/.exec(hash);
  if (match) {
    try {
      return { kind: "detail", id: decodeURIComponent(match[1]) };
    } catch {
      return { kind: "missing" };
    }
  }
  return { kind: "missing" };
}
export const detailLink = (id: string) => `#/runs/${encodeURIComponent(id)}`;
