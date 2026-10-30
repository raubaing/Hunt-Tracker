import type { Session } from "./types";

const pad = (n: number) => String(n).padStart(2, "0");

/** Même règle que côté Rust : sans activité depuis 6 h, le prochain kill ouvre une nouvelle session */
export const NEW_SESSION_AFTER_HOURS = 6;

export function isStale(session: Session, now = new Date()): boolean {
  const last = session.lastActivity ?? session.games[session.games.length - 1]?.timestamp ?? session.startedAt;
  const t = new Date(last).getTime();
  return Number.isFinite(t) && now.getTime() - t > NEW_SESSION_AFTER_HOURS * 3600 * 1000;
}

/** "jeudi 30 octobre 2026 à 21:05" */
export function formatStart(iso: string): string {
  const d = new Date(iso);
  const date = d.toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
  return `${date} à ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
