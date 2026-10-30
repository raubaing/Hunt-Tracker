import type { Session } from "./types";
import { saveSession, setActiveSession } from "./storage";

const pad = (n: number) => String(n).padStart(2, "0");

/** "Session du dd/MM/YY" */
export function sessionBaseName(d: Date): string {
  return `Session du ${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${pad(d.getFullYear() % 100)}`;
}

/** Date locale au format ISO sans fuseau : "2026-10-30T21:05:12" */
export function localIso(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

/** Identifiant unique, utilisé comme nom de fichier : "2026-10-30_210512" */
function sessionId(d: Date): string {
  return localIso(d).replace("T", "_").replace(/:/g, "");
}

/**
 * Crée la session du jour, la sauvegarde et en fait la session en cours.
 * S'il existe déjà une session ce jour-là, on suffixe : "Session du 30/10/26 (2)".
 */
export async function createSession(existing: Session[]): Promise<Session> {
  const now = new Date();
  const base = sessionBaseName(now);
  const sameDay = existing.filter((s) => s.name === base || s.name.startsWith(`${base} (`)).length;
  const session: Session = {
    id: sessionId(now),
    name: sameDay === 0 ? base : `${base} (${sameDay + 1})`,
    startedAt: localIso(now),
    games: [],
  };
  await saveSession(session);
  await setActiveSession(session.id);
  return session;
}

/** "jeudi 30 octobre 2026 à 21:05" */
export function formatStart(iso: string): string {
  const d = new Date(iso);
  const date = d.toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
  return `${date} à ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
