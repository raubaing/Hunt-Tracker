import type { Game, Session } from "./types";

export interface Totals {
  kills: number;
  deaths: number;
  assists: number;
  games: number;
}

/** Totaux de la session : games validées + game en cours */
export function sessionTotals(session: Session): Totals {
  const t = session.games.reduce(
    (t, g) => ({ kills: t.kills + g.kills, deaths: t.deaths + g.deaths, assists: t.assists + g.assists }),
    { ...session.current },
  );
  return { ...t, games: session.games.length };
}

/** KDA = (K + A) / D ; si 0 mort, KDA = K + A */
export function kda(t: { kills: number; deaths: number; assists: number }): number {
  return (t.kills + t.assists) / Math.max(t.deaths, 1);
}

export const formatKda = (value: number) => value.toFixed(2);

// ---------- Statistiques globales (onglet Stats) ----------

export type Period = "7d" | "30d" | "all";

export interface StatsFilter {
  period: Period;
  teamSize: number | null; // null = tous
  map: string | null; // null = toutes
}

/** Une game avec la session à laquelle elle appartient */
export interface GameRef {
  game: Game;
  session: Session;
}

const DAY = 24 * 3600 * 1000;

/** Games correspondant aux filtres, dans l'ordre chronologique */
export function filterGames(sessions: Session[], f: StatsFilter, now = new Date()): GameRef[] {
  const minTime = f.period === "all" ? -Infinity : now.getTime() - (f.period === "7d" ? 7 : 30) * DAY;
  const refs: GameRef[] = [];
  for (const session of sessions) {
    if (new Date(session.startedAt).getTime() < minTime) continue;
    for (const game of session.games) {
      if (f.teamSize !== null && game.teamSize !== f.teamSize) continue;
      if (f.map !== null && game.map !== f.map) continue;
      refs.push({ game, session });
    }
  }
  return refs.sort((a, b) => a.game.timestamp.localeCompare(b.game.timestamp));
}

export interface Summary {
  games: number;
  kills: number;
  deaths: number;
  assists: number;
  kda: number;
  kd: number;
  killsPerGame: number;
  /** extraction / bounty : sur les games où l'info est renseignée */
  extracted: number;
  extractedKnown: number;
  bounty: number;
  bountyKnown: number;
}

export function summarize(refs: GameRef[]): Summary {
  const s = { games: refs.length, kills: 0, deaths: 0, assists: 0, extracted: 0, extractedKnown: 0, bounty: 0, bountyKnown: 0 };
  for (const { game: g } of refs) {
    s.kills += g.kills;
    s.deaths += g.deaths;
    s.assists += g.assists;
    if (g.extracted != null) {
      s.extractedKnown++;
      if (g.extracted) s.extracted++;
    }
    if (g.bounty != null) {
      s.bountyKnown++;
      if (g.bounty) s.bounty++;
    }
  }
  return {
    ...s,
    kda: kda(s),
    kd: s.kills / Math.max(s.deaths, 1),
    killsPerGame: s.games ? s.kills / s.games : 0,
  };
}

/** KDA de chaque session (games filtrées), dans l'ordre chronologique */
export function kdaBySession(refs: GameRef[]) {
  const bySession = new Map<string, { session: Session; games: Game[] }>();
  for (const r of refs) {
    const entry = bySession.get(r.session.id) ?? { session: r.session, games: [] };
    entry.games.push(r.game);
    bySession.set(r.session.id, entry);
  }
  return [...bySession.values()]
    .sort((a, b) => a.session.startedAt.localeCompare(b.session.startedAt))
    .map(({ session, games }) => {
      const t = games.reduce(
        (t, g) => ({ kills: t.kills + g.kills, deaths: t.deaths + g.deaths, assists: t.assists + g.assists }),
        { kills: 0, deaths: 0, assists: 0 },
      );
      return { session, games: games.length, ...t, kda: kda(t) };
    });
}

/** KDA par map (games dont la map est renseignée) */
export function kdaByMap(refs: GameRef[]) {
  const byMap = new Map<string, { kills: number; deaths: number; assists: number; games: number }>();
  for (const { game: g } of refs) {
    if (!g.map) continue;
    const t = byMap.get(g.map) ?? { kills: 0, deaths: 0, assists: 0, games: 0 };
    t.kills += g.kills;
    t.deaths += g.deaths;
    t.assists += g.assists;
    t.games++;
    byMap.set(g.map, t);
  }
  return [...byMap.entries()].map(([map, t]) => ({ map, ...t, kda: kda(t) })).sort((a, b) => b.kda - a.kda);
}

export interface Records {
  bestGame: GameRef | null; // plus de kills (à égalité : meilleur KDA)
  bestSession: { session: Session; kda: number; games: number } | null; // au moins 3 games
  mostKillsSession: { session: Session; kills: number } | null;
  longestNoDeathStreak: number; // games d'affilée sans mourir
}

export function records(refs: GameRef[]): Records {
  let bestGame: GameRef | null = null;
  let streak = 0;
  let longest = 0;
  for (const r of refs) {
    const g = r.game;
    const better =
      !bestGame || g.kills > bestGame.game.kills || (g.kills === bestGame.game.kills && kda(g) > kda(bestGame.game));
    if (g.kills > 0 && better) bestGame = r;
    streak = g.deaths === 0 ? streak + 1 : 0;
    longest = Math.max(longest, streak);
  }
  const sessions = kdaBySession(refs);
  const eligible = sessions.filter((s) => s.games >= 3);
  const bestSession = eligible.length ? eligible.reduce((a, b) => (b.kda > a.kda ? b : a)) : null;
  const mostKills = sessions.length ? sessions.reduce((a, b) => (b.kills > a.kills ? b : a)) : null;
  return {
    bestGame,
    bestSession: bestSession ? { session: bestSession.session, kda: bestSession.kda, games: bestSession.games } : null,
    mostKillsSession: mostKills && mostKills.kills > 0 ? { session: mostKills.session, kills: mostKills.kills } : null,
    longestNoDeathStreak: longest,
  };
}

/** Durée estimée d'une session : du début à la dernière game validée (ms), null si aucune game */
export function sessionDuration(session: Session): number | null {
  const last = session.games[session.games.length - 1];
  if (!last) return null;
  const ms = new Date(last.timestamp).getTime() - new Date(session.startedAt).getTime();
  return ms > 0 ? ms : null;
}

/** "2 h 05" ou "45 min" */
export function formatDuration(ms: number): string {
  const minutes = Math.round(ms / 60000);
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, "0")}`;
}
